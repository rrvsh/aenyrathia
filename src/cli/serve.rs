use askama::Template;
use axum::Router;
use axum::ServiceExt;
use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
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

fn get_file_path(article_path: &str, wiki_dir: &str) -> std::path::PathBuf {
    let ensured_article_path = if article_path.is_empty() {
        "index"
    } else {
        article_path
    };
    PathBuf::from(wiki_dir)
        .join(ensured_article_path)
        .with_extension("md")
}

fn checkout_remote_branch(branch_name: &str, repo_directory: &str) {
    trace!(
        "Fetching from origin: {:?}",
        Command::new("git")
            .current_dir(repo_directory)
            .args(["fetch", "origin"])
            .output()
            .expect("error running git command")
    );
    trace!(
        "Checking out {branch_name}: {:?}",
        Command::new("git")
            .current_dir(repo_directory)
            .args(["checkout", "-B", branch_name])
            .output()
            .expect("error running git command")
    );
    trace!(
        "Resetting {branch_name} to origin: {:?}",
        Command::new("git")
            .current_dir(repo_directory)
            .args(["reset", "--hard", &format!("origin/{branch_name}")])
            .output()
            .expect("error running git command")
    );
}

/// Checks if a file path is editable by checking if any of the following conditions are true:
/// 1. The file already exists and is not a directory
/// 2. The parent directory exists
fn path_editable<P: AsRef<path::Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file() || path.parent().is_some_and(path::Path::is_dir)
}

fn normalise_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

struct AppSettings {
    addr: String,
    git_remote: String,
}

impl AppSettings {
    fn from_env() -> Self {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        Self {
            addr: format!("{host}:{port}"),
            git_remote: "git@github.com:rrvsh/aenyrathia.git".to_string(),
        }
    }
}

#[derive(Clone)]
struct AppState {
    wiki_dir: String,
}

impl AppState {
    fn new(settings: &AppSettings, tempdir: &TempDir) -> Self {
        trace!("Cloning {} into tempdir.", settings.git_remote);
        let path = tempdir
            .path()
            .to_str()
            .expect("Invalid UTF-8 in tempdir path!");
        trace!(
            "`git clone {} {}` result: {:?}",
            &settings.git_remote,
            path,
            std::process::Command::new("git")
                .args(["clone", &settings.git_remote, path])
                .output()
                .expect("git command failed to start")
        );
        std::process::Command::new("git")
            .current_dir(path)
            .args(["config", "user.email", "git@aenyrathia.wiki"])
            .output()
            .expect("git command failed to start");
        std::process::Command::new("git")
            .current_dir(path)
            .args(["config", "user.name", "aenyrathia.wiki"])
            .output()
            .expect("git command failed to start");
        Self {
            wiki_dir: tempdir.path().join("wiki").to_string_lossy().to_string(),
        }
    }
}

#[derive(Args)]
pub struct ServeArgs {}

impl ServeArgs {
    #[tokio::main]
    pub async fn run() {
        let settings = AppSettings::from_env();
        let tempdir = tempdir().expect("Error creating temporary directory;");
        let state = AppState::new(&settings, &tempdir);

        info!("Starting axum router and listening on {}", &settings.addr);
        let listener = tokio::net::TcpListener::bind(&settings.addr).await.unwrap();
        let router = Router::new()
            .route("/login", post(login))
            .route("/logout", get(logout))
            .route(
                "/{*article_path}",
                get(render_wiki_page).post(update_wiki_page),
            )
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

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
}

async fn login(cookies: Cookies, Form(form): Form<LoginRequest>) -> Redirect {
    cookies.add(Cookie::new("username", form.username));
    Redirect::to("/")
}

async fn logout(cookies: Cookies) -> Redirect {
    cookies.remove(Cookie::new("username", ""));
    Redirect::to("/")
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate {
    username: Option<String>,
    edit_mode: bool,
    raw_file_content: String,
    rendered_html: String,
}

#[derive(Deserialize)]
struct EditModeQuery {
    edit_mode: Option<bool>,
}

/// Checks out the latest revision from origin/prime.
/// Renders the requested page, or index.md if not present
async fn render_wiki_page(
    cookies: Cookies,
    Path(article_path): Path<String>,
    Query(params): Query<EditModeQuery>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let git_branch = if params.edit_mode.unwrap_or(false) {
        cookies.get("username").map_or_else(
            || "prime".to_string(),
            |username| format!("user/{}", username.value()),
        )
    } else {
        "prime".to_string()
    };
    checkout_remote_branch(&git_branch, &state.wiki_dir);
    let path = get_file_path(&article_path, &state.wiki_dir);
    fs::read_to_string(&path).map_or_else(
        |e| {
            warn!("Couldn't read {} to string: {}", path.display(), e);
            Err(StatusCode::NOT_FOUND)
            //FIXME: if file_path is editable, edit_mode, and logged in, render empty file_content
        },
        |file_content| {
            ArticleTemplate {
                edit_mode: params.edit_mode.unwrap_or(false),
                username: cookies.get("username").map(|c| c.value().to_string()),
                raw_file_content: file_content.clone(),
                rendered_html: to_html(&file_content),
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

#[derive(Deserialize)]
struct EditForm {
    markdown: String,
}

async fn update_wiki_page(
    Path(article_path): Path<String>,
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    if let Some(username) = cookies.get("username") {
        let git_branch = format!("user/{}", username.value());
        checkout_remote_branch(&git_branch, &state.wiki_dir);

        let path = get_file_path(&article_path, &state.wiki_dir);
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
                Ok(Redirect::to(&("/".to_owned() + &article_path)))
            },
        )
    } else {
        Ok(Redirect::to(&("/".to_owned() + &article_path)))
    }
}
