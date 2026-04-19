use app::{settings, state};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{Extension, Router, ServiceExt};
use log::{error, info, warn};
use routes::auth::AuthRouter;
use routes::wiki::WikiRouter;
use sqlx::sqlite::{SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
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
    let state = state::AppState::init(&settings);

    let env = env_logger::Env::new().filter("PB_LOG");
    let mut builder = env_logger::Builder::from_env(env);
    builder.init();

    let db_options = settings
        .db_options
        .clone()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));
    let db = match SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(db_options)
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
        .nest_service("/static", ServeDir::new("static"))
        .fallback(redirect_to_index)
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

async fn redirect_to_index() -> Redirect {
    Redirect::to("/")
}
