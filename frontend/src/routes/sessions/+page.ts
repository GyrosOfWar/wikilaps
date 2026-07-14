import { getYearsOfData, listSessions } from "$lib/api.js";

export const load = async ({ fetch, url }) => {
  const page = parseInt(url.searchParams.get("page") ?? "0", 10);
  const sort = url.searchParams.get("sort");
  const year = parseInt(url.searchParams.get("year") || "", 10) ?? null;
  const [sessions, years] = await Promise.all([
    listSessions(page, 20, sort, year, "race", { fetch }),
    getYearsOfData({ fetch }),
  ]);

  return {
    allYears: years.data,
    sessions: sessions.data,
  };
};
