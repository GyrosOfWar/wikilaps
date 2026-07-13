<script lang="ts">
  import { page } from "$app/state";
  import { m } from "$lib/paraglide/messages";

  const SITE_URL = "https://wikilaps.com";

  let {
    title,
    description = m.seo_default_description(),
  }: { title?: string; description?: string } = $props();

  const fullTitle = $derived(title ? `${title} - ${m.app_name()}` : m.app_name());
  const canonicalUrl = $derived(new URL(page.url.pathname, SITE_URL).href);
</script>

<svelte:head>
  <title>{fullTitle}</title>
  <link rel="canonical" href={canonicalUrl} />
  <meta name="description" content={description} />
  <meta property="og:url" content={canonicalUrl} />
  <meta property="og:title" content={fullTitle} />
  <meta property="og:description" content={description} />
  <meta name="twitter:title" content={fullTitle} />
  <meta name="twitter:description" content={description} />
</svelte:head>
