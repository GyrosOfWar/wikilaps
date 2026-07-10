<script lang="ts">
  import { createVote } from "$lib/api.js";
  import { invalidateAll } from "$app/navigation";
  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";

  const { data } = $props();

  type VoteType = "FullRace" | "RaceIn30" | "Highlights";

  async function submitVote(sessionId: number, vote: VoteType) {
    const response = await createVote({ sessionId, vote });
    if (response.status !== 201) {
      console.error("Failed to submit vote", response);
      return;
    }
    await invalidateAll();
  }
</script>

<svelte:head>
  <title>wikilaps</title>
</svelte:head>

<section class="grid gap-4 grid-cols-1 mt-6 max-w-3xl self-center w-full">
  {#each data.weekends as weekend (weekend.id)}
    <RaceWeekendCard {weekend} votes={data.votes} year={data.year} onSubmitVote={submitVote} />
  {/each}
</section>
