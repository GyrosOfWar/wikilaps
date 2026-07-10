import * as api from "$lib/api.js";

export const load = async ({ fetch, params }) => {
  const { year: yearString } = params;
  const year = parseInt(yearString, 10);
  const weekends = await api.listWeekends(year, { fetch });
  const votes = await api.listUserVotes({ fetch });

  return { weekends: weekends.data, votes: votes.data, year };
};
