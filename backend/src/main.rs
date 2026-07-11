use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use wikilaps::{create_router, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let (router, config) = create_router().await?;

    let addr = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting on {addr}");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}
