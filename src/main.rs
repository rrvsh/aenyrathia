use app::{settings, state};
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Request, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::{Extension, Router, ServiceExt};
use log::{error, info, warn};
use routes::auth::AuthRouter;
use routes::wiki::WikiRouter;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;
use tower_cookies::CookieManagerLayer;
use tower_http::normalize_path::NormalizePath;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;

mod app;
mod filters;
mod formatting;
mod git;
mod routes;

#[tokio::main]
async fn main() {
    let settings = settings::AppSettings::from_env();

    let env = env_logger::Env::new().filter("PB_LOG");
    let mut builder = env_logger::Builder::from_env(env);
    builder.init();

    let state = state::AppState::init(&settings);

    let db = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(settings.db_options.clone())
        .await
    {
        Ok(db) => match sqlx::migrate!().run(&db).await {
            Ok(()) => Some(db),
            Err(err) => {
                error!("Database migrations failed; login/register disabled: {err}");
                None
            }
        },
        Err(err) => {
            warn!("Database connection failed; login/register disabled: {err}");
            None
        }
    };

    let router = Router::new()
        .merge(AuthRouter::build())
        .merge(WikiRouter::build(state))
        .nest_service("/static", ServeDir::new(settings.static_dir.clone()))
        .fallback(not_found)
        .layer((
            middleware::from_fn(add_response_headers),
            DefaultBodyLimit::max(1024 * 1024),
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

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn add_response_headers(request: Request<Body>, next: Next) -> Response {
    let is_static = request.uri().path().starts_with("/static/");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("SAMEORIGIN"),
    );

    if is_static {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    response
}
