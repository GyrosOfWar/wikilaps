use crate::{
    database::RaceWeekend,
    error::{AppError, Result},
};
use axum::{Json, extract::State, http::HeaderMap};
use jiff::civil::Date;
use serde::Serialize;

use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

#[derive(Debug, Serialize)]
pub struct RaceWeekendResponse {
    pub id: i64,
    pub year: i32,
    pub location: String,
    pub circuit_name: String,
    pub country_key: String,
    pub start_date: Date,
    pub round: i32,
}

impl From<RaceWeekend> for RaceWeekendResponse {
    fn from(value: RaceWeekend) -> Self {
        RaceWeekendResponse {
            id: value.id,
            year: value.year,
            location: value.location,
            circuit_name: value.circuit_name,
            country_key: value.country_key,
            start_date: value.start_date.to_jiff(),
            round: value.round,
        }
    }
}

#[axum::debug_handler]
pub async fn list_weekends(state: State<AppState>) -> Result<Json<Vec<RaceWeekendResponse>>> {
    let weekends: Vec<_> = state
        .db
        .list_weekends()
        .await?
        .into_iter()
        .map(From::from)
        .collect();

    Ok(Json(weekends))
}

#[axum::debug_handler]
pub async fn create_vote(headers: HeaderMap) -> Result<()> {
    Ok(())
}
