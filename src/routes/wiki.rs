use crate::app::state::AppState;
use crate::formatting::{normalise_newlines, resolve_article_path, resolve_branch_name};
use askama::Template;
use axum::Router;
use axum::extract::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::get;
use log::error;
use markdown::to_html;
use serde::Deserialize;
use tower_cookies::Cookies;

pub struct WikiRouter {}

impl WikiRouter {
    pub fn build(state: AppState) -> Router {
        let handlers = get(article_get).put(article_put);
        Router::new()
            .route("/", handlers.clone())
            .route("/{*article_path}", handlers)
            .with_state(state)
    }
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    username: Option<String>,
    edit_mode: bool,
    raw_file_content: String,
    rendered_html: String,
}

#[derive(Deserialize)]
pub struct EditModeQuery {
    edit_mode: Option<bool>,
}

pub async fn article_get(
    cookies: Cookies,
    article_path: Option<Path<String>>,
    Query(params): Query<EditModeQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let username = cookies
        .get("username")
        .map(|cookie| cookie.value().to_string());
    let redirect_path = String::from("/") + &article_path.clone().unwrap_or_default();
    let relative_path = resolve_article_path(article_path);
    let branch_name = resolve_branch_name(params.edit_mode, username.as_ref());
    let edit_mode = if username.is_none() {
        false
    } else {
        params.edit_mode.unwrap_or(false)
    };

    let file_content = state.remote.read_file(&relative_path, Some(&branch_name));
    let mut raw_file_content = String::new();
    let mut rendered_html = String::new();
    if let Some(file_content) = file_content {
        rendered_html = to_html(&file_content);
        raw_file_content = file_content;
    } else if !edit_mode {
        return Err(StatusCode::NOT_FOUND);
    }
    ArticleTemplate {
        username,
        edit_mode,
        raw_file_content,
        rendered_html,
    }
    .render()
    .map_or_else(
        |e| {
            error!("Error rendering template for {redirect_path}: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

#[derive(Deserialize)]
pub struct EditForm {
    markdown: String,
}

pub async fn article_put(
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let redirect_path = String::from("/") + &article_path.clone().unwrap_or_default();
    if let Some(username) = cookies.get("username") {
        let relative_path = resolve_article_path(article_path);
        let branch_name = Some(format!("user/{}", username.value()));
        let content = normalise_newlines(&form.markdown);
        match state
            .remote
            .write_file(&relative_path, &content, branch_name.as_deref())
        {
            Ok(()) => Ok(Redirect::to(&redirect_path)),
            Err(()) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Ok(Redirect::to(&redirect_path))
    }
}
