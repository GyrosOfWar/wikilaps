<script lang="ts">
  import { submitVote } from "$lib/client.js";
  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";

  const { data } = $props();
</script>

<section class="grid gap-4 grid-cols-1 mt-6 max-w-3xl self-center w-full">
  {#each data.weekends as weekend (weekend.id)}
    <RaceWeekendCard {weekend} votes={data.votes} year={data.year} onSubmitVote={submitVote} />
  {/each}
</section>
