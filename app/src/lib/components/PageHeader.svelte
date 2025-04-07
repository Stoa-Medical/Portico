<script lang="ts">
  import { Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
  export let title: string = '';
  export let breadcrumbs: { label: string; url: string }[] = [];
  export let actionBar = [];
</script>

<div class="mb-6">
    <!-- Breadcrumb -->
    <Breadcrumb class="mb-4">
      {#each breadcrumbs as { label, url }, index}
        <BreadcrumbItem href={url} home={index === 0}>
          {label}
        </BreadcrumbItem>
      {/each}
    </Breadcrumb>

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
