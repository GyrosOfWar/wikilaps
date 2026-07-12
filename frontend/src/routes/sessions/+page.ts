import { getYearsOfData, listSessions } from "$lib/api.js";

export const load = async ({ fetch }) => {
  const [sessions, years] = await Promise.all([
    listSessions(null, 20, null, null, null, { fetch }),
    getYearsOfData({ fetch }),
  ]);

  return {
    allYears: years.data,
    sessions: sessions.data,
  };
};
