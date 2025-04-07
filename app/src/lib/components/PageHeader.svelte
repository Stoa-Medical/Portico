<script lang="ts">
  import { Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
  export let title: string = '';
  export let breadcrumbs: { label: string; url: string }[] = [];
  export let actionBar: any = [];
</script>

<div class="mb-6">
    <Breadcrumb class="mb-4 h-[8px]">
      {#each breadcrumbs as { label, url }, index}
        <BreadcrumbItem href={url} home={index === 0}>
          {label}
        </BreadcrumbItem>
      {/each}
    </Breadcrumb>

    <div class="flex justify-between items-center min-h-[70px]">
      <div class="flex items-center h-full">
        <h1 class="text-2xl font-bold flex items-center h-full">
          {title}
        </h1>
      </div>

      <div class="flex items-center gap-2">
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
    </div>
</div>
