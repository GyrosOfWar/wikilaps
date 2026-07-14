<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import Seo from "$lib/components/Seo.svelte";
  import { formatDate } from "$lib/date-time.js";
  import { grandPrixName, sessionTypeLabel } from "$lib/i18n.js";
  import * as m from "$lib/paraglide/messages";
  import { ArrowLeftIcon, ArrowRightIcon } from "@lucide/svelte";
  import { Pagination } from "@skeletonlabs/skeleton-svelte";

  let { data } = $props();
</script>

<Seo title={m.session_page_heading()} description={m.seo_sessions_description()} />

<div class="max-w-3xl w-full self-center flex flex-col">
  <h1 class="h2 font-bold tracking-tight">
    {m.session_page_heading()}
  </h1>

  <section class="flex gap-4 flex-col">
    {#each data.sessions.content as session (session.id)}
      <div
        class="card card-hover border border-surface-200-800 shadow-xl bg-tertiary-50 dark:bg-surface-800"
      >
        <header class="flex items-center gap-3 border-b border-surface-200-800 p-4">
          <span
            class="fi fi-{session.countryKey.toLowerCase()} shrink-0 rounded-sm text-2xl shadow-sm"
          ></span>
          <div class="min-w-0 grow">
            <h2 class="h5 truncate font-bold">
              {grandPrixName(session.grandPrixId)} - {sessionTypeLabel(session.sessionType)}
            </h2>
            <p class="text-sm opacity-60">
              {formatDate(session.raceWeekendStartDate)}
            </p>
          </div>
        </header>
      </div>
    {/each}
  </section>

  <Pagination
    class="self-center mt-4"
    count={data.sessions.totalItems}
    pageSize={data.sessions.pageSize}
    page={data.sessions.pageNumber}
    onPageChange={(event) => goto(resolve(`/sessions?page=${event.page}`))}
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
