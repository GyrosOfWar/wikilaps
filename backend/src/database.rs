use crate::{
    error::{
        AppError::{self, Validation},
        Result,
    },
    pagination::{Page, PageParameters},
    routes::SessionListFilter,
    util::voting_allowed,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "session_type", rename_all = "snake_case")]
pub enum SessionType {
    SprintQualifying,
    SprintRace,
    Qualifying,
    Race,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
#[sqlx(type_name = "vote_type", rename_all = "PascalCase")]
pub enum VoteType {
    FullRace,
    RaceIn30,
    Highlights,
}

#[derive(Debug)]
pub struct Session {
    pub id: i64,
    pub grand_prix_id: String,
    pub country_key: String,
    pub race_weekend_start_date: jiff_sqlx::Date,
    pub session_start_time: jiff_sqlx::Timestamp,
    pub round: i32,
    pub votes: VoteCounts,
    pub session_type: SessionType,
}

#[derive(Debug)]
pub struct RaceWeekend {
    pub id: i64,
    pub year: i32,
    pub location: String,
    pub official_name: String,
    pub circuit_full_name: String,
    pub grand_prix_id: String,
    pub country_key: String,
    pub start_date: jiff_sqlx::Date,
    pub round: i32,
    pub sessions: Vec<SessionWithVotes>,
}

#[derive(Debug)]
pub struct SessionWithVotes {
    pub id: i64,
    pub session_type: SessionType,
    pub start_time: jiff_sqlx::Timestamp,
    pub end_time: Option<jiff_sqlx::Timestamp>,
    pub votes: VoteCounts,
    pub user_vote: Option<VoteType>,
}

/// Tally of votes for a single session, one count per `vote_type`.
#[derive(Debug)]
pub struct VoteCounts {
    pub full_race: i64,
    pub race_in_30: i64,
    pub highlights: i64,
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

    pub fn pool(&self) -> &PgPool {
        &self.db
    }

    pub async fn find_last_weekend(&self, user_id: Option<&str>) -> Result<Option<RaceWeekend>> {
        let rows = sqlx::query!(
            r#"WITH last_weekend AS (
                    SELECT id FROM race_weekend WHERE start_date < now() ORDER BY start_date DESC LIMIT 1
                )
                SELECT
                    r.id AS weekend_id, r.year, r.location, r.circuit_full_name, r.grand_prix_id, r.country_key,
                    r.start_date AS "start_date: jiff_sqlx::Date", r.round, r.official_name,
                    s.id AS "session_id?",
                    s.session_type AS "session_type?: SessionType",
                    s.start_time AS "start_time?: jiff_sqlx::Timestamp",
                    s.end_time AS "end_time?: jiff_sqlx::Timestamp",
                    count(v.id) FILTER (WHERE v.vote_type = 'FullRace'::vote_type)   AS "full_race!",
                    count(v.id) FILTER (WHERE v.vote_type = 'RaceIn30'::vote_type)   AS "race_in_30!",
                    count(v.id) FILTER (WHERE v.vote_type = 'Highlights'::vote_type) AS "highlights!",
                    max(v.vote_type) FILTER (WHERE v.user_identifier = $1) AS "user_vote?: VoteType"
                FROM race_weekend r
                JOIN last_weekend lw ON lw.id = r.id
                LEFT JOIN session s ON s.weekend_id = r.id
                LEFT JOIN votes v ON v.session_id = s.id
                GROUP BY r.id, s.id
                ORDER BY s.start_time ASC NULLS FIRST"#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut weekend: Option<RaceWeekend> = None;
        for row in rows {
            let weekend = weekend.get_or_insert_with(|| RaceWeekend {
                id: row.weekend_id,
                year: row.year,
                location: row.location.clone(),
                official_name: row.official_name.clone(),
                circuit_full_name: row.circuit_full_name.clone(),
                grand_prix_id: row.grand_prix_id.clone(),
                country_key: row.country_key.clone(),
                start_date: row.start_date,
                round: row.round,
                sessions: Vec::new(),
            });

            // A null `session_id` means this weekend has no sessions (the LEFT
            // JOIN produced a single all-null session row); skip it.
            if let Some(session_id) = row.session_id {
                weekend.sessions.push(SessionWithVotes {
                    id: session_id,
                    session_type: row.session_type.expect("session_id implies session_type"),
                    start_time: row.start_time.expect("session_id implies start_time"),
                    end_time: row.end_time,
                    votes: VoteCounts {
                        full_race: row.full_race,
                        race_in_30: row.race_in_30,
                        highlights: row.highlights,
                    },
                    user_vote: row.user_vote,
                });
            }
        }

        Ok(weekend)
    }

    pub async fn list_weekends(
        &self,
        year: i32,
        user_id: Option<&str>,
    ) -> Result<Vec<RaceWeekend>> {
        let rows = sqlx::query!(
            r#"SELECT
                    r.id AS weekend_id, r.year, r.location, r.circuit_full_name, r.grand_prix_id, r.country_key,
                    r.start_date AS "start_date: jiff_sqlx::Date", r.round, r.official_name,
                    s.id AS "session_id?",
                    s.session_type AS "session_type?: SessionType",
                    s.start_time AS "start_time?: jiff_sqlx::Timestamp",
                    s.end_time AS "end_time?: jiff_sqlx::Timestamp",
                    count(v.id) FILTER (WHERE v.vote_type = 'FullRace'::vote_type)   AS "full_race!",
                    count(v.id) FILTER (WHERE v.vote_type = 'RaceIn30'::vote_type)   AS "race_in_30!",
                    count(v.id) FILTER (WHERE v.vote_type = 'Highlights'::vote_type) AS "highlights!",
                    max(v.vote_type) FILTER (WHERE v.user_identifier = $2) AS "user_vote?: VoteType"
                FROM race_weekend r
                LEFT JOIN session s ON s.weekend_id = r.id
                LEFT JOIN votes v ON v.session_id = s.id
                WHERE r.year = $1
                GROUP BY r.id, s.id
                ORDER BY r.start_date ASC, r.id ASC, s.start_time ASC NULLS FIRST"#,
            year,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut weekends: Vec<RaceWeekend> = Vec::new();
        for row in rows {
            if weekends.last().map(|w| w.id) != Some(row.weekend_id) {
                weekends.push(RaceWeekend {
                    id: row.weekend_id,
                    year: row.year,
                    location: row.location,
                    circuit_full_name: row.circuit_full_name,
                    grand_prix_id: row.grand_prix_id,
                    country_key: row.country_key,
                    start_date: row.start_date,
                    round: row.round,
                    official_name: row.official_name,
                    sessions: Vec::new(),
                });
            }

            // A null `session_id` means this weekend has no sessions (the LEFT
            // JOIN produced a single all-null session row); skip it.
            if let Some(session_id) = row.session_id {
                let weekend = weekends.last_mut().expect("weekend pushed above");
                weekend.sessions.push(SessionWithVotes {
                    id: session_id,
                    session_type: row.session_type.expect("session_id implies session_type"),
                    start_time: row.start_time.expect("session_id implies start_time"),
                    end_time: row.end_time,
                    votes: VoteCounts {
                        full_race: row.full_race,
                        race_in_30: row.race_in_30,
                        highlights: row.highlights,
                    },
                    user_vote: row.user_vote,
                });
            }
        }

        Ok(weekends)
    }

    /// Inserts a race weekend, or updates it in place if one with the same
    /// `(year, round)` already exists. Used by the f1db seeder, which needs
    /// to be safely re-runnable as the source schedule changes.
    pub async fn upsert_race_weekend(
        &self,
        year: i32,
        round: i32,
        location: &str,
        circuit_id: &str,
        circuit_full_name: &str,
        grand_prix_id: &str,
        country_key: &str,
        official_name: &str,
        start_date: jiff_sqlx::Date,
    ) -> Result<i64> {
        let id = sqlx::query_scalar!(
            r#"INSERT INTO race_weekend (
                    year, round, location, circuit_id, circuit_full_name,
                    grand_prix_id, country_key, start_date, official_name
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (year, round) DO UPDATE SET
                    location = EXCLUDED.location,
                    circuit_full_name = EXCLUDED.circuit_full_name,
                    country_key = EXCLUDED.country_key,
                    start_date = EXCLUDED.start_date
                RETURNING id"#,
            year,
            round,
            location,
            circuit_id,
            circuit_full_name,
            grand_prix_id,
            country_key,
            start_date as _,
            official_name,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(id)
    }

    /// Inserts a session for a weekend, identified by `(weekend_id,
    /// session_type, start_time)`. Existing rows are left untouched — sessions
    /// don't carry any other seeded data yet, so there's nothing to update.
    pub async fn upsert_session(
        &self,
        weekend_id: i64,
        session_type: SessionType,
        start_time: jiff_sqlx::Timestamp,
    ) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO session (weekend_id, session_type, start_time)
                VALUES ($1, $2, $3)
                ON CONFLICT (weekend_id, session_type, start_time) DO NOTHING"#,
            weekend_id,
            session_type as SessionType,
            start_time as _,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn list_sessions(
        &self,
        page: PageParameters,
        filter: SessionListFilter,
    ) -> Result<Page<Session>> {
        const VALID_SORT: [&str; 2] = ["score", "start_date"];

        let sort_valid = page
            .sort
            .as_deref()
            .map(|s| VALID_SORT.contains(&s))
            .unwrap_or(true);

        if !sort_valid {
            return Err(Validation("Invalid sort parameter"));
        }

        let rows = sqlx::query!(
            r#"SELECT 
                s.id, rw.grand_prix_id, rw.country_key,
                rw.start_date AS "race_weekend_start_time: jiff_sqlx::Date",
                s.start_time AS "session_start_time: jiff_sqlx::Timestamp",
                rw.round, s.session_type AS "session_type: SessionType",
                count(v.id) FILTER (WHERE v.vote_type = 'FullRace'::vote_type)   AS "full_race!",
                count(v.id) FILTER (WHERE v.vote_type = 'RaceIn30'::vote_type)   AS "race_in_30!",
                count(v.id) FILTER (WHERE v.vote_type = 'Highlights'::vote_type) AS "highlights!"
            FROM session s
            JOIN race_weekend rw ON s.weekend_id = rw.id
            LEFT JOIN votes v on v.session_id = s.id
            WHERE ($1::integer IS NULL OR rw.year = $1)
                AND ($2::session_type IS NULL OR s.session_type = $2)
                AND s.start_time < NOW()
            GROUP BY rw.id, s.id
            ORDER BY
                CASE
                    WHEN $5 = 'start_date' THEN rw.start_date
                END DESC,
                CASE
                    WHEN $5 = 'score' THEN
                        count(v.id) FILTER (WHERE v.vote_type = 'FullRace'::vote_type)::float8
                        / NULLIF(count(v.id), 0)
                END DESC NULLS LAST
            LIMIT $3
            OFFSET $4
            "#,
            filter.year,
            filter.session_type as _,
            page.limit(),
            page.offset(),
            page.sort.as_deref().unwrap_or("start_date"),
        )
        .fetch_all(&self.db)
        .await?;

        let count = sqlx::query_scalar!("SELECT count(*) FROM session")
            .fetch_one(&self.db)
            .await?
            .unwrap_or_default();

        let sessions: Vec<_> = rows
            .into_iter()
            .map(|r| Session {
                id: r.id,
                grand_prix_id: r.grand_prix_id,
                country_key: r.country_key,
                race_weekend_start_date: r.race_weekend_start_time,
                session_start_time: r.session_start_time,
                session_type: r.session_type,
                round: r.round,
                votes: VoteCounts {
                    full_race: r.full_race,
                    race_in_30: r.race_in_30,
                    highlights: r.highlights,
                },
            })
            .collect();

        Ok(Page::new(sessions, count as u32, page))
    }

    pub async fn insert_vote(
        &self,
        user_id: &str,
        session_id: i64,
        vote_type: VoteType,
    ) -> Result<()> {
        let row = sqlx::query!(
            r#"SELECT start_time AS "start_time: jiff_sqlx::Timestamp", session_type AS "session_type: SessionType"
            FROM session WHERE id = $1"#,
            session_id
        )
        .fetch_one(&self.db)
        .await?;

        if !voting_allowed(row.start_time.to_jiff(), row.session_type) {
            return Err(AppError::Validation(
                "Voting is not allowed yet for this session, try again later.",
            ));
        }

        sqlx::query!(
            "INSERT INTO votes (vote_type, user_identifier, session_id)
                VALUES ($1, $2, $3)
            ON CONFLICT (user_identifier, session_id) DO NOTHING",
            vote_type as _,
            user_id,
            session_id,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn find_years_of_data(&self) -> Result<Vec<i32>> {
        sqlx::query_scalar!("SELECT DISTINCT year FROM race_weekend ORDER BY year ASC")
            .fetch_all(&self.db)
            .await
            .map_err(From::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::Timestamp as JiffTimestamp;
    use jiff::civil::date;
    use sqlx::PgPool;

    fn db(pool: PgPool) -> Database {
        Database { db: pool }
    }

    async fn seed_weekend(db: &Database, year: i32, round: i32) -> i64 {
        seed_weekend_with_date(db, year, round, date(year as i16, 9, round as i8)).await
    }

    async fn seed_weekend_with_date(
        db: &Database,
        year: i32,
        round: i32,
        start_date: jiff::civil::Date,
    ) -> i64 {
        db.upsert_race_weekend(
            year,
            round,
            "Monza",
            "monza",
            "Autodromo Nazionale Monza",
            "emilia-romagna",
            "ITA",
            "Formula 1 AWS Gran Premio del Made in Italy e dell'Emilia Romagna 2025",
            jiff_sqlx::Date::from(start_date),
        )
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn list_weekends_empty(pool: PgPool) {
        let db = db(pool);
        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert!(weekends.is_empty());
    }

    #[sqlx::test]
    async fn list_weekends_filters_by_year(pool: PgPool) {
        let db = db(pool);
        seed_weekend(&db, 2023, 1).await;
        seed_weekend(&db, 2024, 1).await;

        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert_eq!(weekends.len(), 1);
        assert_eq!(weekends[0].year, 2024);
    }

    #[sqlx::test]
    async fn list_weekends_orders_by_start_date_asc(pool: PgPool) {
        let db = db(pool);
        seed_weekend(&db, 2024, 1).await;
        seed_weekend(&db, 2024, 2).await;

        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert_eq!(weekends.len(), 2);
        assert_eq!(weekends[0].round, 1);
        assert_eq!(weekends[1].round, 2);
    }

    #[sqlx::test]
    async fn list_weekends_tallies_votes_per_session(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend(&db, 2024, 1).await;
        db.upsert_session(
            weekend_id,
            SessionType::Race,
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        let session_id: i64 = sqlx::query_scalar!("SELECT id FROM session")
            .fetch_one(&db.db)
            .await
            .unwrap();

        db.insert_vote("u1", session_id, VoteType::FullRace)
            .await
            .unwrap();
        db.insert_vote("u2", session_id, VoteType::FullRace)
            .await
            .unwrap();
        db.insert_vote("u3", session_id, VoteType::Highlights)
            .await
            .unwrap();

        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert_eq!(weekends.len(), 1);
        let sessions = &weekends[0].sessions;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        assert_eq!(sessions[0].votes.full_race, 2);
        assert_eq!(sessions[0].votes.race_in_30, 0);
        assert_eq!(sessions[0].votes.highlights, 1);
    }

    #[sqlx::test]
    async fn list_weekends_reports_requesting_users_own_vote(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend(&db, 2024, 1).await;
        db.upsert_session(
            weekend_id,
            SessionType::Qualifying,
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        db.upsert_session(
            weekend_id,
            SessionType::Race,
            jiff_sqlx::Timestamp::from("2024-09-02T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        let session_ids: Vec<i64> =
            sqlx::query_scalar!("SELECT id FROM session ORDER BY start_time ASC")
                .fetch_all(&db.db)
                .await
                .unwrap();
        let (qualifying_id, race_id) = (session_ids[0], session_ids[1]);

        db.insert_vote("me", race_id, VoteType::FullRace)
            .await
            .unwrap();
        db.insert_vote("someone-else", qualifying_id, VoteType::Highlights)
            .await
            .unwrap();

        // Anonymous request: no session carries the user's own vote.
        let anon = db.list_weekends(2024, None).await.unwrap();
        assert!(anon[0].sessions.iter().all(|s| s.user_vote.is_none()));

        // "me" only sees their own vote on the race, not the other user's.
        let mine = db.list_weekends(2024, Some("me")).await.unwrap();
        let sessions = &mine[0].sessions;
        assert!(matches!(
            sessions.iter().find(|s| s.id == race_id).unwrap().user_vote,
            Some(VoteType::FullRace)
        ));
        assert!(
            sessions
                .iter()
                .find(|s| s.id == qualifying_id)
                .unwrap()
                .user_vote
                .is_none()
        );
    }

    #[sqlx::test]
    async fn list_weekends_includes_weekend_without_sessions(pool: PgPool) {
        let db = db(pool);
        seed_weekend(&db, 2024, 1).await;

        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert_eq!(weekends.len(), 1);
        assert!(weekends[0].sessions.is_empty());
    }

    #[sqlx::test]
    async fn upsert_race_weekend_inserts_new_row(pool: PgPool) {
        let db = db(pool);
        let id = seed_weekend(&db, 2024, 1).await;

        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert_eq!(weekends.len(), 1);
        assert_eq!(weekends[0].id, id);
        assert_eq!(weekends[0].location, "Monza");
    }

    #[sqlx::test]
    async fn upsert_race_weekend_updates_on_conflict(pool: PgPool) {
        let db = db(pool);
        let id = seed_weekend(&db, 2024, 1).await;

        let updated_id = db
            .upsert_race_weekend(
                2024,
                1,
                "Imola",
                "imola",
                "Autodromo Enzo e Dino Ferrari",
                "emilia-romagna",
                "ITA",
                "Formula 1 Made in Italy e dell'Emilia Romagna 2024",
                jiff_sqlx::Date::from(date(2024, 9, 1)),
            )
            .await
            .unwrap();

        assert_eq!(id, updated_id);

        let weekends = db.list_weekends(2024, None).await.unwrap();
        assert_eq!(weekends.len(), 1);
        assert_eq!(weekends[0].location, "Imola");
    }

    #[sqlx::test]
    async fn upsert_session_inserts_new_row(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend(&db, 2024, 1).await;

        db.upsert_session(
            weekend_id,
            SessionType::Race,
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();

        let count: i64 = sqlx::query_scalar!("SELECT count(*) FROM session")
            .fetch_one(&db.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn upsert_session_is_idempotent_on_conflict(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend(&db, 2024, 1).await;
        let start_time =
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap());

        db.upsert_session(weekend_id, SessionType::Race, start_time)
            .await
            .unwrap();
        db.upsert_session(weekend_id, SessionType::Race, start_time)
            .await
            .unwrap();

        let count: i64 = sqlx::query_scalar!("SELECT count(*) FROM session")
            .fetch_one(&db.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn insert_vote_inserts_new_row(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend(&db, 2024, 1).await;
        db.upsert_session(
            weekend_id,
            SessionType::Race,
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        let session_id: i64 = sqlx::query_scalar!("SELECT id FROM session")
            .fetch_one(&db.db)
            .await
            .unwrap();

        db.insert_vote("user-1", session_id, VoteType::Highlights)
            .await
            .unwrap();

        let count: i64 = sqlx::query_scalar!("SELECT count(*) FROM votes")
            .fetch_one(&db.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn insert_vote_ignores_duplicate_vote(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend(&db, 2024, 1).await;
        db.upsert_session(
            weekend_id,
            SessionType::Race,
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        let session_id: i64 = sqlx::query_scalar!("SELECT id FROM session")
            .fetch_one(&db.db)
            .await
            .unwrap();

        db.insert_vote("user-1", session_id, VoteType::Highlights)
            .await
            .unwrap();
        db.insert_vote("user-1", session_id, VoteType::FullRace)
            .await
            .unwrap();

        let count: i64 = sqlx::query_scalar!("SELECT count(*) FROM votes")
            .fetch_one(&db.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(count, 1);
    }

    #[sqlx::test]
    async fn find_last_weekend_returns_none_when_empty(pool: PgPool) {
        let db = db(pool);
        assert!(db.find_last_weekend(None).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn find_last_weekend_returns_none_when_no_weekend_has_elapsed(pool: PgPool) {
        let db = db(pool);
        seed_weekend_with_date(&db, 2099, 1, date(2099, 9, 1)).await;

        assert!(db.find_last_weekend(None).await.unwrap().is_none());
    }

    #[sqlx::test]
    async fn find_last_weekend_ignores_future_weekends(pool: PgPool) {
        let db = db(pool);
        let past_id = seed_weekend_with_date(&db, 2020, 1, date(2020, 9, 1)).await;
        seed_weekend_with_date(&db, 2099, 1, date(2099, 9, 1)).await;

        let weekend = db.find_last_weekend(None).await.unwrap().unwrap();
        assert_eq!(weekend.id, past_id);
    }

    #[sqlx::test]
    async fn find_last_weekend_returns_the_most_recent_elapsed_weekend(pool: PgPool) {
        let db = db(pool);
        seed_weekend_with_date(&db, 2020, 1, date(2020, 9, 1)).await;
        let recent_id = seed_weekend_with_date(&db, 2024, 1, date(2024, 9, 1)).await;

        let weekend = db.find_last_weekend(None).await.unwrap().unwrap();
        assert_eq!(weekend.id, recent_id);
        assert_eq!(weekend.year, 2024);
    }

    #[sqlx::test]
    async fn find_last_weekend_includes_weekend_without_sessions(pool: PgPool) {
        let db = db(pool);
        let id = seed_weekend_with_date(&db, 2024, 1, date(2024, 9, 1)).await;

        let weekend = db.find_last_weekend(None).await.unwrap().unwrap();
        assert_eq!(weekend.id, id);
        assert!(weekend.sessions.is_empty());
    }

    #[sqlx::test]
    async fn find_last_weekend_includes_all_sessions_and_votes(pool: PgPool) {
        let db = db(pool);
        let weekend_id = seed_weekend_with_date(&db, 2024, 1, date(2024, 9, 1)).await;
        db.upsert_session(
            weekend_id,
            SessionType::Qualifying,
            jiff_sqlx::Timestamp::from("2024-09-01T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        db.upsert_session(
            weekend_id,
            SessionType::Race,
            jiff_sqlx::Timestamp::from("2024-09-02T13:00:00Z".parse::<JiffTimestamp>().unwrap()),
        )
        .await
        .unwrap();
        let session_ids: Vec<i64> =
            sqlx::query_scalar!("SELECT id FROM session ORDER BY start_time ASC")
                .fetch_all(&db.db)
                .await
                .unwrap();
        assert_eq!(session_ids.len(), 2);
        let (qualifying_id, race_id) = (session_ids[0], session_ids[1]);

        db.insert_vote("u1", race_id, VoteType::FullRace)
            .await
            .unwrap();
        db.insert_vote("u2", race_id, VoteType::FullRace)
            .await
            .unwrap();
        db.insert_vote("u3", race_id, VoteType::Highlights)
            .await
            .unwrap();

        let weekend = db.find_last_weekend(None).await.unwrap().unwrap();
        assert_eq!(weekend.id, weekend_id);
        assert_eq!(weekend.sessions.len(), 2);
        assert_eq!(weekend.sessions[0].id, qualifying_id);
        assert_eq!(weekend.sessions[0].votes.full_race, 0);
        assert_eq!(weekend.sessions[1].id, race_id);
        assert_eq!(weekend.sessions[1].votes.full_race, 2);
        assert_eq!(weekend.sessions[1].votes.highlights, 1);
    }
}
