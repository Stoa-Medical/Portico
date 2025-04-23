<script lang="ts">
  import { Breadcrumb, BreadcrumbItem } from "flowbite-svelte";
  export let title: string = "";
  export let breadcrumbs: { label: string; url: string }[] = [];
  export let actionBar: any = [];
</script>

<div class="my-6">
  <Breadcrumb class="mb-4 h-[8px]">
    {#each breadcrumbs as { label, url }, index}
      <BreadcrumbItem href={url} home={index === 0}>
        {label}
      </BreadcrumbItem>
    {/each}
  </Breadcrumb>

  <div class="flex justify-between items-center min-h-[70px]">
    <div class="flex items-center h-full">
      <h1 class="flex items-center">
        {title}
      </h1>
    </div>

    <div class="flex items-center gap-2">
      {#each actionBar as { label, onClick, icon, color, disabled: shouldBeDisabled }}
        <button
          on:click={onClick}
          disabled={shouldBeDisabled}
          class={`py-2 px-4 rounded transition
  ${
    shouldBeDisabled
      ? "bg-gray-400 cursor-not-allowed"
      : color === "red"
        ? "bg-[#CE5A5A] hover:bg-red"
        : "bg-sea text-black hover:bg-sea"
  }`}
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
