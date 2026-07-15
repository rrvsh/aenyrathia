use crate::app::state::AppState;
use crate::formatting::{HOME_PAGE, resolve_branch_name};
use crate::routes::auth::current_user;
use crate::routes::wiki::{build_file_tree, encode_query_value, render_file_tree_html};
use askama::Template;
use axum::Extension;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use log::error;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate {
    full_name: Option<String>,
    edit_mode: bool,
    current_path_query: String,
    file_tree_html: String,
    csrf_token: Option<String>,
}

pub async fn not_found(
    cookies: Cookies,
    OriginalUri(uri): OriginalUri,
    Extension(db): Extension<Option<SqlitePool>>,
    Extension(state): Extension<AppState>,
) -> Response {
    let user = current_user(db.as_ref(), &cookies).await;
    let edit_mode = edit_mode(&cookies, user.is_some());
    let branch_name = resolve_branch_name(Some(edit_mode), user.as_ref().map(|user| &user.email));
    let file_tree_html = file_tree_html(&state, Some(&branch_name), HOME_PAGE);
    let current_path_query =
        encode_query_value(uri.path_and_query().map_or(uri.path(), |v| v.as_str()));
    render_not_found(
        file_tree_html,
        user.as_ref().map(|user| user.full_name.clone()),
        edit_mode,
        current_path_query,
        user.map(|user| user.csrf_token),
    )
}

pub fn render_not_found(
    file_tree_html: String,
    full_name: Option<String>,
    edit_mode: bool,
    current_path_query: String,
    csrf_token: Option<String>,
) -> Response {
    match (NotFoundTemplate {
        full_name,
        edit_mode,
        current_path_query,
        file_tree_html,
        csrf_token,
    })
    .render()
    {
        Ok(rendered) => (StatusCode::NOT_FOUND, Html(rendered)).into_response(),
        Err(err) => {
            error!("Error rendering 404 template: {err}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub(crate) fn file_tree_html(
    state: &AppState,
    branch_name: Option<&str>,
    current_slug: &str,
) -> String {
    let file_tree_paths = state
        .remote
        .list_markdown_paths("wiki", branch_name)
        .unwrap_or_default();
    let file_tree = build_file_tree(&file_tree_paths, current_slug);
    render_file_tree_html(&file_tree)
}

pub(crate) fn edit_mode(cookies: &Cookies, has_user: bool) -> bool {
    if !has_user {
        return false;
    }

    cookies
        .get("edit_mode")
        .and_then(|cookie| match cookie.value() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
        .unwrap_or(false)
}
