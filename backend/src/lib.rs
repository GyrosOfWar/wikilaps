use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::database::Database;
use crate::docs::ApiDocs;
use crate::error::Result;
use crate::routes::AppState;

pub mod auth;
pub mod config;
pub mod database;
pub mod docs;
pub mod error;
pub mod pagination;
pub mod routes;
pub mod util;

pub async fn create_router() -> Result<(Router, AppConfig)> {
    let config = AppConfig::default();
    let database = Database::new(&config.database_url).await?;

    let (router, api) = OpenApiRouter::with_openapi(ApiDocs::openapi())
        .routes(routes!(routes::get_latest_weekend))
        .routes(routes!(routes::list_weekends))
        .routes(routes!(routes::init_session))
        .routes(routes!(routes::create_vote))
        .routes(routes!(routes::get_years_of_data))
        .with_state(AppState {
            db: database,
            cookie_secret: config.cookie_secret.clone().into(),
            cookie_secure: config.cookie_secure,
        })
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));
    Ok((router, config))
}
