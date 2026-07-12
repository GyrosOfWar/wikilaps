<script lang="ts">
  import { formatDate } from "$lib/date-time.js";
  import { grandPrixName } from "$lib/i18n.js";
  import * as m from "$lib/paraglide/messages";

  let { data } = $props();
</script>

<div class="max-w-3xl w-full self-center">
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
              {grandPrixName(session.grandPrixId)}
            </h2>
            <p class="text-sm opacity-60">
              <!-- {m.race_weekend_round({ n: session.round })} · {formatDate(session.startDate)} -->
            </p>
          </div>
        </header>
      </div>
    {/each}
  </section>
</div>
