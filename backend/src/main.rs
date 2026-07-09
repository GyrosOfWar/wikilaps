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

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let config = AppConfig::default();
    let database = Database::new(&config.database_url).await?;

    let app = Router::new()
        .route("/api/race-weekends/{year}", get(routes::list_weekends))
        .route("/api/session", post(routes::init_session))
        .route("/api/vote", post(routes::create_vote))
        .with_state(AppState {
            db: database,
            cookie_secret: config.cookie_secret.into(),
            cookie_secure: config.cookie_secure,
        });

    info!("Starting on port 13252");
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, config.server_port))
            .await
            .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
