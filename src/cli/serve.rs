use askama::Template;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use clap::Args;
use log::{error, info, trace, warn};
use markdown::to_html;
use std::fs;

#[derive(Args)]
pub struct ServeArgs {}

impl ServeArgs {
    #[tokio::main]
    pub async fn run() {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!("Starting axum router and listening on 0.0.0.0:3000");
        axum::serve(
            listener,
            Router::new()
                .route("/wiki/{*article_path}", get(wiki_page))
                .route("/edit/wiki/{*article_path}", get(edit_get)),
        )
        .await
        .unwrap();
    }
}

#[derive(Template)]
#[template(path = "article.html")]
struct WikiArticleTemplate {
    content: String,
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
    let wiki_directory = "wiki/".to_string();
    let file_path = wiki_directory + &article_path + ".md";
    trace!("Rendering template for /edit/{file_path}.");
    if !parent_directory_exists(&file_path) {
        warn!("Parent directory doesn't exist for {file_path}.");
        return Err(StatusCode::NOT_FOUND);
    }
    fs::read_to_string(&file_path).map_or_else(
        |e| {
            warn!("Couldn't read {file_path} to string: {e}");
            Err(StatusCode::NOT_FOUND)
        },
        |file_content| {
            trace!("file_content for {file_path}: {file_content}");
            EditorTemplate { file_content }.render().map_or_else(
                |e| {
                    error!("Error rendering template for {file_path}: {e}");
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

const fn parent_directory_exists(_file_path: &str) -> bool {
    true
}
