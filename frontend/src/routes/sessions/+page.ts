import { getYearsOfData, listSessions } from "$lib/api.js";

export const load = async ({ fetch, url }) => {
  const page = parseInt(url.searchParams.get("page") ?? "0");
  const [sessions, years] = await Promise.all([
    listSessions(page, 20, null, null, "race", { fetch }),
    getYearsOfData({ fetch }),
  ]);

  return {
    allYears: years.data,
    sessions: sessions.data,
  };
};
