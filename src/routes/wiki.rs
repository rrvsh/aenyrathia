use crate::app::state::AppState;
use crate::filters;
use crate::formatting::{normalise_newlines, resolve_article_path, resolve_branch_name};
use crate::git::Author;
use askama::Template;
use axum::Router;
use axum::extract::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use log::{error, trace};
use serde::Deserialize;
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
    prefix_path: String,
    file_tree_html: String,
}

#[derive(Clone)]
struct FileTreeNode {
    name: String,
    href: String,
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
) -> Result<Response, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let full_name = cookies
        .get("full_name")
        .map(|cookie| cookie.value().to_string());
    let (branch_prefix, article_path) = parse_branch_prefix(article_path.as_deref(), |branch| {
        state.remote.branch_exists(branch)
    });
    let has_prefix = branch_prefix.is_some();
    let relative_path = resolve_article_path(article_path.clone());
    let prefix_path = branch_prefix
        .as_deref()
        .map(|prefix| format!("/{prefix}"))
        .unwrap_or_default();
    let current_path = match (branch_prefix.as_deref(), article_path.as_deref()) {
        (Some(prefix), Some(path)) if !path.is_empty() => format!("/{prefix}/{path}"),
        (Some(prefix), _) => format!("/{prefix}"),
        (None, Some(path)) if !path.is_empty() => format!("/{path}"),
        _ => "/".to_string(),
    };
    let edit_cookie = cookies
        .get("edit_mode")
        .and_then(|cookie| match cookie.value() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
        .unwrap_or(false);
    let edit_mode = resolve_edit_mode(has_prefix, full_name.as_deref(), edit_cookie);
    let branch_name = if let Some(prefix) = branch_prefix.as_ref() {
        format!("user/{prefix}")
    } else {
        resolve_branch_name(Some(edit_mode), full_name.as_ref())
    };
    let current_slug = article_path
        .as_deref()
        .filter(|path| !path.is_empty())
        .unwrap_or("index")
        .to_string();

    let file_content = state.remote.read_file(&relative_path, Some(&branch_name));
    let file_tree_paths = state
        .remote
        .list_markdown_paths("wiki", Some(&branch_name))
        .unwrap_or_default();
    let file_tree = build_file_tree(&file_tree_paths, &current_slug, &prefix_path);
    let file_tree_html = render_file_tree_html(&file_tree);
    let mut raw_file_content = String::new();
    if let Some(file_content) = file_content {
        raw_file_content = file_content;
    } else {
        match missing_file_response(has_prefix, edit_mode) {
            Ok(()) => {}
            Err(StatusCode::TEMPORARY_REDIRECT) => {
                return Ok(Redirect::to("/").into_response());
            }
            Err(status) => return Err(status),
        }
    }
    ArticleTemplate {
        full_name,
        edit_mode,
        raw_file_content,
        current_path: current_path.clone(),
        prefix_path,
        file_tree_html,
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
}

pub async fn preview_markdown(Form(form): Form<EditForm>) -> Html<String> {
    Html(markdown::to_html(&form.markdown))
}

#[derive(Deserialize)]
pub struct RedirectQuery {
    redirect_to: Option<String>,
}

pub async fn toggle_edit_mode(cookies: Cookies, Query(params): Query<RedirectQuery>) -> Redirect {
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

    let redirect = params.redirect_to.unwrap_or_else(|| "/".to_string());
    Redirect::to(&redirect)
}

pub async fn article_post(
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> StatusCode {
    let article_path = article_path.map(|Path(article_path)| {
        trace!("article path: {article_path}");
        article_path
    });
    let (branch_prefix, article_path) = parse_branch_prefix(article_path.as_deref(), |branch| {
        state.remote.branch_exists(branch)
    });
    if branch_prefix.is_some() {
        return StatusCode::FORBIDDEN;
    }
    if let Some(full_name) = cookies.get("full_name") {
        let relative_path = resolve_article_path(article_path);
        trace!("file path: {relative_path}");
        let branch_name = resolve_branch_name(Some(true), Some(&full_name.value().to_string()));
        let content = normalise_newlines(&form.markdown);
        let author = cookies.get("email").map(|email| Author {
            name: full_name.value().to_string(),
            email: email.value().to_string(),
        });
        match state.remote.write_file(
            &relative_path,
            &content,
            Some(&branch_name),
            author.as_ref(),
        ) {
            Ok(()) => StatusCode::NO_CONTENT,
            Err(()) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::NO_CONTENT
    }
}

fn build_file_tree(paths: &[String], current_slug: &str, base_href: &str) -> Vec<FileTreeNode> {
    let mut root = TreeBuilderNode::default();

    for path in paths {
        let slug_path = path.trim_end_matches(".md");
        if slug_path.is_empty() {
            continue;
        }

        let is_current = slug_path == current_slug;
        let segments: Vec<&str> = slug_path.split('/').collect();
        insert_path(&mut root, &segments, slug_path, is_current, base_href);
    }

    let mut nodes = Vec::new();
    for (name, node) in root.children {
        let (template_node, _) = tree_builder_to_template(&name, node);
        nodes.push(template_node);
    }
    sort_nodes(&mut nodes);
    nodes
}

fn insert_path(
    parent: &mut TreeBuilderNode,
    segments: &[&str],
    slug_path: &str,
    is_current: bool,
    base_href: &str,
) {
    if let Some((head, tail)) = segments.split_first() {
        let child = parent.children.entry((*head).to_string()).or_default();
        if tail.is_empty() {
            let trimmed_base = base_href.trim_end_matches('/');
            let target_path = if slug_path == "index" {
                trimmed_base.to_string()
            } else if trimmed_base.is_empty() {
                format!("/{slug_path}")
            } else {
                format!("{trimmed_base}/{slug_path}")
            };
            child.href = Some(if target_path.is_empty() {
                "/".to_string()
            } else {
                target_path
            });
            child.is_current = is_current;
        } else {
            insert_path(child, tail, slug_path, is_current, base_href);
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

    let is_dir = node.href.is_none();
    let template_node = FileTreeNode {
        name: name.to_string(),
        href: node.href.unwrap_or_default(),
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

fn render_file_tree_html(nodes: &[FileTreeNode]) -> String {
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
            let open_attr = if node.is_current_ancestor {
                " open"
            } else {
                ""
            };
            write!(
                output,
                "<details class=\"file-tree__dir\"{open_attr}><summary>{}</summary>",
                escape_html(&node.name)
            )
            .expect("Error appending filetree to string.");
            if !node.children.is_empty() {
                output.push_str("<ul class=\"file-tree\">");
                render_nodes(&node.children, output);
                output.push_str("</ul>");
            }
            output.push_str("</details>");
        } else {
            let mut class = "file-tree__link".to_string();
            if node.is_current {
                class.push_str(" active");
            }
            write!(
                output,
                "<a href=\"{}\" class=\"{class}\">{}</a>",
                escape_html(&node.href),
                escape_html(&node.name)
            )
            .expect("Error appending filetree to string.");
        }
        output.push_str("</li>");
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn parse_branch_prefix(
    path: Option<&str>,
    branch_exists: impl Fn(&str) -> bool,
) -> (Option<String>, Option<String>) {
    let Some(path) = path.filter(|path| !path.is_empty()) else {
        return (None, None);
    };
    let mut segments = path.splitn(2, '/');
    let candidate = segments.next().unwrap_or_default();
    let remainder = segments.next().unwrap_or_default();
    let branch_name = format!("user/{candidate}");
    if branch_exists(&branch_name) {
        let remainder = remainder.to_string();
        let remainder = if remainder.is_empty() {
            None
        } else {
            Some(remainder)
        };
        (Some(candidate.to_string()), remainder)
    } else {
        (None, Some(path.to_string()))
    }
}

fn resolve_edit_mode(has_prefix: bool, full_name: Option<&str>, edit_cookie: bool) -> bool {
    if has_prefix || full_name.is_none() {
        false
    } else {
        edit_cookie
    }
}

fn missing_file_response(has_prefix: bool, edit_mode: bool) -> Result<(), StatusCode> {
    if has_prefix {
        Err(StatusCode::NOT_FOUND)
    } else if !edit_mode {
        Err(StatusCode::TEMPORARY_REDIRECT)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_file_tree, missing_file_response, parse_branch_prefix, render_file_tree_html,
        resolve_edit_mode,
    };
    use axum::http::StatusCode;

    #[test]
    fn parse_branch_prefix_resolves_known_branch() {
        let (prefix, rest) = parse_branch_prefix(Some("mohammad-rafiq/campaigns"), |branch| {
            branch == "user/mohammad-rafiq"
        });
        assert_eq!(prefix.as_deref(), Some("mohammad-rafiq"));
        assert_eq!(rest.as_deref(), Some("campaigns"));
    }

    #[test]
    fn parse_branch_prefix_falls_back_when_missing() {
        let (prefix, rest) = parse_branch_prefix(Some("docs/getting-started"), |_| false);
        assert!(prefix.is_none());
        assert_eq!(rest.as_deref(), Some("docs/getting-started"));
    }

    #[test]
    fn parse_branch_prefix_handles_empty_path() {
        let (prefix, rest) = parse_branch_prefix(None, |_| true);
        assert!(prefix.is_none());
        assert!(rest.is_none());
    }

    #[test]
    fn edit_mode_disabled_when_prefix_present() {
        assert!(!resolve_edit_mode(true, Some("User"), true));
    }

    #[test]
    fn edit_mode_enabled_for_logged_in_user_without_prefix() {
        assert!(resolve_edit_mode(false, Some("User"), true));
    }

    #[test]
    fn edit_mode_disabled_for_anonymous_user() {
        assert!(!resolve_edit_mode(false, None, true));
    }

    #[test]
    fn missing_file_with_prefix_returns_404() {
        let result = missing_file_response(true, false);
        assert_eq!(result, Err(StatusCode::NOT_FOUND));
    }

    #[test]
    fn missing_file_without_prefix_redirects_when_not_editing() {
        let result = missing_file_response(false, false);
        assert_eq!(result, Err(StatusCode::TEMPORARY_REDIRECT));
    }

    #[test]
    fn missing_file_without_prefix_allows_edit_mode() {
        let result = missing_file_response(false, true);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn file_tree_links_include_prefix() {
        let paths = vec!["notes/todo.md".to_string()];
        let tree = build_file_tree(&paths, "notes/todo", "/mohammad-rafiq");
        let html = render_file_tree_html(&tree);
        assert!(html.contains("/mohammad-rafiq/notes/todo"));
    }
}
