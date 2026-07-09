<script lang="ts">
  import * as m from "$lib/paraglide/messages";
  import type { SessionType, VoteCounts } from "$lib/api";
  import CheckIcon from "@lucide/svelte/icons/check";

  const { votes, sessionType }: { votes: VoteCounts; sessionType: SessionType } = $props();

  const options = $derived.by(() => {
    if (sessionType === "race") {
      return [
        { label: m.vote_type_full_race(), count: votes.full },
        { label: m.vote_type_race_in_30(), count: votes.raceIn30! },
        { label: m.vote_type_highlights(), count: votes.highlights },
      ];
    } else if (sessionType === "sprint_race") {
      return [
        { label: m.vote_type_full_race(), count: votes.full },
        { label: m.vote_type_highlights(), count: votes.highlights },
      ];
    } else {
      return [
        { label: m.vote_type_full_session(), count: votes.full },
        { label: m.vote_type_highlights(), count: votes.highlights },
      ];
    }
  });

  const total = $derived(options.reduce((sum, o) => sum + o.count, 0));
  const maxCount = $derived(Math.max(...options.map((o) => o.count)));
</script>

<div class="space-y-2">
  {#each options as option (option.label)}
    {@const pct = total > 0 ? Math.round((option.count / total) * 100) : 0}
    {@const isWinner = total > 0 && option.count === maxCount}
    <div
      class="relative overflow-hidden rounded border {isWinner
        ? 'border-primary-500'
        : 'border-surface-200-800'}"
    >
      <div
        class="absolute inset-y-0 left-0 transition-[width] duration-500 {isWinner
          ? 'bg-primary-500/25'
          : 'bg-surface-300-700/40'}"
        style="width: {pct}%"
      ></div>
      <div class="relative flex items-center justify-between gap-2 px-3 py-1.5">
        <span class="flex items-center gap-1.5 text-sm font-medium">
          {#if isWinner}
            <CheckIcon class="text-primary-500 h-4 w-4 shrink-0" />
          {/if}
          {option.label}
        </span>
        <span class="text-sm whitespace-nowrap">
          <span class="opacity-60">{option.count}</span>
          <span class="ml-1 font-bold tabular-nums">{pct}%</span>
        </span>
      </div>
    </div>
  {/each}

  <p class="pt-0.5 text-xs opacity-60">
    {total > 0 ? m.poll_total_votes({ n: total }) : m.poll_no_votes()}
  </p>
</div>
