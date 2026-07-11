<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import type { SessionResponse, RaceWeekendResponse, VoteType } from "$lib/api";
  import { formatDate } from "$lib/date-time";
  import { Temporal } from "temporal-polyfill";
  import VoteResults from "./VoteResults.svelte";
  import { sessionTypeLabel } from "$lib/i18n";
  import * as m from "$lib/paraglide/messages";

  interface Props {
    class?: ClassValue;
    weekend: RaceWeekendResponse;
    onSubmitVote: (sessionId: number, vote: VoteType) => void;
  }

  let { weekend, onSubmitVote, class: klass }: Props = $props();

  function isInFuture(date: string): boolean {
    const until = Temporal.PlainDate.from(date).until(Temporal.Now.plainDateISO());
    return until.total("second") < 0;
  }

  function canVote(session: SessionResponse) {
    const start = Temporal.Instant.from(session.startTime);
    const end = start.add("PT2H");

    return Temporal.Now.instant().until(end).total("second") < 0;
  }

  // translate a GP based on its ID like `las-vegas` to a message key like `gp_las_vegas`
  function grandPrixName(grandPrixId: string) {
    const id = `gp_${grandPrixId.replace("-", "_")}`;
    // @ts-expect-error dynamic key but it's generally fine
    const fn = m[id];
    if (fn) {
      return fn();
    } else {
      console.warn(`No translation key found for input '${id}', falling back to ID`);
      return grandPrixId;
    }
  }

  const future = $derived(isInFuture(weekend.startDate));
</script>

<div
  id="round-{weekend.round}"
  aria-disabled={future}
  class={["card card-hover overflow-hidden", future && "opacity-70", klass]}
>
  <header class="flex items-center gap-3 border-b border-surface-200-800 p-4">
    <span class="badge preset-filled-secondary-500 shrink-0 font-bold">
      R{weekend.round}
    </span>
    <span class="fi fi-{weekend.countryKey.toLowerCase()} shrink-0 rounded-sm text-2xl shadow-sm"
    ></span>
    <div class="min-w-0 grow">
      <h2 class="h5 truncate font-bold">
        {grandPrixName(weekend.grandPrixId)}
      </h2>
      <p class="text-sm opacity-60">
        {m.race_weekend_round({ n: weekend.round })} · {formatDate(weekend.startDate)}
      </p>
    </div>
  </header>

  <div class="p-4">
    {#if future}
      <p class="text-sm opacity-60">Voting opens once the weekend gets underway.</p>
    {:else}
      <section class="space-y-4">
        {#each weekend.sessions as session, i (session.id)}
          {@const interactive = canVote(session) && !session.userVote}
          {#if i !== 0}
            <hr class="hr" />
          {/if}
          <h3 class="text-xs font-bold tracking-wide uppercase opacity-70">
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
</div>
