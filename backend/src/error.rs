use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
    Validation(&'static str),
}

impl AppError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            AppError::Database(_) | AppError::Migration(_) => ErrorCategory::Database,
            AppError::Validation(_) => ErrorCategory::Validation,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::Database(value)
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        AppError::Migration(value)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("Error during request handling: {self:?}");

        let message = format!("{self:?}");
        let category = self.category();
        let status_code = match category {
            ErrorCategory::Database => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCategory::Validation => StatusCode::BAD_REQUEST,
        };

        (status_code, Json(JsonError { message, category })).into_response()
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCategory {
    Database,
    Validation,
}

#[derive(Serialize, Debug)]
pub struct JsonError {
    pub category: ErrorCategory,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, AppError>;
