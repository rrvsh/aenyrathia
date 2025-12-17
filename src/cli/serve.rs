use askama::Template;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use clap::Args;

#[derive(Args)]
pub struct ServeArgs {}

impl ServeArgs {
    #[tokio::main]
    pub async fn run() {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, Router::new().route("/hello", get(hello)))
            .await
            .unwrap();
    }
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn hello() -> Result<String, StatusCode> {
    let template = HelloTemplate { name: "world" };
    template
        .render()
        .map_or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR), Ok)
}
