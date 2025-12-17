use axum::Router;
use clap::Args;
use log::info;

#[derive(Args)]
pub struct ServeArgs {}

impl ServeArgs {
    #[tokio::main]
    pub async fn run() {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!("Starting axum router and listening on 0.0.0.0:3000");
        axum::serve(listener, Router::new()).await.unwrap();
    }
}
