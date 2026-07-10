<script lang="ts">
  import { submitVote } from "$lib/client.js";
  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";

  const { data } = $props();
</script>

<h1 class="h3">
  Season {data.year}
</h1>

<section class="grid gap-4 grid-cols-1 mt-6 max-w-3xl self-center w-full">
  {#each data.weekends as weekend (weekend.id)}
    <RaceWeekendCard
      {weekend}
      class={["bg-white dark:bg-surface-800 shadow-xl"]}
      votes={data.votes}
      onSubmitVote={submitVote}
    />
  {/each}
</section>
