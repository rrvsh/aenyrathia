use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use log::error;

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate {
    file_tree_html: String,
}

pub async fn not_found() -> Response {
    render_not_found(String::new())
}

pub fn render_not_found(file_tree_html: String) -> Response {
    match (NotFoundTemplate { file_tree_html }).render() {
        Ok(rendered) => (StatusCode::NOT_FOUND, Html(rendered)).into_response(),
        Err(err) => {
            error!("Error rendering 404 template: {err}");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
