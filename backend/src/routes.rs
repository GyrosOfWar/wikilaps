use crate::{
    auth::{self, UserId},
    database::{self, Database, RaceWeekend, SessionType, SessionWithVotes, VoteType},
    error::Result,
    pagination::{Page, PageParameters},
    util::{voting_allowed, weekend_upcoming},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use jiff::{Timestamp, civil::Date};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    /// HMAC secret for signing/verifying identity cookies. `Arc` so cloning the
    /// state per request is cheap.
    pub cookie_secret: Arc<[u8]>,
    /// Whether identity cookies get the `Secure` attribute (HTTPS only).
    pub cookie_secure: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RaceWeekendResponse {
    pub id: i64,
    pub year: i32,
    pub location: String,
    pub circuit_full_name: String,
    pub grand_prix_id: String,
    pub country_key: String,
    pub start_date: Date,
    pub round: i32,
    pub official_name: String,
    pub sessions: Vec<SessionResponse>,
    /// True while none of the weekend's sessions have started yet — the UI
    /// renders such weekends as disabled. Computed server-side so it stays
    /// consistent with each session's `voting_allowed`.
    pub upcoming: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub id: i64,
    pub session_type: SessionType,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub votes: VoteCounts,
    pub user_vote: Option<VoteType>,
    pub voting_allowed: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VoteCounts {
    pub full: i64,
    pub race_in_30: Option<i64>,
    pub highlights: i64,
}

impl From<RaceWeekend> for RaceWeekendResponse {
    fn from(value: RaceWeekend) -> Self {
        let sessions: Vec<SessionResponse> = value
            .sessions
            .into_iter()
            .map(SessionResponse::from)
            .collect();
        let start_times: Vec<Timestamp> = sessions.iter().map(|s| s.start_time).collect();
        RaceWeekendResponse {
            id: value.id,
            year: value.year,
            location: value.location,
            circuit_full_name: value.circuit_full_name,
            grand_prix_id: value.grand_prix_id,
            country_key: value.country_key,
            start_date: value.start_date.to_jiff(),
            round: value.round,
            official_name: value.official_name,
            upcoming: weekend_upcoming(Timestamp::now(), &start_times),
            sessions,
        }
    }
}

impl From<SessionWithVotes> for SessionResponse {
    fn from(value: SessionWithVotes) -> Self {
        let start_time = value.start_time.to_jiff();
        SessionResponse {
            id: value.id,
            session_type: value.session_type,
            start_time,
            end_time: value.end_time.map(|t| t.to_jiff()),
            user_vote: value.user_vote,
            voting_allowed: voting_allowed(start_time, value.session_type),
            votes: match value.session_type {
                SessionType::Race => VoteCounts {
                    full: value.votes.full_race,
                    race_in_30: Some(value.votes.race_in_30),
                    highlights: value.votes.highlights,
                },
                _ => VoteCounts {
                    full: value.votes.full_race,
                    race_in_30: None,
                    highlights: value.votes.highlights,
                },
            },
        }
    }
}

#[axum::debug_handler]
#[utoipa::path(
    method(get),
    path = "/api/race/years",
    responses(
        (status = OK, description = "Success", body = Vec<i32>)
    )
)]
pub async fn get_years_of_data(state: State<AppState>) -> Result<Json<Vec<i32>>> {
    let years = state.db.find_years_of_data().await?;
    Ok(Json(years))
}

#[axum::debug_handler]
#[utoipa::path(
    method(get),
    path = "/api/race/weekends/latest",
    responses(
        (status = OK, description = "Success", body = Option<RaceWeekendResponse>)
    )
)]
pub async fn get_latest_weekend(
    state: State<AppState>,
    user: Option<UserId>,
) -> Result<Json<Option<RaceWeekendResponse>>> {
    let mut closest = state
        .db
        .find_last_weekend(user.as_ref().map(|u| u.0.as_str()))
        .await?
        .map(RaceWeekendResponse::from);
    let now = Timestamp::now();
    // only send the last elapsed session
    if let Some(ref mut weekend) = closest {
        weekend.sessions.retain(|s| s.start_time < now);
        if !weekend.sessions.is_empty() {
            weekend.sessions = vec![weekend.sessions.pop().unwrap()];
        }
    }

    Ok(Json(closest))
}

#[axum::debug_handler]
#[utoipa::path(
    method(get),
    path = "/api/race/weekends/{year}",
    params(
        ("year" = i32, Path),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<RaceWeekendResponse>)
    )
)]
pub async fn list_weekends(
    state: State<AppState>,
    user: Option<UserId>,
    Path(year): Path<i32>,
) -> Result<Json<Vec<RaceWeekendResponse>>> {
    let weekends: Vec<_> = state
        .db
        .list_weekends(year, user.as_ref().map(|u| u.0.as_str()))
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
#[utoipa::path(method(get), path = "/api/session")]
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

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VoteRequest {
    pub session_id: i64,
    pub vote: VoteType,
}

/// Cast a vote for a session on behalf of the browser identified by the signed
/// cookie. The `(user_identifier, session_id)` unique constraint means a
/// browser's first vote for a session wins; subsequent votes are ignored.
#[axum::debug_handler]
#[utoipa::path(method(post), path = "/api/vote", request_body = VoteRequest)]
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

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionListResponse {
    pub id: i64,
    pub grand_prix_id: String,
    pub country_key: String,
    pub race_weekend_start_date: Date,
    pub session_start_time: Timestamp,
    pub round: i32,
    pub session_type: SessionType,
    pub votes: VoteCounts,
}

impl From<database::Session> for SessionListResponse {
    fn from(s: database::Session) -> Self {
        SessionListResponse {
            id: s.id,
            grand_prix_id: s.grand_prix_id,
            country_key: s.country_key,
            race_weekend_start_date: s.race_weekend_start_date.to_jiff(),
            session_start_time: s.session_start_time.to_jiff(),
            round: s.round,
            session_type: s.session_type,
            votes: match s.session_type {
                SessionType::Race => VoteCounts {
                    full: s.votes.full_race,
                    race_in_30: Some(s.votes.race_in_30),
                    highlights: s.votes.highlights,
                },
                _ => VoteCounts {
                    full: s.votes.full_race,
                    race_in_30: None,
                    highlights: s.votes.highlights,
                },
            },
        }
    }
}

#[derive(Deserialize, IntoParams, Debug)]
#[into_params(parameter_in = Query)]
pub struct SessionListFilter {
    pub year: Option<i32>,

    #[serde(rename = "type")]
    pub session_type: Option<SessionType>,
}

#[axum::debug_handler]
#[utoipa::path(
    path = "/api/race/sessions", 
    params(PageParameters, SessionListFilter),
    method(get), responses(
        (status = OK, description = "Success", body = Page<SessionListResponse>)
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(page): Query<PageParameters>,
    Query(filter): Query<SessionListFilter>,
) -> Result<Json<Page<SessionListResponse>>> {
    info!(page = ?page, filter = ?filter, "list_sessions");

    let page = state
        .db
        .list_sessions(page, filter)
        .await?
        .map(SessionListResponse::from);
    Ok(Json(page))
}
