import * as api from "$lib/api.js";

export const load = async ({ fetch }) => {
  const weekends = await api.listWeekends(2026, { fetch });
  const votes = await api.listUserVotes({ fetch });

  return { weekends: weekends.data, votes: votes.data };
};
