<script lang="ts">
  import type { ClassValue } from "svelte/elements";
  import type { RaceWeekendResponse, VoteType } from "$lib/api";
  import { formatDate } from "$lib/date-time";
  import VoteResults from "./VoteResults.svelte";
  import { grandPrixName, sessionTypeLabel } from "$lib/i18n";
  import * as m from "$lib/paraglide/messages";

  interface Props {
    class?: ClassValue;
    weekend: RaceWeekendResponse;
    onSubmitVote: (sessionId: number, vote: VoteType) => void;
  }

  let { weekend, onSubmitVote, class: klass }: Props = $props();
</script>

<div
  id="round-{weekend.round}"
  aria-disabled={weekend.upcoming}
  class={["card card-hover overflow-hidden", weekend.upcoming && "opacity-70", klass]}
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
    {#if weekend.upcoming}
      <p class="text-sm opacity-60">
        {m.race_voting_not_yet()}
      </p>
    {:else}
      <section class="space-y-4">
        {#each weekend.sessions as session, i (session.id)}
          {@const interactive = session.votingAllowed && !session.userVote}
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
