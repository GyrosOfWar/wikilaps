use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
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

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error!("Error during request handling: {self:?}");

        let status_code = StatusCode::INTERNAL_SERVER_ERROR;
        let message = format!("{self:?}");

        (status_code, Json(JsonError { message })).into_response()
    }
}

#[derive(Serialize, Debug)]
pub struct JsonError {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, AppError>;
