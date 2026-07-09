use crate::error::Result;

use sqlx::PgPool;

#[derive(Debug, Clone, Copy, sqlx::Type, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "session_type", rename_all = "snake_case")]
pub enum SessionType {
    FreePractice,
    SprintQualifying,
    SprintRace,
    Qualifying,
    Race,
}

#[derive(Debug, Clone, Copy, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "vote_type", rename_all = "PascalCase")]
pub enum VoteType {
    FullRace,
    RaceIn30,
    Highlights,
}

#[derive(Debug)]
pub struct RaceWeekend {
    pub id: i64,
    pub year: i32,
    pub location: String,
    pub official_name: String,
    pub circuit_name: String,
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
    pub async fn list_weekends(&self, year: i32) -> Result<Vec<RaceWeekend>> {
        let rows = sqlx::query!(
            r#"SELECT
                    r.id AS weekend_id, r.year, r.location, r.circuit_name, r.country_key,
                    r.start_date AS "start_date: jiff_sqlx::Date", r.round, r.official_name,
                    s.id AS "session_id?",
                    s.session_type AS "session_type?: SessionType",
                    s.start_time AS "start_time?: jiff_sqlx::Timestamp",
                    s.end_time AS "end_time?: jiff_sqlx::Timestamp",
                    count(v.id) FILTER (WHERE v.vote_type = 'FullRace'::vote_type)   AS "full_race!",
                    count(v.id) FILTER (WHERE v.vote_type = 'RaceIn30'::vote_type)   AS "race_in_30!",
                    count(v.id) FILTER (WHERE v.vote_type = 'Highlights'::vote_type) AS "highlights!"
                FROM race_weekend r
                LEFT JOIN session s ON s.weekend_id = r.id
                LEFT JOIN votes v ON v.session_id = s.id
                WHERE r.year = $1
                GROUP BY r.id, s.id
                ORDER BY r.start_date ASC, r.id ASC, s.start_time ASC NULLS FIRST"#,
            year
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
                    circuit_name: row.circuit_name,
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
        circuit_name: &str,
        country_key: &str,
        official_name: &str,
        start_date: jiff_sqlx::Date,
    ) -> Result<i64> {
        let id = sqlx::query_scalar!(
            r#"INSERT INTO race_weekend (year, round, location, circuit_name, country_key, start_date, official_name)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (year, round) DO UPDATE SET
                    location = EXCLUDED.location,
                    circuit_name = EXCLUDED.circuit_name,
                    country_key = EXCLUDED.country_key,
                    start_date = EXCLUDED.start_date
                RETURNING id"#,
            year,
            round,
            location,
            circuit_name,
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

    pub async fn insert_vote(
        &self,
        user_id: &str,
        session_id: i64,
        vote_type: VoteType,
    ) -> Result<()> {
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
        db.upsert_race_weekend(
            year,
            round,
            "Monza",
            "Autodromo Nazionale Monza",
            "ITA",
            "Formula 1 AWS Gran Premio del Made in Italy e dell'Emilia Romagna 2025",
            jiff_sqlx::Date::from(date(year as i16, 9, round as i8)),
        )
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn list_weekends_empty(pool: PgPool) {
        let db = db(pool);
        let weekends = db.list_weekends(2024).await.unwrap();
        assert!(weekends.is_empty());
    }

    #[sqlx::test]
    async fn list_weekends_filters_by_year(pool: PgPool) {
        let db = db(pool);
        seed_weekend(&db, 2023, 1).await;
        seed_weekend(&db, 2024, 1).await;

        let weekends = db.list_weekends(2024).await.unwrap();
        assert_eq!(weekends.len(), 1);
        assert_eq!(weekends[0].year, 2024);
    }

    #[sqlx::test]
    async fn list_weekends_orders_by_start_date_asc(pool: PgPool) {
        let db = db(pool);
        seed_weekend(&db, 2024, 1).await;
        seed_weekend(&db, 2024, 2).await;

        let weekends = db.list_weekends(2024).await.unwrap();
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

        let weekends = db.list_weekends(2024).await.unwrap();
        assert_eq!(weekends.len(), 1);
        let sessions = &weekends[0].sessions;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        assert_eq!(sessions[0].votes.full_race, 2);
        assert_eq!(sessions[0].votes.race_in_30, 0);
        assert_eq!(sessions[0].votes.highlights, 1);
    }

    #[sqlx::test]
    async fn list_weekends_includes_weekend_without_sessions(pool: PgPool) {
        let db = db(pool);
        seed_weekend(&db, 2024, 1).await;

        let weekends = db.list_weekends(2024).await.unwrap();
        assert_eq!(weekends.len(), 1);
        assert!(weekends[0].sessions.is_empty());
    }

    #[sqlx::test]
    async fn upsert_race_weekend_inserts_new_row(pool: PgPool) {
        let db = db(pool);
        let id = seed_weekend(&db, 2024, 1).await;

        let weekends = db.list_weekends(2024).await.unwrap();
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
                "Autodromo Enzo e Dino Ferrari",
                "ITA",
                "Formula 1 Made in Italy e dell'Emilia Romagna 2024",
                jiff_sqlx::Date::from(date(2024, 9, 1)),
            )
            .await
            .unwrap();

        assert_eq!(id, updated_id);

        let weekends = db.list_weekends(2024).await.unwrap();
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
}
