/**
 * wikilaps
 * 0.1.0
 * DO NOT MODIFY - This file has been generated using oazapfts.
 * See https://www.npmjs.com/package/oazapfts
 */
import * as Oazapfts from "@oazapfts/runtime";
import * as QS from "@oazapfts/runtime/query";
export const defaults: Oazapfts.Defaults<Oazapfts.CustomHeaders> = {
  headers: {},
  baseUrl: "/",
};
const oazapfts = Oazapfts.runtime(defaults);
export const servers = {};
export type SessionType = "sprint_qualifying" | "sprint_race" | "qualifying" | "race";
export type String = string;
export type VoteCounts = {
  full: number;
  highlights: number;
  raceIn30?: number | null;
};
export type PageSessionListResponse = {
  content: {
    countryKey: string;
    grandPrixId: string;
    id: number;
    raceWeekendStartDate: String;
    round: number;
    sessionStartTime: String;
    sessionType: SessionType;
    votes: VoteCounts;
  }[];
  pageNumber: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
};
export type VoteType = "FullRace" | "RaceIn30" | "Highlights";
export type SessionResponse = {
  endTime?: null | String;
  id: number;
  sessionType: SessionType;
  startTime: String;
  userVote?: null | VoteType;
  votes: VoteCounts;
  votingAllowed: boolean;
};
export type RaceWeekendResponse = {
  circuitFullName: string;
  countryKey: string;
  grandPrixId: string;
  id: number;
  location: string;
  officialName: string;
  round: number;
  sessions: SessionResponse[];
  startDate: String;
  /** True while none of the weekend's sessions have started yet — the UI
    renders such weekends as disabled. Computed server-side so it stays
    consistent with each session's `voting_allowed`. */
  upcoming: boolean;
  year: number;
};
export type VoteRequest = {
  sessionId: number;
  vote: VoteType;
};
export function listSessions(
  {
    page,
    size,
    sort,
    year,
    $type,
  }: {
    page?: number;
    size?: number;
    sort?: string;
    year?: number;
    $type?: SessionType;
  } = {},
  opts?: Oazapfts.RequestOpts,
) {
  return oazapfts.fetchJson<{
    status: 200;
    data: PageSessionListResponse;
  }>(
    `/api/race/sessions${QS.query(
      QS.explode({
        page,
        size,
        sort,
        year,
        type: $type,
      }),
    )}`,
    {
      ...opts,
    },
  );
}
export function getLatestWeekend(opts?: Oazapfts.RequestOpts) {
  return oazapfts.fetchJson<{
    status: 200;
    data: null | RaceWeekendResponse;
  }>("/api/race/weekends/latest", {
    ...opts,
  });
}
export function listWeekends(year: number, opts?: Oazapfts.RequestOpts) {
  return oazapfts.fetchJson<{
    status: 200;
    data: RaceWeekendResponse[];
  }>(`/api/race/weekends/${encodeURIComponent(year)}`, {
    ...opts,
  });
}
export function getYearsOfData(opts?: Oazapfts.RequestOpts) {
  return oazapfts.fetchJson<{
    status: 200;
    data: number[];
  }>("/api/race/years", {
    ...opts,
  });
}
/**
 * Called by the frontend when the user opens the site. Issues a signed
 * identity cookie if the browser doesn't already have a valid one, and is a
 * no-op (keeping the existing identity) otherwise.
 */
export function initSession(opts?: Oazapfts.RequestOpts) {
  return oazapfts.fetchText("/api/session", {
    ...opts,
  });
}
/**
 * Cast a vote for a session on behalf of the browser identified by the signed
 * cookie. The `(user_identifier, session_id)` unique constraint means a
 * browser's first vote for a session wins; subsequent votes are ignored.
 */
export function createVote(voteRequest: VoteRequest, opts?: Oazapfts.RequestOpts) {
  return oazapfts.fetchText(
    "/api/vote",
    oazapfts.json({
      ...opts,
      method: "POST",
      body: voteRequest,
    }),
  );
}
