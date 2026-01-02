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
use axum::routing::{get, post};
use log::error;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

pub struct WikiRouter {}

impl WikiRouter {
    pub fn build(state: AppState) -> Router {
        let handlers = get(article_get).post(article_post);
        Router::new()
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
}

pub async fn article_get(
    cookies: Cookies,
    article_path: Option<Path<String>>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let article_path = article_path.map(|Path(article_path)| article_path);
    let full_name = cookies
        .get("full_name")
        .map(|cookie| cookie.value().to_string());
    let current_path = String::from("/") + &article_path.clone().unwrap_or_default();
    let relative_path = resolve_article_path(article_path);
    let edit_mode = if full_name.is_none() {
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
    let branch_name = resolve_branch_name(Some(edit_mode), full_name.as_ref());

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
    let article_path = article_path.map(|Path(article_path)| article_path);
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
            Ok(()) => StatusCode::NO_CONTENT,
            Err(()) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        StatusCode::NO_CONTENT
    }
}
