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

  <h3 class="text-xs font-bold tracking-wide uppercase opacity-70 mt-4 mb-1">Jump to</h3>
  <ul class="flex gap-2 w-full flex-wrap">
    {#each data.weekends as weekend (weekend.id)}
      <li>
        <a
          href={`#round-${weekend.round}`}
          class="badge preset-filled-secondary-500 shrink-0 font-bold"
        >
          <span class="fi fi-{weekend.countryKey.toLowerCase()} shrink-0 rounded-sm shadow-sm"
          ></span>
          R{weekend.round}
        </a>
      </li>
    {/each}
  </ul>

  <section class="grid gap-6 grid-cols-1 mt-8 w-full">
    {#each data.weekends as weekend (weekend.id)}
      <RaceWeekendCard
        {weekend}
        class="border border-surface-200-800 shadow-xl bg-tertiary-50-950"
        onSubmitVote={submitVote}
      />
    {/each}
  </section>
</div>
