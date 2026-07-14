import type { RaceWeekendResponse, SessionResponse, SessionType, VoteCounts } from "./api";

const emptyVotes: VoteCounts = { full: 0, highlights: 0, raceIn30: 0 };

export function makeSession(overrides: Partial<SessionResponse> = {}): SessionResponse {
  return {
    id: 1,
    sessionType: "race",
    startTime: "2026-03-15T15:00:00Z",
    endTime: "2026-03-15T17:00:00Z",
    votes: emptyVotes,
    userVote: null,
    votingAllowed: false,
    ...overrides,
  };
}

export function makeWeekend(overrides: Partial<RaceWeekendResponse> = {}): RaceWeekendResponse {
  return {
    id: 1,
    year: 2026,
    location: "Melbourne",
    circuitFullName: "Albert Park Grand Prix Circuit",
    grandPrixId: "australia",
    countryKey: "au",
    startDate: "2026-03-13",
    round: 1,
    officialName: "Australian Grand Prix",
    sessions: [],
    upcoming: false,
    ...overrides,
  };
}

const raceSessionTypes: SessionType[] = ["qualifying", "race"];

/**
 * A weekend that has been scheduled but whose first session hasn't started yet:
 * `upcoming: true`, no votes, nothing votable. The UI renders it as disabled.
 */
export const upcomingWeekend: RaceWeekendResponse = makeWeekend({
  id: 24,
  round: 2,
  grandPrixId: "china",
  countryKey: "cn",
  location: "Shanghai",
  officialName: "Chinese Grand Prix",
  startDate: "2026-03-20",
  upcoming: true,
  sessions: raceSessionTypes.map((sessionType, i) =>
    makeSession({
      id: 100 + i,
      sessionType,
      startTime: "2026-03-20T07:00:00Z",
      endTime: null,
      votingAllowed: false,
    }),
  ),
});

/**
 * A weekend whose race has finished and is open for voting, with some votes
 * already cast. Used for the "first votable session finished" scenario.
 */
export const finishedRaceWeekend: RaceWeekendResponse = makeWeekend({
  id: 1,
  round: 1,
  grandPrixId: "australia",
  countryKey: "au",
  startDate: "2026-03-13",
  upcoming: false,
  sessions: [
    makeSession({
      id: 10,
      sessionType: "race",
      startTime: "2026-03-15T04:00:00Z",
      endTime: "2026-03-15T06:00:00Z",
      votingAllowed: true,
      userVote: null,
      votes: { full: 7, raceIn30: 3, highlights: 2 },
    }),
  ],
});

/**
 * The previous GP's race, as returned by `/weekends/latest` while a newer
 * weekend is scheduled but hasn't started. Shown on the landing page.
 */
export const previousRaceWeekend: RaceWeekendResponse = makeWeekend({
  id: 1,
  round: 1,
  grandPrixId: "australia",
  countryKey: "au",
  startDate: "2026-03-13",
  upcoming: false,
  sessions: [
    makeSession({
      id: 10,
      sessionType: "race",
      startTime: "2026-03-15T04:00:00Z",
      endTime: "2026-03-15T06:00:00Z",
      votingAllowed: true,
      votes: { full: 12, raceIn30: 5, highlights: 4 },
    }),
  ],
});
