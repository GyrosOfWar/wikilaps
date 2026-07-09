<script lang="ts">
  import { Menu, Portal } from "@skeletonlabs/skeleton-svelte";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import CheckIcon from "@lucide/svelte/icons/check";
  import { getLocale, locales, setLocale, type Locale } from "$lib/paraglide/runtime";

  /** Flag (flag-icons country code) and native name for each locale. */
  const localeMeta: Record<Locale, { flag: string; label: string }> = {
    en: { flag: "gb", label: "English" },
    de: { flag: "de", label: "Deutsch" },
  };

  const current = $derived(getLocale());

  function selectLocale(value: string) {
    if (value === current) return;
    // Cookie strategy: setLocale writes the cookie and reloads the page.
    setLocale(value as Locale);
  }
</script>

<Menu positioning={{ placement: "bottom-end" }} onSelect={(details) => selectLocale(details.value)}>
  <Menu.Trigger
    class="btn preset-tonal-surface border border-surface-200-800"
    aria-label="Change language"
  >
    <span class="fi fi-{localeMeta[current].flag}"></span>
    <span>{localeMeta[current].label}</span>
    <ChevronDownIcon class="h-4 w-4" />
  </Menu.Trigger>
  <Portal>
    <Menu.Positioner>
      <Menu.Content
        class="card preset-filled-surface-100-900 border border-surface-200-800 z-50 min-w-40 space-y-1 p-1 shadow-xl"
      >
        {#each locales as locale (locale)}
          <Menu.Item
            value={locale}
            class="flex cursor-pointer items-center gap-2 rounded px-3 py-2 data-[highlighted]:preset-tonal-primary"
          >
            <span class="fi fi-{localeMeta[locale].flag}"></span>
            <span class="grow">{localeMeta[locale].label}</span>
            {#if locale === current}
              <CheckIcon class="h-4 w-4" />
            {/if}
          </Menu.Item>
        {/each}
      </Menu.Content>
    </Menu.Positioner>
  </Portal>
</Menu>
