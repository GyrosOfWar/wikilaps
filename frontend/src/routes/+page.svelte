<script lang="ts">
  const { data } = $props();
</script>

<svelte:head>
  <title>wikilaps</title>
</svelte:head>

<main class="container mx-auto mt-4">
  <h1 class="h1">wikilaps</h1>
  <p>Spoiler-free Formula 1 race recommendations.</p>

  <section class="grid gap-4 grid-cols-1 lg:grid-cols-3 xl:grid-cols-4 mt-6">
    {#each data.weekends as weekend (weekend.id)}
      <div
        class="card preset-filled-surface-100-900 border border-surface-200-800 card-hover divide-surface-200-800"
      >
        <header></header>
        <article class="space-y-4 p-4">
          <div>
            <h2 class="h6">
              Round {weekend.round}
            </h2>
            <h1 class="h3">
              {weekend.location}
            </h1>
          </div>
        </article>
      </div>
    {/each}
  </section>
</main>
