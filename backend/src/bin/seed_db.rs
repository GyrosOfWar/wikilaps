//! Seeds `race_weekend`/`session` for a given season from f1db
//! (https://github.com/f1db/f1db), a community-maintained, MIT/CC-licensed
//! F1 dataset. Fetches the latest `f1db-json-splitted.zip` release asset at
//! run time — nothing is vendored.
//!
//! Usage (from `backend/`): `cargo run --features seed --bin seed_db [year]`
//! (defaults to the current year).

use clap::Parser;
use jiff::{
    Zoned,
    civil::{Date, Time},
};
use jiff_sqlx::ToSqlx;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};
use tracing::info;
use wikilaps::{
    config::AppConfig,
    database::{Database, SessionType, VoteType},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const F1DB_REPO: &str = "f1db/f1db";
const F1DB_ASSET_NAME: &str = "f1db-json-splitted.zip";

mod f1db {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Race {
        pub year: i16,
        pub round: i32,
        pub date: String,
        pub time: Option<String>,
        pub official_name: String,
        pub circuit_id: String,
        pub grand_prix_id: String,
        pub qualifying_date: Option<String>,
        pub qualifying_time: Option<String>,
        pub sprint_qualifying_date: Option<String>,
        pub sprint_qualifying_time: Option<String>,
        pub sprint_race_date: Option<String>,
        pub sprint_race_time: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Circuit {
        pub id: String,
        pub place_name: String,
        pub full_name: String,
        pub country_id: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Country {
        pub id: String,
        pub alpha2_code: String,
    }
}

fn race_sessions(race: &f1db::Race) -> Vec<(SessionType, Option<&str>, Option<&str>)> {
    vec![
        (
            SessionType::SprintQualifying,
            race.sprint_qualifying_date.as_deref(),
            race.sprint_qualifying_time.as_deref(),
        ),
        (
            SessionType::SprintRace,
            race.sprint_race_date.as_deref(),
            race.sprint_race_time.as_deref(),
        ),
        (
            SessionType::Qualifying,
            race.qualifying_date.as_deref(),
            race.qualifying_time.as_deref(),
        ),
        (
            SessionType::Race,
            Some(race.date.as_str()),
            race.time.as_deref(),
        ),
    ]
}

/// f1db session/race times are given in UTC.
fn parse_utc_timestamp(date: &str, time: &str) -> Result<jiff::Timestamp> {
    let date: Date = date.parse()?;
    let time: Time = time.parse()?;
    let zoned = date.to_datetime(time).to_zoned(jiff::tz::TimeZone::UTC)?;
    Ok(zoned.timestamp())
}

async fn download_f1db_snapshot() -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent("wikilaps-seeder")
        .build()?;

    let release: serde_json::Value = client
        .get(format!(
            "https://api.github.com/repos/{F1DB_REPO}/releases/latest"
        ))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let asset_url = release["assets"]
        .as_array()
        .and_then(|assets| assets.iter().find(|asset| asset["name"] == F1DB_ASSET_NAME))
        .and_then(|asset| asset["browser_download_url"].as_str())
        .ok_or("f1db latest release is missing the f1db-json-splitted.zip asset")?;

    tracing::info!("Downloading {F1DB_ASSET_NAME} from {asset_url}");

    let bytes = client
        .get(asset_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    Ok(bytes.to_vec())
}

fn read_json<T: serde::de::DeserializeOwned>(
    archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    name: &str,
) -> Result<Vec<T>> {
    let mut file = archive.by_name(name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(serde_json::from_str(&contents)?)
}

#[derive(Parser)]
enum CliArgs {
    /// Batch-import data for an entire season from f1db's latest data release on GitHub
    F1Db { year: Option<i16> },
    /// Import the Discord poll data from the supplied path to the YAML file
    Poll { path: PathBuf },
}

#[derive(Deserialize)]
struct DiscordPollData {
    polls: HashMap<String, HashMap<String, HashMap<VoteType, i32>>>,
}

async fn import_f1db(db: &Database, year: i16) -> Result<()> {
    tracing::info!("Seeding season {year} from f1db");

    let zip_bytes = download_f1db_snapshot().await?;
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes))?;

    let races: Vec<f1db::Race> = read_json(&mut archive, "f1db-races.json")?;
    let circuits: Vec<f1db::Circuit> = read_json(&mut archive, "f1db-circuits.json")?;
    let countries: Vec<f1db::Country> = read_json(&mut archive, "f1db-countries.json")?;

    let circuits_by_id: HashMap<&str, &f1db::Circuit> =
        circuits.iter().map(|c| (c.id.as_str(), c)).collect();
    let country_codes: HashMap<&str, &str> = countries
        .iter()
        .map(|c| (c.id.as_str(), c.alpha2_code.as_str()))
        .collect();

    let mut seeded_weekends = 0;
    let mut seeded_sessions = 0;

    for race in races.iter().filter(|r| r.year == year) {
        let Some(circuit) = circuits_by_id.get(race.circuit_id.as_str()) else {
            tracing::warn!(
                "Skipping round {} ({}): unknown circuit id {:?}",
                race.round,
                race.year,
                race.circuit_id
            );
            continue;
        };
        let Some(country_key) = country_codes.get(circuit.country_id.as_str()) else {
            tracing::warn!(
                "Skipping round {} ({}): unknown country id {:?}",
                race.round,
                race.year,
                circuit.country_id
            );
            continue;
        };

        let sessions = race_sessions(race);
        let start_date_str = sessions
            .iter()
            .filter_map(|(_, date, _)| *date)
            .min()
            .unwrap_or(race.date.as_str());
        let start_date: Date = start_date_str.parse()?;

        let weekend_id = db
            .upsert_race_weekend(
                race.year.into(),
                race.round,
                &circuit.place_name,
                &circuit.id,
                &circuit.full_name,
                &race.grand_prix_id,
                country_key,
                &race.official_name,
                start_date.to_sqlx(),
            )
            .await?;
        seeded_weekends += 1;

        for (session_type, date, time) in sessions {
            let (Some(date), Some(time)) = (date, time) else {
                continue;
            };
            let start_time = parse_utc_timestamp(date, time)?;
            db.upsert_session(weekend_id, session_type, start_time.to_sqlx())
                .await?;
            seeded_sessions += 1;
        }
    }

    tracing::info!("Seeded {seeded_weekends} weekend(s), {seeded_sessions} session(s) for {year}");

    Ok(())
}

async fn import_poll_results(db: &Database, path: &PathBuf) -> Result<()> {
    let data: DiscordPollData = serde_norway::from_reader(File::open(path)?)?;

    let mut tx = db.pool().begin().await?;

    for (year, results) in data.polls {
        let year: i32 = year.parse().unwrap();
        info!("Inserting poll results for {year}");
        for (grand_prix_id, poll_result) in results {
            for (vote_type, count) in poll_result {
                info!(
                    "Inserting {count} votes for the race at {grand_prix_id} with type {vote_type:?}"
                );
                sqlx::query!(
                    "WITH session_info AS (
                        SELECT s.id AS session_id 
                        FROM session s
                        JOIN race_weekend rw
                        ON s.weekend_id = rw.id AND rw.grand_prix_id = $1 AND rw.year = $2
                        WHERE s.session_type = 'race'
                    )
                    INSERT INTO votes (user_identifier, vote_type, session_id)
                    SELECT gen_random_uuid()::varchar, $3, s.session_id
                    FROM session_info s
                    CROSS JOIN LATERAL generate_series(1, $4)",
                    grand_prix_id,
                    year,
                    vote_type as _,
                    count,
                )
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let args = CliArgs::parse();

    let config = AppConfig::default();
    let db = Database::new(&config.database_url).await?;

    match args {
        CliArgs::F1Db { year } => {
            import_f1db(&db, year.unwrap_or_else(|| Zoned::now().year())).await?
        }
        CliArgs::Poll { path } => import_poll_results(&db, &path).await?,
    }

    Ok(())
}
