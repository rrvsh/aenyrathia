use axum::ServiceExt;
use log::info;
use tower_http::normalize_path::NormalizePath;

mod formatting;
mod handlers;
mod router;
mod settings;
mod wiki;

#[tokio::main]
async fn main() {
    let env = env_logger::Env::new().filter("PB_LOG");
    let mut builder = env_logger::Builder::from_env(env);
    builder.init();

    let settings = settings::AppSettings::from_env();
    let tempdir = tempfile::tempdir().expect("Error creating temporary directory;");
    let wiki = wiki::Wiki::from_remote(&settings.git_remote, &tempdir);

    info!("Starting app and listening on {}", &settings.addr);
    let listener = tokio::net::TcpListener::bind(&settings.addr).await.unwrap();
    let app = NormalizePath::trim_trailing_slash(router::build(wiki));
    let app = ServiceExt::<axum::extract::Request>::into_make_service(app);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(tempdir))
        .await
        .unwrap();
}

async fn shutdown_signal(tempdir: tempfile::TempDir) {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to initalise Ctrl C signal handler.");
    tempdir.close().expect("");
}
