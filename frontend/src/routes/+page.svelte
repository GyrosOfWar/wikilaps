<script lang="ts">
  import { resolve } from "$app/paths";
  import { submitVote } from "$lib/client.js";
  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";

  const { data } = $props();
</script>

<section class="max-w-2xl mt-4 self-center">
  <h1 class="h2 mb-4">Community-driven race votes for Formula 1 races.</h1>
  <p>
    Inspired by the excellent <a
      target="_blank"
      class="text-primary-500 underline hover:text-primary-400-600"
      href="https://wikihoops.com">wikihoops.com</a
    >.
  </p>
  <a href={resolve("/weekends/2026")} class="btn preset-filled-primary-500 mt-4 px-4 py-3"
    >View 2026 Weekends <ArrowRight />
  </a>
</section>

{#if data.weekend}
  <section
    class="bg-white dark:bg-surface-900 max-w-2xl mt-6 self-center w-full p-4 rounded-lg shadow-xl"
  >
    <h2 class="h4 text-primary-500">Latest session</h2>
    <RaceWeekendCard weekend={data.weekend} votes={[]} onSubmitVote={submitVote} />
  </section>
{/if}
