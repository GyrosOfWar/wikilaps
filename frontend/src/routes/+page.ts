import { getLatestWeekend } from "$lib/api";

export const load = async ({ fetch }) => {
  const response = await getLatestWeekend({ fetch });
  return { weekend: response.data };
};
