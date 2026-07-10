<script lang="ts">
  import { resolve } from "$app/paths";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
</script>

<section class="max-w-2xl mt-4 self-center">
  <h1 class="h2 mb-4">Community-driven race votes for Formula 1 races.</h1>
  <a href={resolve("/weekends/2026")} class="btn preset-filled-primary-500 mt-4 px-4 py-3"
    >View 2026 Weekends <ArrowRight />
  </a>
</section>
