use crate::app::state::AppState;
use crate::filters;
use crate::formatting::{normalise_newlines, resolve_article_path, resolve_branch_name};
use crate::git::Author;
use askama::Template;
use axum::Router;
use axum::extract::Form;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::get;
use log::error;
use serde::Deserialize;
use tower_cookies::Cookies;

pub struct WikiRouter {}

impl WikiRouter {
    pub fn build(state: AppState) -> Router {
        let handlers = get(article_get).post(article_post);
        Router::new()
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
    let full_name = cookies
        .get("full_name")
        .map(|cookie| cookie.value().to_string());
    let current_path = String::from("/") + &article_path.clone().unwrap_or_default();
    let relative_path = resolve_article_path(article_path);
    let branch_name = resolve_branch_name(params.edit_mode, full_name.as_ref());
    let edit_mode = if full_name.is_none() {
        false
    } else {
        params.edit_mode.unwrap_or(false)
    };

    let file_content = state.remote.read_file(&relative_path, Some(&branch_name));
    let mut raw_file_content = String::new();
    if let Some(file_content) = file_content {
        raw_file_content = file_content;
    } else if !edit_mode {
        return Err(StatusCode::NOT_FOUND);
    }
    ArticleTemplate {
        full_name,
        edit_mode,
        raw_file_content,
        current_path: current_path.clone(),
    }
    .render()
    .map_or_else(
        |e| {
            error!("Error rendering template for {current_path}: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

#[derive(Deserialize)]
pub struct EditForm {
    markdown: String,
}

pub async fn article_post(
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let redirect_path = String::from("/") + &article_path.clone().unwrap_or_default();
    if let Some(full_name) = cookies.get("full_name") {
        let relative_path = resolve_article_path(article_path);
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
            Ok(()) => Ok(Redirect::to(&redirect_path)),
            Err(()) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Ok(Redirect::to(&redirect_path))
    }
}
