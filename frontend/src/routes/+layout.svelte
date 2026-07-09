<script lang="ts">
  import "./layout.css";
  import "/node_modules/flag-icons/css/flag-icons.min.css";
  import favicon from "$lib/assets/favicon.svg";

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<main class="container mx-auto mt-4 flex flex-col">
  {@render children()}
</main>
