<script lang="ts">
  import '@fontsource-variable/inter';
  import '../app.css';
  import { onMount } from 'svelte';

  const { children } = $props();

  onMount(() => {
    const saved = localStorage.getItem('theme') ?? 'airy';
    document.documentElement.setAttribute('data-theme', saved);
  });
</script>

{@render children()}
