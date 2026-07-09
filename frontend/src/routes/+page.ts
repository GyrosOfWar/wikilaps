import * as api from "$lib/api.js";

export const load = async ({ fetch }) => {
  const weekends = await api.listWeekends(2026, { fetch });

  return { weekends: weekends.data };
};
