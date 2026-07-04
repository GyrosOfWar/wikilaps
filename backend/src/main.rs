use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;
use wikilaps::{config::AppConfig, database::Database, error::Result, routes, routes::AppState};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let config = AppConfig::default();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let database = Database::new(&config.database_url).await?;

    let app = Router::new()
        .route("/api/race-weekends", get(routes::list_weekends))
        .route("/api/vote", post(routes::create_vote))
        .with_state(AppState { db: database });

    info!("Starting on port 13252");
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, config.server_port))
            .await
            .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
