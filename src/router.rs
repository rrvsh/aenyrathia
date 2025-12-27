use crate::handlers;
use crate::wiki::Wiki;
use axum::http::StatusCode;
use axum::routing::{get, post};
use std::time::Duration;
use tower_cookies::CookieManagerLayer;
use tower_http::timeout::TimeoutLayer;

pub fn build(wiki: Wiki) -> axum::Router {
    axum::Router::new()
        .route("/login", post(handlers::login))
        .route("/logout", get(handlers::logout))
        .route(
            "/",
            get(handlers::render_wiki_page).post(handlers::update_wiki_page),
        )
        .route(
            "/{*article_path}",
            get(handlers::render_wiki_page).post(handlers::update_wiki_page),
        )
        .with_state(wiki)
        .layer((
            CookieManagerLayer::new(),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
        ))
}
