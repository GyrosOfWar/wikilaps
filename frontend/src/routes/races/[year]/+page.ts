import * as api from "$lib/api.js";

export const load = async ({ fetch, params }) => {
  const { year: yearString } = params;
  const year = parseInt(yearString, 10);
  const [weekends, years] = await Promise.all([
    api.listWeekends(year, { fetch }),
    api.getYearsOfData({ fetch }),
  ]);

  return { weekends: weekends.data, year, allYears: years.data };
};
