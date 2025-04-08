<script>
    import "../app.css";
    import { HistoryBreadcrumb } from '$lib/components';
    let { children } = $props();
</script>

<div class="app">
  <main>
    <slot />
  </main>
</div>
