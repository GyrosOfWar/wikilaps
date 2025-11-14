use axum::{Router, routing::get};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::database::Database;

mod database;

pub type Result<T> = color_eyre::Result<T>;

pub struct AppConfig {
    pub database_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

async fn hello_world() -> &'static str {
    "hello, world!"
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
        .route("/", get(hello_world))
        .with_state(AppState { db: database });

    info!("Starting on port 13252");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:13252")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
