use axum::{Router, routing::get};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{database::Database, error::Result, routes::AppState};

mod database;
mod error;
mod pagination;
mod routes;

pub struct AppConfig {
    pub database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();
    let db_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL env var");
    let database = Database::new(&db_url).await?;

    let app = Router::new()
        .route("/api/race-weekends", get(routes::list_weekends))
        .with_state(AppState { db: database });

    info!("Starting on port 13252");
    let listener = tokio::net::TcpListener::bind("localhost:13252")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
