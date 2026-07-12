<script lang="ts">
  import * as m from "$lib/paraglide/messages";
  import type { SessionType, VoteCounts, VoteType } from "$lib/api";
  import CheckIcon from "@lucide/svelte/icons/check";

  interface Option {
    value: VoteType;
    label: string;
    count: number;
  }

  interface Props {
    votes: VoteCounts;
    sessionType: SessionType;
    interactive?: boolean;
    onSubmit?: (vote: VoteType) => void;
  }

  const { votes, sessionType, interactive = false, onSubmit }: Props = $props();

  const options = $derived.by<Option[]>(() => {
    if (sessionType === "race") {
      return [
        { value: "FullRace", label: m.vote_type_full_race(), count: votes.full },
        { value: "RaceIn30", label: m.vote_type_race_in_30(), count: votes.raceIn30! },
        { value: "Highlights", label: m.vote_type_highlights(), count: votes.highlights },
      ];
    } else if (sessionType === "sprint_race") {
      return [
        { value: "FullRace", label: m.vote_type_full_race(), count: votes.full },
        { value: "Highlights", label: m.vote_type_highlights(), count: votes.highlights },
      ];
    } else {
      return [
        { value: "FullRace", label: m.vote_type_full_session(), count: votes.full },
        { value: "Highlights", label: m.vote_type_highlights(), count: votes.highlights },
      ];
    }
  });

  const total = $derived(options.reduce((sum, o) => sum + o.count, 0));
  const maxCount = $derived(Math.max(...options.map((o) => o.count)));

  let selected = $state<VoteType | null>(null);
  let showResults = $state(false);

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    if (selected) {
      onSubmit?.(selected);
    }
  }
</script>

{#snippet results()}
  <div class="space-y-2">
    {#each options as option (option.value)}
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
  </div>
{/snippet}

{#if interactive}
  {#if showResults}
    {@render results()}
    <button
      type="button"
      class="btn preset-outlined-surface-500 mt-3 mb-0.5 w-full transition-colors hover:preset-tonal-surface"
      onclick={() => (showResults = false)}
    >
      {m.vote_back_to_vote()}
    </button>
  {:else}
    <form class="space-y-3 mb-0.5" onsubmit={handleSubmit}>
      <div class="space-y-2">
        {#each options as option (option.value)}
          <label
            class={[
              "flex cursor-pointer items-center gap-2 rounded border px-3 py-2 text-sm font-medium transition-colors",
              selected === option.value
                ? "border-primary-500 preset-tonal-primary"
                : "border-surface-200-800 hover:preset-tonal-surface",
            ]}
          >
            <input
              type="radio"
              class="radio"
              name="vote"
              value={option.value}
              bind:group={selected}
            />
            {option.label}
          </label>
        {/each}
      </div>

      <div class="flex gap-2">
        <button
          type="submit"
          class="btn preset-filled-primary-500 flex-1 transition-colors hover:preset-filled-primary-600-400 disabled:pointer-events-none disabled:opacity-50"
          disabled={!selected}
        >
          {m.vote_submit()}
        </button>
        <button
          type="button"
          class="btn preset-outlined-surface-500 transition-colors hover:preset-tonal-surface"
          onclick={() => (showResults = true)}
        >
          {m.vote_show_results()}
        </button>
      </div>
    </form>
  {/if}
{:else}
  {@render results()}
{/if}

<p class="text-xs opacity-60">
  {m.poll_total_votes({ count: total })}
</p>
