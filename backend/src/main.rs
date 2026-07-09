use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;
use wikilaps::{
    config::AppConfig,
    database::Database,
    docs::ApiDocs,
    error::Result,
    routes::{self, AppState},
};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    let config = AppConfig::default();
    let database = Database::new(&config.database_url).await?;

    let (router, api) = OpenApiRouter::with_openapi(ApiDocs::openapi())
        .routes(routes!(routes::list_weekends))
        .routes(routes!(routes::init_session))
        .routes(routes!(routes::create_vote))
        .with_state(AppState {
            db: database,
            cookie_secret: config.cookie_secret.into(),
            cookie_secure: config.cookie_secure,
        })
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    info!("Starting on port 13252");
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, config.server_port))
            .await
            .unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}
