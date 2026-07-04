use crate::error::Result;
use sqlx::PgPool;

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
}
