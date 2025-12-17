use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::get;
use pinbreak::domain::markdown::Markdown;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(
        listener,
        Router::new()
            .route("/hello", get(hello))
            .route("/wiki/{*article_path}", get(wiki_page)),
    )
    .await
    .unwrap();
}

async fn hello() -> String {
    "Hello, World!".to_string()
}

async fn wiki_page(Path(article_path): Path<String>) -> Result<String, StatusCode> {
    // note: article_path resolves without preceding slash
    let wiki_directory = "wiki/".to_string();
    Markdown::from_path(wiki_directory + &article_path + ".md")
        .map_or_else(|_| Err(StatusCode::NOT_FOUND), |md| Ok(md.as_html()))
}
