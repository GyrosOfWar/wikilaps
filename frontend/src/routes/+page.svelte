<script lang="ts">
  import { resolve } from "$app/paths";
  import { submitVote } from "$lib/client.js";
  import { ParaglideMessage } from "@inlang/paraglide-js-svelte";

  import RaceWeekendCard from "$lib/components/RaceWeekendCard.svelte";
  import Seo from "$lib/components/Seo.svelte";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import * as m from "$lib/paraglide/messages";

  const { data } = $props();
</script>

<Seo />

<section class="max-w-2xl w-full mt-10 self-center">
  <h1 class="h1 font-bold tracking-tight text-balance mb-4">
    {m.hero_heading()}
  </h1>
  <p class="text-lg opacity-70">
    <ParaglideMessage message={m.hero_sub_heading}>
      {#snippet link({ children, options })}
        <a
          class="text-primary-500 underline decoration-primary-500/40 underline-offset-2 hover:decoration-primary-500"
          target="_blank"
          href={options.to}
        >
          {@render children?.()}
        </a>
      {/snippet}
    </ParaglideMessage>
  </p>
  <a
    href={resolve("/races/2026")}
    class="btn preset-filled-primary-500 shadow-lg shadow-primary-500/25 mt-6 px-5 py-3"
  >
    {m.hero_button()}
    <ArrowRight class="size-5" />
  </a>
</section>

{#if data.weekend}
  <section class="max-w-2xl mt-14 self-center w-full">
    <p class="mb-3 text-xs font-bold uppercase tracking-widest text-primary-500">Latest session</p>
    <RaceWeekendCard
      weekend={data.weekend}
      onSubmitVote={submitVote}
      class="border border-surface-200-800 shadow-xl bg-tertiary-50 dark:bg-surface-800"
    />
  </section>
{/if}
