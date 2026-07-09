import type { RaceWeekendResponse } from "$lib/types.js";

export const load = async ({ fetch }) => {
  const response = await fetch("/api/race-weekends/2026");
  const json: RaceWeekendResponse[] = await response.json();
  return { weekends: json };
};
