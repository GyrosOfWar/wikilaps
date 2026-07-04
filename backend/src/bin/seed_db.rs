use dotenvy::dotenv;
use sqlx::PgPool;
use wikilaps::{config::AppConfig, error::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let config = AppConfig::default();
    let pool = PgPool::connect(&config.database_url).await?;
    Ok(())
}
