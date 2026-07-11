use jiff::{Span, Timestamp};

use crate::database::SessionType;

pub fn voting_allowed(start_time: Timestamp, session_type: SessionType) -> bool {
    // how long after the session start date should votes be allowed? (in minutes)
    let session_voting_start = match session_type {
        SessionType::SprintQualifying => 30,
        SessionType::SprintRace => 40,
        SessionType::Qualifying => 50,
        SessionType::Race => 80,
    };

    let estimated_end = start_time + Span::new().minutes(session_voting_start);

    let distance = Timestamp::now()
        .until(estimated_end)
        .expect("time overflow");

    // negative distance means the date was in the past
    distance.is_negative()
}
