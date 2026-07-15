use crate::app::state::AppState;
use crate::filters;
use crate::formatting::{
    HOME_PAGE, normalise_newlines, resolve_article_path, resolve_article_slug, resolve_branch_name,
};
use crate::git::Author;
use crate::routes::auth::{current_user, safe_redirect_path, verify_csrf_cookie};
use crate::routes::errors::render_not_found;
use askama::Template;
use axum::Extension;
use axum::Router;
use axum::extract::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use log::{error, trace};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use tower_cookies::{Cookie, Cookies};

pub struct WikiRouter {}

impl WikiRouter {
    pub fn build(state: AppState) -> Router {
        let handlers = get(article_get).post(article_post);
        Router::new()
            .route("/preview", post(preview_markdown))
            .route("/edit-mode/toggle", post(toggle_edit_mode))
            .route("/", handlers.clone())
            .route("/{*article_path}", handlers)
            .with_state(state)
    }
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    full_name: Option<String>,
    edit_mode: bool,
    raw_file_content: String,
    current_path: String,
    current_path_query: String,
    page_title: String,
    file_tree_html: String,
    csrf_token: Option<String>,
}

#[derive(Clone)]
pub(crate) struct FileTreeNode {
    name: String,
    href: Option<String>,
    is_dir: bool,
    children: Vec<FileTreeNode>,
    is_current: bool,
    is_current_ancestor: bool,
}

#[derive(Default)]
struct TreeBuilderNode {
    children: BTreeMap<String, TreeBuilderNode>,
    href: Option<String>,
    is_current: bool,
}

pub async fn article_get(
    cookies: Cookies,
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
    Extension(db): Extension<Option<SqlitePool>>,
) -> Result<Response, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let current_slug =
        resolve_article_slug(article_path.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let relative_path = resolve_article_path(article_path).map_err(|_| StatusCode::BAD_REQUEST)?;
    let current_path = if current_slug == HOME_PAGE {
        "/".to_string()
    } else {
        format!("/{}", encode_url_path(&current_slug))
    };
    let current_path_query = encode_query_value(&current_path);

    let user = current_user(db.as_ref(), &cookies).await;
    let full_name = user.as_ref().map(|user| user.full_name.clone());
    let edit_mode = if user.is_none() {
        false
    } else {
        cookies
            .get("edit_mode")
            .and_then(|cookie| match cookie.value() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            })
            .unwrap_or(false)
    };
    let branch_name = resolve_branch_name(Some(edit_mode), user.as_ref().map(|user| &user.email));

    let file_content = state.remote.read_file(&relative_path, Some(&branch_name));
    let file_tree_paths = state
        .remote
        .list_markdown_paths("wiki", Some(&branch_name))
        .unwrap_or_default();
    let file_tree = build_file_tree(&file_tree_paths, &current_slug);
    let file_tree_html = render_file_tree_html(&file_tree);
    let raw_file_content = match file_content {
        Some(file_content) => file_content,
        None if edit_mode => String::new(),
        None => {
            return Ok(render_not_found(
                file_tree_html,
                full_name,
                edit_mode,
                current_path_query,
                user.map(|user| user.csrf_token),
            ));
        }
    };
    let page_title = page_title(&current_slug, &raw_file_content);

    ArticleTemplate {
        full_name,
        edit_mode,
        raw_file_content,
        current_path: current_path.clone(),
        current_path_query,
        page_title,
        file_tree_html,
        csrf_token: user.map(|user| user.csrf_token),
    }
    .render()
    .map_or_else(
        |e| {
            error!("Error rendering template for {current_path}: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered).into_response()),
    )
}

#[derive(Deserialize)]
pub struct EditForm {
    markdown: String,
    csrf_token: Option<String>,
}

pub async fn preview_markdown(Form(form): Form<EditForm>) -> Html<String> {
    Html(filters::render_markdown(&form.markdown))
}

#[derive(Deserialize)]
pub struct RedirectQuery {
    redirect_to: Option<String>,
}

#[derive(Deserialize)]
pub struct CsrfForm {
    csrf_token: String,
}

pub async fn toggle_edit_mode(
    cookies: Cookies,
    Query(params): Query<RedirectQuery>,
    Extension(db): Extension<Option<SqlitePool>>,
    Form(form): Form<CsrfForm>,
) -> Result<impl IntoResponse, StatusCode> {
    verify_csrf_cookie(&cookies, &form.csrf_token)?;
    let user = current_user(db.as_ref(), &cookies)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if user.csrf_token != form.csrf_token {
        return Err(StatusCode::FORBIDDEN);
    }

    let current = cookies
        .get("edit_mode")
        .and_then(|cookie| match cookie.value() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
        .unwrap_or(false);

    let mut updated = Cookie::new("edit_mode", (!current).to_string());
    updated.set_path("/");
    cookies.add(updated);

    Ok(axum::response::Redirect::to(&safe_redirect_path(
        params.redirect_to,
    )))
}

pub async fn article_post(
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
    cookies: Cookies,
    Extension(db): Extension<Option<SqlitePool>>,
    Form(form): Form<EditForm>,
) -> StatusCode {
    let Some(user) = current_user(db.as_ref(), &cookies).await else {
        return StatusCode::UNAUTHORIZED;
    };
    let Some(csrf_token) = form.csrf_token.as_deref() else {
        return StatusCode::FORBIDDEN;
    };
    if verify_csrf_cookie(&cookies, csrf_token).is_err() || user.csrf_token != csrf_token {
        return StatusCode::FORBIDDEN;
    }

    let article_path = article_path.map(|Path(article_path)| {
        trace!("article path: {article_path}");
        article_path
    });
    let Ok(relative_path) = resolve_article_path(article_path) else {
        return StatusCode::BAD_REQUEST;
    };
    trace!("file path: {relative_path}");
    let branch_name = resolve_branch_name(Some(true), Some(&user.email));
    let content = normalise_newlines(&form.markdown);
    let author = Author {
        name: user.full_name,
        email: user.email,
    };
    match state
        .remote
        .write_file(&relative_path, &content, Some(&branch_name), Some(&author))
    {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(()) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn page_title(current_slug: &str, raw_file_content: &str) -> String {
    if current_slug == HOME_PAGE {
        return "Aenyrathia".to_string();
    }

    let title = first_heading(raw_file_content).unwrap_or_else(|| {
        current_slug
            .rsplit('/')
            .next()
            .unwrap_or(current_slug)
            .to_string()
    });
    format!("Aenyrathia - {title}")
}

fn first_heading(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let line = line.trim_start();
        let heading = line.strip_prefix("# ")?.trim();
        if heading.is_empty() {
            None
        } else {
            Some(heading.to_string())
        }
    })
}

pub(crate) fn build_file_tree(paths: &[String], current_slug: &str) -> Vec<FileTreeNode> {
    let mut root = TreeBuilderNode::default();

    for path in paths {
        let slug_path = path.trim_end_matches(".md");
        if slug_path.is_empty() || slug_path == HOME_PAGE {
            continue;
        }

        let is_current = slug_path == current_slug;
        let segments: Vec<&str> = slug_path.split('/').collect();
        insert_path(&mut root, &segments, slug_path, is_current);
    }

    let mut nodes = Vec::new();
    for (name, node) in root.children {
        let (template_node, _) = tree_builder_to_template(&name, node);
        nodes.push(template_node);
    }
    sort_nodes(&mut nodes);
    nodes
}

fn insert_path(parent: &mut TreeBuilderNode, segments: &[&str], slug_path: &str, is_current: bool) {
    if let Some((head, tail)) = segments.split_first() {
        let child = parent.children.entry((*head).to_string()).or_default();
        if tail.is_empty() {
            child.href = Some(if slug_path == HOME_PAGE {
                "/".to_string()
            } else {
                format!("/{}", encode_url_path(slug_path))
            });
            child.is_current = is_current;
        } else {
            insert_path(child, tail, slug_path, is_current);
        }
    }
}

fn tree_builder_to_template(name: &str, node: TreeBuilderNode) -> (FileTreeNode, bool) {
    let mut children = Vec::new();
    let mut contains_current = node.is_current;

    for (child_name, child_node) in node.children {
        let (child, child_contains) = tree_builder_to_template(&child_name, child_node);
        contains_current |= child_contains;
        children.push(child);
    }

    sort_nodes(&mut children);

    let is_dir = !children.is_empty();
    let template_node = FileTreeNode {
        name: name.to_string(),
        href: node.href,
        is_dir,
        children,
        is_current: node.is_current,
        is_current_ancestor: contains_current && !node.is_current && is_dir,
    };

    (template_node, contains_current)
}

fn sort_nodes(nodes: &mut [FileTreeNode]) {
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
}

pub(crate) fn render_file_tree_html(nodes: &[FileTreeNode]) -> String {
    let mut output = String::new();
    output.push_str("<ul class=\"file-tree\">");
    render_nodes(nodes, &mut output);
    output.push_str("</ul>");
    output
}

fn render_nodes(nodes: &[FileTreeNode], output: &mut String) {
    for node in nodes {
        output.push_str("<li class=\"file-tree__item\">");
        if node.is_dir {
            let open_attr = if node.is_current || node.is_current_ancestor {
                " open"
            } else {
                ""
            };
            let mut summary_class = "file-tree__summary".to_string();
            if node.is_current {
                summary_class.push_str(" active");
            }
            write!(
                output,
                "<details class=\"file-tree__dir\"{open_attr}><summary class=\"{summary_class}\">"
            )
            .expect("Error appending filetree to string.");
            render_node_link(node, output, true);
            output.push_str("</summary>");
            if !node.children.is_empty() {
                output.push_str("<ul class=\"file-tree\">");
                render_nodes(&node.children, output);
                output.push_str("</ul>");
            }
            output.push_str("</details>");
        } else {
            render_node_link(node, output, false);
        }
        output.push_str("</li>");
    }
}

fn render_node_link(node: &FileTreeNode, output: &mut String, in_summary: bool) {
    let mut class = "file-tree__link".to_string();
    if in_summary {
        class.push_str(" file-tree__link--summary");
    }
    if node.is_current && !in_summary {
        class.push_str(" active");
    }

    if let Some(href) = &node.href {
        write!(
            output,
            "<a href=\"{}\" class=\"{class}\">{}</a>",
            escape_html(href),
            escape_html(&node.name)
        )
        .expect("Error appending filetree to string.");
    } else {
        write!(
            output,
            "<span class=\"{class}\">{}</span>",
            escape_html(&node.name)
        )
        .expect("Error appending filetree to string.");
    }
}

pub(crate) fn encode_query_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char);
            }
            _ => write!(encoded, "%{byte:02X}").expect("Error appending encoded byte."),
        }
    }
    encoded
}

fn encode_url_path(path: &str) -> String {
    let mut encoded = String::new();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char);
            }
            _ => write!(encoded, "%{byte:02X}").expect("Error appending encoded byte."),
        }
    }
    encoded
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
