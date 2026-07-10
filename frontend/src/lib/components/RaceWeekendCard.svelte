<script lang="ts">
  import type { SessionResponse, RaceWeekendResponse, VoteType } from "$lib/api";
  import { formatDate } from "$lib/date-time";
  import { Temporal } from "temporal-polyfill";
  import VoteResults from "./VoteResults.svelte";
  import { sessionTypeLabel } from "$lib/i18n";
  import * as m from "$lib/paraglide/messages";

  interface Props {
    weekend: RaceWeekendResponse;
    votes: number[];
    year: number;
    onSubmitVote: (sessionId: number, vote: VoteType) => void;
  }

  let { weekend, votes, year, onSubmitVote }: Props = $props();

  function isInFuture(date: string): boolean {
    const until = Temporal.PlainDate.from(date).until(Temporal.Now.plainDateISO());
    return until.total("second") < 0;
  }

  function canVote(session: SessionResponse) {
    const start = Temporal.Instant.from(session.startTime);
    const end = start.add("PT2H");

    return Temporal.Now.instant().until(end).total("second") < 0;
  }

  const future = $derived(isInFuture(weekend.startDate));
</script>

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
        {weekend.officialName.replace("Formula 1", "").replace(year.toString(), "").trim()}
      </h1>
      {#if !future}
        <section class="space-y-4">
          {#each weekend.sessions as session, i (session.id)}
            {@const interactive = canVote(session) && !votes.includes(session.id)}
            {#if i !== 0}
              <hr class="hr" />
            {/if}
            <h3 class="font-bold flex items-center justify-between gap-2">
              {sessionTypeLabel(session.sessionType)}
            </h3>
            <VoteResults
              {interactive}
              sessionType={session.sessionType}
              votes={session.votes}
              onSubmit={(vote) => onSubmitVote(session.id, vote)}
            />
          {/each}
        </section>
      {/if}
    </div>
  </article>
</div>
