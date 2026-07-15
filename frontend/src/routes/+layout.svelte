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
  <meta property="og:site_name" content={m.app_name()} />
  <meta property="og:type" content="website" />
  <meta name="twitter:card" content="summary" />
</svelte:head>

<div class="min-h-screen flex flex-col">
  <header
    class="border-b border-surface-200-800 bg-surface-50-950 h-(--header-height) flex items-center"
  >
    <div class="container mx-auto p-4 flex items-center justify-between gap-2">
      <a href={resolve("/")} class="flex items-center gap-2.5 shrink-0">
        <span
          class="grid place-items-center size-9 rounded-lg preset-filled-primary-500 shadow-sm shadow-primary-500/30"
        >
          <FlagIcon class="size-5" />
        </span>
        <span class="h4 font-bold tracking-tight hidden sm:inline">{m.app_name()}</span>
      </a>
      <nav class="flex items-center gap-1 sm:gap-2">
        <a
          href={resolve("/races/2026")}
          class="btn btn-sm sm:btn-base preset-tonal-surface hover:preset-tonal-primary"
        >
          {m.nav_entry_races()}
        </a>
        <a
          href={resolve("/sessions")}
          class="btn btn-sm sm:btn-base preset-tonal-surface hover:preset-tonal-primary"
        >
          {m.nav_entry_sessions()}
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
</div>
