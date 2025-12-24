use askama::Template;
use axum::Router;
use axum::ServiceExt;
use axum::extract::{Form, Path};
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::get;
use clap::Args;
use log::{debug, error, info, trace, warn};
use markdown::to_html;
use serde::Deserialize;
use std::fs;
use std::path::{self, PathBuf};
use tower_http::normalize_path::NormalizePath;

#[derive(Args)]
pub struct ServeArgs {}

impl ServeArgs {
    #[tokio::main]
    pub async fn run() {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        let router = Router::new()
            .route("/login", get(login_get).post(login_post))
            .route("/wiki", get(wiki_index))
            .route("/wiki/{*article_path}", get(wiki_page))
            .route("/edit/wiki/{*article_path}", get(edit_get).post(edit_post));
        let app = NormalizePath::trim_trailing_slash(router);
        let app = ServiceExt::<axum::extract::Request>::into_make_service(app);
        info!("Starting axum router and listening on 0.0.0.0:3000");
        axum::serve(listener, app).await.unwrap();
    }
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

#[derive(Deserialize, Debug)]
struct LoginForm {
    username: String,
}

async fn login_post(Form(form): Form<LoginForm>) -> Result<Redirect, StatusCode> {
    // get the username passed in and store it as a cookie
    info!("username: {}", form.username);
    Ok(Redirect::to("/wiki"))
}

#[derive(Template)]
#[template(path = "article.html")]
struct WikiArticleTemplate {
    content: String,
    username: String,
}

async fn wiki_index() -> Result<Html<String>, StatusCode> {
    let wiki_directory = "wiki/".to_string();
    let file_path = wiki_directory + "index.md";
    fs::read_to_string(&file_path).map_or_else(
        |e| {
            warn!("Couldn't read {file_path} to string: {e}");
            Err(StatusCode::NOT_FOUND)
        },
        |file_content| {
            WikiArticleTemplate {
                username: "rafiq".to_string(),
                content: to_html(&file_content),
            }
            .render()
            .map_or_else(
                |e| {
                    warn!("Error rendering template for {file_path}: {e}");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                },
                |rendered| Ok(Html(rendered)),
            )
        },
    )
}

async fn wiki_page(Path(article_path): Path<String>) -> Result<Html<String>, StatusCode> {
    // note: article_path resolves without preceding slash
    let wiki_directory = "wiki/".to_string();
    let file_path = wiki_directory + &article_path + ".md";
    fs::read_to_string(&file_path).map_or_else(
        |e| {
            warn!("Couldn't read {file_path} to string: {e}");
            Err(StatusCode::NOT_FOUND)
        },
        |file_content| {
            WikiArticleTemplate {
                username: "rafiq".to_string(),
                content: to_html(&file_content),
            }
            .render()
            .map_or_else(
                |e| {
                    warn!("Error rendering template for {file_path}: {e}");
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

async fn edit_get(Path(article_path): Path<String>) -> Result<Html<String>, StatusCode> {
    // note: article_path resolves without preceding slash
    let path = PathBuf::from("wiki")
        .join(&article_path)
        .with_extension("md");
    trace!("Rendering template for /edit/{}.", path.display());
    if !path_editable(&path) {
        debug!("{} not editable.", path.display());
        return Err(StatusCode::NOT_FOUND);
    }
    fs::read_to_string(&path).map_or_else(
        |e| {
            warn!("Couldn't read {} to string: {}", path.display(), e);
            Err(StatusCode::NOT_FOUND)
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
    Form(form): Form<EditForm>,
) -> Result<Redirect, StatusCode> {
    let path = PathBuf::from("wiki")
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
            let wiki_path = "/wiki/".to_owned() + &article_path;
            Ok(Redirect::to(&wiki_path))
        },
    )
}
