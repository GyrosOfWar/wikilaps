// TODO replace with generated types via OpenAPI spec

export interface VoteCounts {
  fullRace: number;
  raceIn30: number;
  highlights: number;
}

export type SessionType =
  "FREE_PRACTICE" | "SPRINT_QUALIFYING" | "SPRINT_RACE" | "QUALIFYING" | "RACE";

export interface SessionResponse {
  id: number;
  sessionType: SessionType;
  startTime: string;
  endTime: string | null;
  votes: VoteCounts;
}

export interface RaceWeekendResponse {
  id: number;
  year: number;
  location: string;
  circuitName: string;
  countryKey: string;
  startDate: string;
  round: number;
  officialName: string;
  sessions: SessionResponse[];
}
