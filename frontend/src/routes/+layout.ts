import { initSession } from "$lib/api";

export const load = async ({ fetch }) => {
  await initSession({ fetch });
};
