<script>
  import "../app.css";
  import { onMount } from "svelte";

  onMount(() => {
    document.documentElement.classList.add("dark");
  });
</script>

<div class="app">
  <main>
    <slot />
  </main>
</div>
