use app::{settings, state};
use axum::http::StatusCode;
use axum::{Extension, Router, ServiceExt};
use log::info;
use routes::auth::AuthRouter;
use routes::wiki::WikiRouter;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower_cookies::CookieManagerLayer;
use tower_http::normalize_path::NormalizePath;
use tower_http::timeout::TimeoutLayer;

mod app;
mod formatting;
mod git;
mod routes;

#[tokio::main]
async fn main() {
    let settings = settings::AppSettings::from_env();
    let state = state::AppState::init(&settings);

    let env = env_logger::Env::new().filter("PB_LOG");
    let mut builder = env_logger::Builder::from_env(env);
    builder.init();

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&settings.db_url)
        .await
        .expect("");
    sqlx::migrate!().run(&db).await.expect("Migration failed!");

    let router = Router::new()
        .merge(AuthRouter::build())
        .merge(WikiRouter::build(state))
        .layer((
            CookieManagerLayer::new(),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
            Extension(db),
        ));

    info!("Starting app and listening on {}", &settings.addr);
    let listener = tokio::net::TcpListener::bind(&settings.addr).await.unwrap();
    let app = NormalizePath::trim_trailing_slash(router);
    let app = ServiceExt::<axum::extract::Request>::into_make_service(app);
    axum::serve(listener, app).await.unwrap();
}
