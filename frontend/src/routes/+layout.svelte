<script lang="ts">
  import "./layout.css";
  import "/node_modules/flag-icons/css/flag-icons.min.css";
  import favicon from "$lib/assets/favicon.svg";
  import { resolve } from "$app/paths";

  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import { m } from "$lib/paraglide/messages";

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<header
  class="z-50 sticky top-0 border-b border-surface-200-800 bg-surface-50-950/75 backdrop-blur-lg h-(--header-height) flex items-center"
>
  <div class="container mx-auto p-4 flex items-center justify-between gap-2">
    <h1 class="h4">
      <a href={resolve("/")}>
        {m.app_name()}
      </a>
    </h1>
    <nav class="flex gap-4">
      <a href={resolve("/weekends/2026")} class="btn hover:preset-tonal">Weekends</a>
      <LanguageSwitcher />
    </nav>
  </div>
</header>

<main class="container mx-auto mt-4 flex flex-col">
  {@render children()}
</main>
