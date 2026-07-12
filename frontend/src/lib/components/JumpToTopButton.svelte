<script lang="ts">
  import { ArrowUp } from "@lucide/svelte";

  function onClick() {
    window.scrollTo({ top: 0 });
  }
</script>

<button
  class="btn preset-filled-primary-500 fixed bottom-8 right-8 z-50 h-12 w-12 rounded-full shadow-lg"
  onclick={onClick}
>
  <ArrowUp />
</button>
