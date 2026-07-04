use crate::error::Result;

use sqlx::PgPool;

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "session_type", rename_all = "PascalCase")]
pub enum SessionType {
    FreePractice,
    SprintQualification,
    SprintRace,
    Qualifying,
    Race,
}

#[derive(Debug)]
pub struct RaceWeekend {
    pub id: i64,
    pub year: i32,
    pub location: String,
    pub circuit_name: String,
    pub country_key: String,
    pub start_date: jiff_sqlx::Date,
    pub round: i32,
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

    pub async fn list_weekends(&self) -> Result<Vec<RaceWeekend>> {
        let data = sqlx::query_as!(
            RaceWeekend,
            r#"SELECT id, year, location, circuit_name, country_key, 
                      start_date as "start_date: jiff_sqlx::Date", round
                FROM race_weekend
                ORDER BY start_date DESC"#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(data)
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
        start_date: jiff_sqlx::Date,
    ) -> Result<i64> {
        let id = sqlx::query_scalar!(
            r#"INSERT INTO race_weekend (year, round, location, circuit_name, country_key, start_date)
                VALUES ($1, $2, $3, $4, $5, $6)
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
}
