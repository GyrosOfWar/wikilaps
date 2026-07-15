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

pub fn weekend_upcoming(now: Timestamp, session_start_times: &[Timestamp]) -> bool {
    session_start_times.iter().all(|&start| start > now)
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

    #[test]
    fn weekend_upcoming_all_future() {
        let now = Timestamp::now();
        let starts = [now + Span::new().hours(3), now + Span::new().hours(5)];
        assert_eq!(true, weekend_upcoming(now, &starts));
    }

    #[test]
    fn weekend_upcoming_first_session_started() {
        let now = Timestamp::now();
        // FP1 three hours ago, remaining sessions still to come.
        let starts = [now - Span::new().hours(3), now + Span::new().hours(2)];
        assert_eq!(false, weekend_upcoming(now, &starts));
    }

    #[test]
    fn weekend_upcoming_all_elapsed() {
        let now = Timestamp::now();
        let starts = [now - Span::new().hours(5), now - Span::new().hours(3)];
        assert_eq!(false, weekend_upcoming(now, &starts));
    }
}
