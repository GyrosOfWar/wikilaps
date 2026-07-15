import * as m from "$lib/paraglide/messages";
import type { SessionType, VoteCounts, VoteType } from "$lib/api";

export interface VoteOption {
  value: VoteType;
  label: string;
  count: number;
}

export function voteOptions(votes: VoteCounts, sessionType: SessionType): VoteOption[] {
  if (sessionType === "race") {
    return [
      { value: "FullRace", label: m.vote_type_full_race(), count: votes.full },
      { value: "RaceIn30", label: m.vote_type_race_in_30(), count: votes.raceIn30! },
      { value: "Highlights", label: m.vote_type_highlights(), count: votes.highlights },
    ];
  } else if (sessionType === "sprint_race") {
    return [
      { value: "FullRace", label: m.vote_type_full_race(), count: votes.full },
      { value: "Highlights", label: m.vote_type_highlights(), count: votes.highlights },
    ];
  } else {
    return [
      { value: "FullRace", label: m.vote_type_full_session(), count: votes.full },
      { value: "Highlights", label: m.vote_type_highlights(), count: votes.highlights },
    ];
  }
}

export interface VoteSummary {
  total: number;
  winner: VoteOption | null;
  percent: number;
}

export function summarizeVotes(votes: VoteCounts, sessionType: SessionType): VoteSummary {
  const options = voteOptions(votes, sessionType);
  const total = options.reduce((sum, o) => sum + o.count, 0);
  if (total === 0) {
    return { total, winner: null, percent: 0 };
  }

  const winner = options.reduce((best, o) => (o.count > best.count ? o : best));
  return { total, winner, percent: Math.round((winner.count / total) * 100) };
}
