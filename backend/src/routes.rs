use crate::{
    auth::{self, UserId},
    database::{Database, RaceWeekend, SessionType, SessionWithVotes, VoteType},
    error::Result,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use jiff::{Timestamp, civil::Date};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    /// HMAC secret for signing/verifying identity cookies. `Arc` so cloning the
    /// state per request is cheap.
    pub cookie_secret: Arc<[u8]>,
    /// Whether identity cookies get the `Secure` attribute (HTTPS only).
    pub cookie_secure: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceWeekendResponse {
    pub id: i64,
    pub year: i32,
    pub location: String,
    pub circuit_name: String,
    pub country_key: String,
    pub start_date: Date,
    pub round: i32,
    pub official_name: String,
    pub sessions: Vec<SessionResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub id: i64,
    pub session_type: SessionType,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub votes: VoteCounts,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteCounts {
    pub full_race: i64,
    pub race_in_30: i64,
    pub highlights: i64,
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
            official_name: value.official_name,
            sessions: value
                .sessions
                .into_iter()
                .map(SessionResponse::from)
                .collect(),
        }
    }
}

impl From<SessionWithVotes> for SessionResponse {
    fn from(value: SessionWithVotes) -> Self {
        SessionResponse {
            id: value.id,
            session_type: value.session_type,
            start_time: value.start_time.to_jiff(),
            end_time: value.end_time.map(|t| t.to_jiff()),
            votes: VoteCounts {
                full_race: value.votes.full_race,
                race_in_30: value.votes.race_in_30,
                highlights: value.votes.highlights,
            },
        }
    }
}

#[axum::debug_handler]
pub async fn list_weekends(
    state: State<AppState>,
    Path(year): Path<i32>,
) -> Result<Json<Vec<RaceWeekendResponse>>> {
    let weekends: Vec<_> = state
        .db
        .list_weekends(year)
        .await?
        .into_iter()
        .map(From::from)
        .collect();

    Ok(Json(weekends))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitSessionResponse {
    /// Whether this call created a fresh identity or the browser already had a valid one
    pub created: bool,
}

/// Called by the frontend when the user opens the site. Issues a signed
/// identity cookie if the browser doesn't already have a valid one, and is a
/// no-op (keeping the existing identity) otherwise.
#[axum::debug_handler]
pub async fn init_session(
    state: State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> (CookieJar, Json<InitSessionResponse>) {
    let existing = jar
        .get(auth::COOKIE_NAME)
        .and_then(|cookie| auth::verify_token(&state.cookie_secret, cookie.value()));

    if existing.is_some() {
        return (jar, Json(InitSessionResponse { created: false }));
    }

    let (token, cookie_value) = auth::issue_token(&state.cookie_secret);
    info!(
        user_id = %token,
        ip = auth::client_ip(&headers).as_deref().unwrap_or("unknown"),
        user_agent = auth::user_agent(&headers).as_deref().unwrap_or("unknown"),
        "issued new browser identity"
    );

    let jar = jar.add(auth::build_cookie(cookie_value, state.cookie_secure));
    (jar, Json(InitSessionResponse { created: true }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRequest {
    pub session_id: i64,
    pub vote: VoteType,
}

/// Cast a vote for a session on behalf of the browser identified by the signed
/// cookie. The `(user_identifier, session_id)` unique constraint means a
/// browser's first vote for a session wins; subsequent votes are ignored.
#[axum::debug_handler]
pub async fn create_vote(
    state: State<AppState>,
    user: UserId,
    headers: HeaderMap,
    Json(request): Json<VoteRequest>,
) -> Result<StatusCode> {
    info!(
        user_id = %user.0,
        session_id = request.session_id,
        vote = ?request.vote,
        ip = auth::client_ip(&headers).as_deref().unwrap_or("unknown"),
        "recording vote"
    );

    state
        .db
        .insert_vote(&user.0, request.session_id, request.vote)
        .await?;

    Ok(StatusCode::CREATED)
}
