use axum::{Router, routing::get};
use dotenvy::dotenv;
use sqlx::PgPool;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub type Result<T> = color_eyre::Result<T>;

pub struct AppConfig {
    pub database_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

#[derive(Clone)]
pub struct Database {
    db: PgPool,
}

impl Database {
    pub async fn new(db_url: &str) -> Result<Self> {
        let db = PgPool::connect(db_url).await?;
        sqlx::migrate!("./migrations").run(&db).await?;

        Ok(Self { db })
    }

    pub async fn list_weekends(&self) -> Result<()> {
        let data = sqlx::query!("SELECT * FROM race_weekend")
            .fetch_all(&self.db)
            .await?;
        Ok(())
    }
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
