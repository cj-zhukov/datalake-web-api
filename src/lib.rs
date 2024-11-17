use anyhow::Result;
use axum::{routing::{get, post}, serve::Serve, Router};
use routes::{ping, post_select, post_download_id, post_download};

pub mod routes;
pub mod data_store;
pub mod utils;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    fn new(server: Serve<Router, Router>, address: String) -> Self {
        Self { server, address }
    }

    pub async fn build(address: &str) -> Result<Self> {
        let router = Router::new()
            .route("/", get(|| async { "datalake web api" }))
            .route("/alive", get(ping))
            .route("/select", post(post_select))
            .route("/download", post(post_download))
            .route("/download/:id", post(post_download_id));

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application::new(server, address))
    }

    pub async fn run(self) -> Result<()> {
        println!("listening on {}", &self.address);
        self.server.await?;

        Ok(())
    }
}