<script lang="ts">
  import { submitVote } from "$lib/client.js";
  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";
  import * as m from "$lib/paraglide/messages";

  const { data } = $props();
</script>

<div class="max-w-3xl w-full self-center">
  <h1 class="h2 font-bold tracking-tight">
    {m.weekend_page_heading({ year: data.year })}
  </h1>
</div>

<section class="grid gap-6 grid-cols-1 mt-8 max-w-3xl self-center w-full">
  {#each data.weekends as weekend (weekend.id)}
    <RaceWeekendCard
      {weekend}
      class="preset-filled-surface-50-950 border border-surface-200-800 shadow-xl"
      votes={data.votes}
      onSubmitVote={submitVote}
    />
  {/each}
</section>
