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

#[cfg(test)]
mod test {
    #![allow(clippy::bool_assert_comparison)]
    use super::*;

    #[test]
    fn voting_allowed_race() {
        let start_time = Timestamp::now() + Span::new().minutes(80);
        assert_eq!(false, voting_allowed(start_time, SessionType::Race));

        let start_time = Timestamp::now() + Span::new().minutes(30);
        assert_eq!(false, voting_allowed(start_time, SessionType::Race));

        let start_time = Timestamp::now() + Span::new().minutes(-30);
        assert_eq!(false, voting_allowed(start_time, SessionType::Race));

        let start_time = Timestamp::now() + Span::new().minutes(-81);
        assert_eq!(true, voting_allowed(start_time, SessionType::Race));
    }

    #[test]
    fn voting_allowed_sprint_qual() {
        let start_time = Timestamp::now() + Span::new().minutes(80);
        assert_eq!(
            false,
            voting_allowed(start_time, SessionType::SprintQualifying)
        );

        let start_time = Timestamp::now() + Span::new().minutes(30);
        assert_eq!(
            false,
            voting_allowed(start_time, SessionType::SprintQualifying)
        );

        let start_time = Timestamp::now() + Span::new().minutes(-20);
        assert_eq!(
            false,
            voting_allowed(start_time, SessionType::SprintQualifying)
        );

        let start_time = Timestamp::now() + Span::new().minutes(-81);
        assert_eq!(
            true,
            voting_allowed(start_time, SessionType::SprintQualifying)
        );
    }
}
