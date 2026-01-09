<script lang="ts">
  import favicon from "$lib/assets/favicon.svg";

  let { children } = $props();
</script>

<svelte:head>
  <meta name="color-scheme" content="light dark" />
  <link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
