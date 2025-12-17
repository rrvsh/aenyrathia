use askama::Template;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use clap::Args;
use log::info;
use log::warn;
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
            Router::new().route("/wiki/{*article_path}", get(wiki_page)),
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
