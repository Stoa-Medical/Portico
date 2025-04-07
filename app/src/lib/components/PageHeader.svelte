<script lang="ts">
  export let title: string = '';
  export let breadcrumbs: { label: string; url: string }[] = [];
  export let actionBar = [];
</script>

<div class="mb-6">
    <!-- Breadcrumb -->
    <nav class="mb-4">
      <div class="flex items-center space-x-4">
        {#each breadcrumbs as { label, url }, i}
          <a href={url} class="text-blue-500 hover:underline">{label}</a>
          {#if i < breadcrumbs.length - 1}
            <span>/</span>
          {/if}
        {/each}
      </div>
    </nav>

    <!-- Header Title and Actions -->
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
      <h1 class="text-2xl font-bold">{title}</h1>

      <!-- Action Bar Buttons -->
      {#if actionBar.length > 0}
        <div class="flex flex-wrap gap-2">
          {#each actionBar as { label, onClick, icon, color }}
            <button
              on:click={onClick}
              class={`py-2 px-4 rounded text-white transition ${color === 'red' ? 'bg-red-500 hover:bg-red-600' : 'bg-blue-500 hover:bg-blue-600'}`}
            >
              {#if icon}
                <svelte:component this={icon} class="mr-2 h-5 w-5 inline" />
              {/if}
              {label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
</div>
