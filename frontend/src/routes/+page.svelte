<script lang="ts">
  import { Accordion } from "@skeletonlabs/skeleton-svelte";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import { slide } from "svelte/transition";
  import { sessionTypeLabel } from "$lib/i18n.js";

  const { data } = $props();
</script>

<svelte:head>
  <title>wikilaps</title>
</svelte:head>

<main class="container mx-auto mt-4">
  <h1 class="h1">wikilaps</h1>
  <p>Spoiler-free Formula 1 race recommendations.</p>

  <section class="grid gap-4 grid-cols-1 lg:grid-cols-3 xl:grid-cols-3 mt-6">
    {#each data.weekends as weekend (weekend.id)}
      <div class="card card-hover preset-outlined-primary-500">
        <header></header>
        <article class="space-y-4 p-4">
          <div>
            <h2 class="h6">
              Round {weekend.round}
            </h2>
            <h1 class="h3">
              {weekend.location}
            </h1>
            <Accordion>
              {#each weekend.sessions as session, i (session.id)}
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
                        <div {...attributes} transition:slide={{ duration: 150 }}>Hello!</div>
                      {/if}
                    {/snippet}
                  </Accordion.ItemContent>
                </Accordion.Item>
              {/each}
            </Accordion>
          </div>
        </article>
      </div>
    {/each}
  </section>
</main>
