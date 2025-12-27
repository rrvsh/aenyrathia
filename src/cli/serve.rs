use askama::Template;
use axum::Router;
use axum::ServiceExt;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::get;
use clap::Args;
use log::{debug, error, info, trace, warn};
use markdown::to_html;
use serde::Deserialize;
use std::fs;
use std::path::{self, PathBuf};
use std::process::Command;
use std::time::Duration;
use tempfile::{TempDir, tempdir};
use tokio::signal;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::normalize_path::NormalizePath;
use tower_http::timeout::TimeoutLayer;

#[derive(Args)]
pub struct ServeArgs {}

#[derive(Clone)]
struct AppState {
    wiki_dir: String,
}

impl ServeArgs {
    #[tokio::main]
    pub async fn run() {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let addr = format!("{host}:{port}");

        let git_remote = "git@github.com:rrvsh/aenyrathia.git";
        trace!("Cloning {git_remote} into tempdir.");
        let tempdir = tempdir().expect("Error creating temporary directory;");
        let path = tempdir
            .path()
            .to_str()
            .expect("Invalid UTF-8 in tempdir path!");
        trace!(
            "`git clone {} {}` result: {:?}",
            git_remote,
            path,
            std::process::Command::new("git")
                .args(["clone", git_remote, path])
                .output()
                .expect("git command failed to start")
        );
        std::process::Command::new("git")
            .args(["config", "--global", "user.email", "git@aenyrathia.wiki"])
            .output()
            .expect("git command failed to start");
        std::process::Command::new("git")
            .args(["config", "--global", "user.name", "aenyrathia.wiki"])
            .output()
            .expect("git command failed to start");
        let wiki_dir = tempdir.path().join("wiki").to_string_lossy().to_string();
        let state = AppState { wiki_dir };

        info!("Starting axum router and listening on {addr}");
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let router = Router::new()
            .route("/login", get(login_get).post(login_post))
            .route("/logout", get(logout_get))
            .route("/wiki", get(wiki_index))
            .route("/wiki/{*article_path}", get(wiki_page))
            .route("/edit/wiki/{*article_path}", get(edit_get).post(edit_post))
            .with_state(state)
            .layer((
                CookieManagerLayer::new(),
                TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(10),
                ),
            ));
        let app = NormalizePath::trim_trailing_slash(router);
        let app = ServiceExt::<axum::extract::Request>::into_make_service(app);
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(tempdir))
            .await
            .unwrap();
    }
}

async fn shutdown_signal(tempdir: TempDir) {
    signal::ctrl_c()
        .await
        .expect("Failed to initalise Ctrl C signal handler.");
    trace!("Closing tempdir.");
    tempdir.close().expect("");
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn login_get() -> Result<Html<String>, StatusCode> {
    LoginTemplate {}.render().map_or_else(
        |e| {
            warn!("Error rendering template for /login: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
        |rendered| Ok(Html(rendered)),
    )
}

async fn logout_get(cookies: Cookies) -> Redirect {
    cookies.remove(Cookie::new("username", ""));
    Redirect::to("/wiki")
}

#[derive(Deserialize, Debug)]
struct LoginForm {
    username: String,
}

async fn login_post(cookies: Cookies, Form(form): Form<LoginForm>) -> Result<Redirect, StatusCode> {
    cookies.add(Cookie::new("username", form.username));
    Ok(Redirect::to("/wiki"))
}

#[derive(Template)]
#[template(path = "article.html")]
struct WikiArticleTemplate {
    content: String,
    username: Option<String>,
}

async fn wiki_index(
    cookies: Cookies,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    trace!(
        "Fetching from origin: {:?}",
        Command::new("git")
            .current_dir(&state.wiki_dir)
            .args(["fetch", "origin"])
            .output()
            .expect("error running git command")
    );
    trace!(
        "Checking out prime: {:?}",
        Command::new("git")
            .current_dir(&state.wiki_dir)
            .args(["checkout", "prime"])
            .output()
            .expect("error running git command")
    );
    trace!(
        "Resetting prime to origin: {:?}",
        Command::new("git")
            .current_dir(&state.wiki_dir)
            .args(["reset", "--hard", "origin/prime"])
            .output()
            .expect("error running git command")
    );
    let path = PathBuf::from(state.wiki_dir)
        .join("index")
        .with_extension("md");
    fs::read_to_string(&path).map_or_else(
        |e| {
            warn!("Couldn't read {} to string: {}", path.display(), e);
            Err(StatusCode::NOT_FOUND)
        },
        |file_content| {
            WikiArticleTemplate {
                username: cookies.get("username").map(|c| c.value().to_string()),
                content: to_html(&file_content),
            }
            .render()
            .map_or_else(
                |e| {
                    error!("Error rendering template for {}: {}", path.display(), e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                },
                |rendered| Ok(Html(rendered)),
            )
        },
    )
}

async fn wiki_page(
    Path(article_path): Path<String>,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Html<String>, StatusCode> {
    trace!(
        "Fetching from origin: {:?}",
        Command::new("git")
            .current_dir(&state.wiki_dir)
            .args(["fetch", "origin"])
            .output()
            .expect("error running git command")
    );
    trace!(
        "Checking out prime: {:?}",
        Command::new("git")
            .current_dir(&state.wiki_dir)
            .args(["checkout", "prime"])
            .output()
            .expect("error running git command")
    );
    trace!(
        "Resetting prime to origin: {:?}",
        Command::new("git")
            .current_dir(&state.wiki_dir)
            .args(["reset", "--hard", "origin/prime"])
            .output()
            .expect("error running git command")
    );
    let path = PathBuf::from(state.wiki_dir)
        .join(&article_path)
        .with_extension("md");
    fs::read_to_string(&path).map_or_else(
        |e| {
            warn!("Couldn't read {} to string: {}", path.display(), e);
            Err(StatusCode::NOT_FOUND)
        },
        |file_content| {
            WikiArticleTemplate {
                username: cookies.get("username").map(|c| c.value().to_string()),
                content: to_html(&file_content),
            }
            .render()
            .map_or_else(
                |e| {
                    error!("Error rendering template for {}: {}", path.display(), e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                },
                |rendered| Ok(Html(rendered)),
            )
        },
    )
}

#[derive(Template)]
#[template(path = "editor.html")]
struct EditorTemplate {
    file_content: String,
}

async fn edit_get(
    Path(article_path): Path<String>,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<Html<String>, StatusCode> {
    if let Some(username) = cookies.get("username") {
        let git_branch = format!("user/{}", username.value());
        trace!(
            "Fetching from origin: {:?}",
            Command::new("git")
                .current_dir(&state.wiki_dir)
                .args(["fetch", "origin"])
                .output()
                .expect("error running git command")
        );
        trace!(
            "Checking out {git_branch}: {:?}",
            Command::new("git")
                .current_dir(&state.wiki_dir)
                .args(["checkout", "-B", &git_branch])
                .output()
                .expect("error running git command")
        );
        trace!(
            "Resetting {git_branch} to origin: {:?}",
            Command::new("git")
                .current_dir(&state.wiki_dir)
                .args(["reset", "--hard", &format!("origin/{git_branch}")])
                .output()
                .expect("error running git command")
        );

        let path = PathBuf::from(state.wiki_dir)
            .join(&article_path)
            .with_extension("md");
        if !path_editable(&path) {
            debug!("{} not editable.", path.display());
            return Err(StatusCode::NOT_FOUND);
        }
        trace!("Rendering template for /edit/{}.", path.display());
        fs::read_to_string(&path).map_or_else(
            |_| {
                trace!("file not found at {}: creating new file", path.display());
                EditorTemplate {
                    file_content: String::new(),
                }
                .render()
                .map_or_else(
                    |e| {
                        error!("Error rendering template for {}: {}", path.display(), e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    },
                    |rendered| {
                        trace!("rendered html for /edit/wiki/{article_path}: {rendered}");
                        Ok(Html(rendered))
                    },
                )
            },
            |file_content| {
                trace!("file_content for {}: {}", path.display(), file_content);
                EditorTemplate { file_content }.render().map_or_else(
                    |e| {
                        error!("Error rendering template for {}: {}", path.display(), e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    },
                    |rendered| {
                        trace!("rendered html for /edit/wiki/{article_path}: {rendered}");
                        Ok(Html(rendered))
                    },
                )
            },
        )
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Checks if a file path is editable by checking if any of the following conditions are true:
/// 1. The file already exists and is not a directory
/// 2. The parent directory exists
fn path_editable<P: AsRef<path::Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file() || path.parent().is_some_and(path::Path::is_dir)
}

#[derive(Deserialize)]
struct EditForm {
    markdown: String,
}

fn normalise_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

async fn edit_post(
    Path(article_path): Path<String>,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    if let Some(username) = cookies.get("username") {
        let git_branch = format!("user/{}", username.value());
        trace!(
            "Fetching from origin: {:?}",
            Command::new("git")
                .current_dir(&state.wiki_dir)
                .args(["fetch", "origin"])
                .output()
                .expect("error running git command")
        );
        trace!(
            "Checking out {git_branch}: {:?}",
            Command::new("git")
                .current_dir(&state.wiki_dir)
                .args(["checkout", "-B", &git_branch])
                .output()
                .expect("error running git command")
        );
        trace!(
            "Resetting {git_branch} to origin: {:?}",
            Command::new("git")
                .current_dir(&state.wiki_dir)
                .args(["reset", "--hard", &format!("origin/{git_branch}")])
                .output()
                .expect("error running git command")
        );

        let path = PathBuf::from(&state.wiki_dir)
            .join(&article_path)
            .with_extension("md");
        if !path_editable(&path) {
            debug!("{} not editable.", path.display());
            return Err(StatusCode::NOT_FOUND);
        }

        fs::write(&path, normalise_newlines(&form.markdown)).map_or_else(
            |e| {
                warn!("Couldn't write to file {}: {}", path.display(), e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            },
            |()| {
                trace!("Wrote to file {}", path.display());
                trace!(
                    "Running git add: {:?}",
                    Command::new("git")
                        .current_dir(&state.wiki_dir)
                        .args(["add", path.to_str().expect("invalid UTF-8!")])
                        .output()
                        .expect("error running git command")
                );
                trace!(
                    "Running git commit: {:?}",
                    Command::new("git")
                        .current_dir(&state.wiki_dir)
                        .args(["commit", "-m", username.value()])
                        .output()
                        .expect("error running git command")
                );
                trace!(
                    "Running git push: {:?}",
                    Command::new("git")
                        .current_dir(&state.wiki_dir)
                        .args(["push", "origin", &git_branch])
                        .output()
                        .expect("error running git command")
                );
                let wiki_path = "/wiki/".to_owned() + &article_path;
                Ok(Redirect::to(&wiki_path))
            },
        )
    } else {
        let wiki_path = "/wiki/".to_owned() + &article_path;
        Ok(Redirect::to(&wiki_path))
    }
}
