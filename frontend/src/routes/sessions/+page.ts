import { getYearsOfData, listSessions, type SessionType } from "$lib/api.js";
import { optionalIntegerParameter, requiredIntegerParameter } from "$lib/url.js";

export const load = async ({ fetch, url }) => {
  const p = url.searchParams;
  const page = requiredIntegerParameter(p, "page", 1);
  const sort = url.searchParams.get("sort") ?? "start_date";
  const year = optionalIntegerParameter(p, "year");
  const $type = (url.searchParams.get("type") ?? undefined) as SessionType | undefined;
  const [sessions, years] = await Promise.all([
    listSessions({ page, size: 20, sort, year, $type }, { fetch }),
    getYearsOfData({ fetch }),
  ]);

  return {
    allYears: years.data,
    sessions: sessions.data,
    page,
    sort,
    year,
    type: $type,
  };
};
