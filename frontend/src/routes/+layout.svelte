<script lang="ts">
  import "./layout.css";
  import "/node_modules/flag-icons/css/flag-icons.min.css";
  import favicon from "$lib/assets/favicon.svg";
  import { resolve } from "$app/paths";

  import LanguageSwitcher from "$lib/components/LanguageSwitcher.svelte";
  import { m } from "$lib/paraglide/messages";
  import FlagIcon from "@lucide/svelte/icons/flag";

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
  <title>wikilaps</title>
</svelte:head>

<header
  class="border-b border-surface-200-800 bg-surface-50-950 h-(--header-height) flex items-center"
>
  <div class="container mx-auto p-4 flex items-center justify-between gap-2">
    <a href={resolve("/")} class="flex items-center gap-2.5">
      <span
        class="grid place-items-center size-9 rounded-lg preset-filled-primary-500 shadow-sm shadow-primary-500/30"
      >
        <FlagIcon class="size-5" />
      </span>
      <span class="h4 font-bold tracking-tight">{m.app_name()}</span>
    </a>
    <nav class="flex items-center gap-2">
      <a href={resolve("/races/2026")} class="btn preset-tonal-surface hover:preset-tonal-primary">
        {m.nav_entry_races()}
      </a>
      <LanguageSwitcher />
    </nav>
  </div>
</header>

<main class="container mx-auto mt-4 flex flex-col px-4">
  {@render children()}
</main>

<footer class="text-base-content/55 mt-auto py-3 text-center text-xs">
  wikilaps {__APP_VERSION__}
</footer>
