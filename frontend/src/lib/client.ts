import { invalidateAll } from "$app/navigation";
import { createVote, type VoteType } from "./api";

export async function submitVote(sessionId: number, vote: VoteType) {
  const response = await createVote({ sessionId, vote });
  if (response.status !== 201) {
    // TODO show a toast or something to the user
    console.error("Failed to submit vote", response);
    return;
  }
  await invalidateAll();
}
