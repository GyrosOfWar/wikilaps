<script lang="ts">
  import * as m from "$lib/paraglide/messages";
  import { Accordion } from "@skeletonlabs/skeleton-svelte";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import { slide } from "svelte/transition";
  import { sessionTypeLabel } from "$lib/i18n.js";
  import VoteResults from "$lib/components/VoteResults.svelte";
  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import { Temporal } from "temporal-polyfill";
  import { formatDate } from "$lib/date-time.js";
  import type { SessionResponse } from "$lib/api.js";
  import { invalidateAll } from "$app/navigation";

  const { data } = $props();

  type VoteType = "FullRace" | "RaceIn30" | "Highlights";

  async function submitVote(sessionId: number, vote: VoteType) {
    await fetch("/api/vote", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionId, vote }),
    });
    await invalidateAll();
  }

  function isInFuture(date: string): boolean {
    const until = Temporal.PlainDate.from(date).until(Temporal.Now.plainDateISO());
    return until.total("second") < 0;
  }

  function canVote(session: SessionResponse) {
    const start = Temporal.Instant.from(session.startTime);
    const end = start.add("PT2H");

    return Temporal.Now.instant().until(end).total("second") < 0;
  }
</script>

<svelte:head>
  <title>wikilaps</title>
</svelte:head>

<header class="w-full flex justify-between items-start">
  <div>
    <h1 class="h1">{m.app_name()}</h1>
    <p>{m.welcome_text()}</p>
  </div>

  <LanguageSwitcher />
</header>

<section class="grid gap-4 grid-cols-1 mt-6 max-w-3xl self-center w-full">
  {#each data.weekends as weekend (weekend.id)}
    {@const future = isInFuture(weekend.startDate)}
    <div
      id="round-{weekend.round}"
      aria-disabled={future}
      class={[
        "card border border-surface-200-800 card-hover divide-surface-200-800 block divide-y overflow-hidden",
        future
          ? "cursor-not-allowed preset-outlined-surface-100-900 opacity-50"
          : "preset-filled-surface-100-900",
      ]}
    >
      <article class="space-y-4 p-4">
        <div>
          <h2 class="h6 mb-2">
            {m.race_weekend_round({ n: weekend.round })} ({formatDate(weekend.startDate)})
          </h2>
          <h1 class="h3 mb-4">
            <span class="fi fi-{weekend.countryKey.toLowerCase()}"></span>
            {weekend.officialName.replace("Formula 1", "").replace("2026", "").trim()}
          </h1>
          {#if !future}
            <Accordion>
              {#each weekend.sessions as session, i (session.id)}
                {@const interactive = canVote(session) && !data.votes.includes(session.id)}
                {#if i !== 0}
                  <hr class="hr" />
                {/if}
                <Accordion.Item value={session.id.toString()}>
                  <h3>
                    <Accordion.ItemTrigger
                      class="font-bold flex items-center justify-between gap-2"
                    >
                      {sessionTypeLabel(session.sessionType)}
                      <Accordion.ItemIndicator class="group">
                        <ChevronDownIcon
                          class="h-5 w-5 transition group-data-[state=open]:rotate-180"
                        />
                      </Accordion.ItemIndicator>
                    </Accordion.ItemTrigger>
                  </h3>
                  <Accordion.ItemContent>
                    {#snippet element(attributes)}
                      {#if !attributes.hidden}
                        <div {...attributes} transition:slide={{ duration: 150 }} class="pb-3">
                          <VoteResults
                            {interactive}
                            sessionType={session.sessionType}
                            votes={session.votes}
                            onSubmit={(vote) => submitVote(session.id, vote)}
                          />
                        </div>
                      {/if}
                    {/snippet}
                  </Accordion.ItemContent>
                </Accordion.Item>
              {/each}
            </Accordion>
          {/if}
        </div>
      </article>
    </div>
  {/each}
</section>
