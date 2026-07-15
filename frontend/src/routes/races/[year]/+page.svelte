<script lang="ts">
  import { resolve } from "$app/paths";
  import { submitVote } from "$lib/client.js";
  import JumpToTopButton from "$lib/components/JumpToTopButton.svelte";
  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";
  import Seo from "$lib/components/Seo.svelte";
  import * as m from "$lib/paraglide/messages";

  const { data } = $props();
</script>

<Seo
  title={m.weekend_page_heading({ year: data.year })}
  description={m.seo_races_description({ year: data.year })}
/>

<JumpToTopButton />

<div class="max-w-3xl w-full self-center">
  <h1 class="h2 font-bold tracking-tight">
    {m.weekend_page_heading({ year: data.year })}
  </h1>

  <ul class="flex gap-2 w-full flex-wrap">
    {#each data.allYears as year (year)}
      {@const isCurrentYear = year === data.year}
      <li
        class={[
          "badge font-bold preset-filled-primary-500",
          isCurrentYear && "opacity-70 cursor-not-allowed",
        ]}
      >
        <a
          aria-disabled={isCurrentYear}
          href={isCurrentYear ? undefined : resolve(`/races/${year}`)}
        >
          {year}
        </a>
      </li>
    {/each}
  </ul>

  <h3 class="text-xs font-bold tracking-wide uppercase opacity-70 mt-4 mb-1">
    {m.session_selector_label()}
  </h3>
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
        onSubmitVote={submitVote}
        class="border border-surface-200-800 shadow-xl bg-tertiary-50 dark:bg-surface-800"
      />
    {/each}
  </section>
</div>
