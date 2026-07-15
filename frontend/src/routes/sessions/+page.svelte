<script lang="ts">
  import { goto } from "$app/navigation";
  import { page as currentPage } from "$app/state";
  import type { SessionType } from "$lib/api.js";
  import JumpToTopButton from "$lib/components/JumpToTopButton.svelte";
  import Seo from "$lib/components/Seo.svelte";
  import { getYear } from "$lib/date-time.js";
  import { grandPrixName, sessionTypeLabel } from "$lib/i18n.js";
  import * as m from "$lib/paraglide/messages";
  import { withSearchParams, type SearchParamValue } from "$lib/url.js";
  import { summarizeVotes } from "$lib/votes.js";
  import { ArrowLeftIcon, ArrowRightIcon } from "@lucide/svelte";
  import { Pagination, SegmentedControl } from "@skeletonlabs/skeleton-svelte";

  let { data } = $props();
  let sort = $derived(data.sort);
  let year = $derived(data.year);
  let type = $derived(data.type);

  const sessionTypes: SessionType[] = ["race", "sprint_race", "qualifying", "sprint_qualifying"];

  const ALL = "all";

  const badgePresets: Record<SessionType, string> = {
    race: "preset-filled-primary-500",
    sprint_race: "preset-filled-secondary-500",
    qualifying: "preset-tonal-primary",
    sprint_qualifying: "preset-tonal-secondary",
  };

  function navigate(updates: Record<string, SearchParamValue>) {
    const query = withSearchParams(currentPage.url.searchParams, { page: null, ...updates });
    // eslint-disable-next-line svelte/no-navigation-without-resolve
    return goto("/sessions" + query, { keepFocus: true, noScroll: true });
  }
</script>

<Seo title={m.session_page_heading()} description={m.seo_sessions_description()} />

<JumpToTopButton />

<div class="max-w-3xl w-full self-center flex flex-col">
  <h1 class="h2 font-bold tracking-tight">
    {m.session_page_heading()}
  </h1>

  <div class="my-4 flex flex-col gap-4">
    <div class="w-full overflow-x-auto pb-1">
      <SegmentedControl
        class="w-max"
        value={sort}
        onValueChange={(e) => navigate({ sort: e.value })}
      >
        <SegmentedControl.Label>{m.session_filter_sort()}</SegmentedControl.Label>
        <SegmentedControl.Control>
          <SegmentedControl.Indicator />
          <SegmentedControl.Item value="score">
            <SegmentedControl.ItemText class="whitespace-nowrap">
              {m.session_sort_quality()}
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
          <SegmentedControl.Item value="start_date">
            <SegmentedControl.ItemText class="whitespace-nowrap">
              {m.session_sort_recent()}
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
        </SegmentedControl.Control>
      </SegmentedControl>
    </div>

    <div class="w-full overflow-x-auto pb-1">
      <SegmentedControl
        class="w-max"
        value={year?.toString() ?? ALL}
        onValueChange={(e) => navigate({ year: e.value === ALL ? null : e.value })}
      >
        <SegmentedControl.Label>{m.session_filter_year()}</SegmentedControl.Label>
        <SegmentedControl.Control>
          <SegmentedControl.Indicator />
          <SegmentedControl.Item value={ALL}>
            <SegmentedControl.ItemText class="whitespace-nowrap">
              {m.session_filter_all()}
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
          {#each data.allYears as year (year)}
            <SegmentedControl.Item value={year.toString()}>
              <SegmentedControl.ItemText class="whitespace-nowrap">
                {year}
              </SegmentedControl.ItemText>
              <SegmentedControl.ItemHiddenInput />
            </SegmentedControl.Item>
          {/each}
        </SegmentedControl.Control>
      </SegmentedControl>
    </div>

    <div class="w-full overflow-x-auto pb-1">
      <SegmentedControl
        class="w-max"
        value={type ?? ALL}
        onValueChange={(e) => navigate({ type: e.value === ALL ? null : e.value })}
      >
        <SegmentedControl.Label>{m.session_filter_type()}</SegmentedControl.Label>
        <SegmentedControl.Control>
          <SegmentedControl.Indicator />
          <SegmentedControl.Item value={ALL}>
            <SegmentedControl.ItemText class="whitespace-nowrap">
              {m.session_filter_all()}
            </SegmentedControl.ItemText>
            <SegmentedControl.ItemHiddenInput />
          </SegmentedControl.Item>
          {#each sessionTypes as sessionType (sessionType)}
            <SegmentedControl.Item value={sessionType}>
              <SegmentedControl.ItemText class="whitespace-nowrap">
                {sessionTypeLabel(sessionType)}
              </SegmentedControl.ItemText>
              <SegmentedControl.ItemHiddenInput />
            </SegmentedControl.Item>
          {/each}
        </SegmentedControl.Control>
      </SegmentedControl>
    </div>
  </div>

  <section class="flex gap-4 flex-col">
    {#each data.sessions.content as session (session.id)}
      {@const summary = summarizeVotes(session.votes, session.sessionType)}
      <div
        class="card card-hover border border-surface-200-800 shadow-xl bg-tertiary-50 dark:bg-surface-800"
      >
        <header class="flex items-center gap-3 p-4">
          <span
            class="fi fi-{session.countryKey.toLowerCase()} shrink-0 rounded-sm text-2xl shadow-sm"
          ></span>
          <div class="min-w-0 grow">
            <div class="flex items-baseline gap-2">
              <h2 class="h5 truncate font-bold">
                {grandPrixName(session.grandPrixId)}
              </h2>
              <span class="text-sm opacity-60 tabular-nums shrink-0">
                {getYear(session.raceWeekendStartDate)}
              </span>
            </div>
            <div class="mt-1.5 flex flex-wrap items-center gap-x-2 gap-y-1">
              <span class="badge {badgePresets[session.sessionType]} text-xs">
                {sessionTypeLabel(session.sessionType)}
              </span>
              {#if summary.winner}
                <span class="text-sm">
                  <span class="font-semibold">{summary.winner.label}</span>
                  <span class="font-bold tabular-nums">{summary.percent}%</span>
                </span>
                <span class="text-xs opacity-60">
                  &middot; {m.poll_total_votes({ count: summary.total })}
                </span>
              {:else}
                <span class="text-xs opacity-60">{m.vote_summary_none()}</span>
              {/if}
            </div>
          </div>
        </header>
      </div>
    {/each}
  </section>

  <Pagination
    class="self-center mt-4 max-w-full flex-wrap justify-center"
    count={data.sessions.totalItems}
    pageSize={data.sessions.pageSize}
    page={data.sessions.pageNumber}
    siblingCount={0}
    onPageChange={(event) => navigate({ page: event.page })}
  >
    <Pagination.PrevTrigger>
      <ArrowLeftIcon class="size-4" />
    </Pagination.PrevTrigger>
    <Pagination.Context>
      {#snippet children(pagination)}
        {#each pagination().pages as page, index (page)}
          {#if page.type === "page"}
            <Pagination.Item {...page}>
              {page.value}
            </Pagination.Item>
          {:else}
            <Pagination.Ellipsis {index}>&#8230;</Pagination.Ellipsis>
          {/if}
        {/each}
      {/snippet}
    </Pagination.Context>
    <Pagination.NextTrigger>
      <ArrowRightIcon class="size-4" />
    </Pagination.NextTrigger>
  </Pagination>
</div>
