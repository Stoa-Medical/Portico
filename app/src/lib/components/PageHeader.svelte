<script lang="ts">
  import { Breadcrumb, BreadcrumbItem, Select } from "flowbite-svelte";
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
      {#each actionBar as action}
        {#if action.type === "button"}
          <button
            on:click={action.onClick}
            disabled={action.disabled}
            class={`py-2 px-4 rounded transition
              ${
                action.disabled
                  ? "bg-gray-400 cursor-not-allowed"
                  : action.color === "red"
                    ? "bg-[#CE5A5A] hover:bg-red"
                    : "bg-sea text-black hover:bg-sea"
              }`}
          >
            {#if action.icon}
              <svelte:component
                this={action.icon}
                class="mr-2 h-5 w-5 inline"
              />
            {/if}
            {action.label}
          </button>
        {:else if action.type === "select"}
          <div class="flex items-center gap-2">
            {#if action.icon}
              <svelte:component
                this={action.icon}
                class="h-5 w-5 text-gray-500"
              />
            {/if}
            <Select
              class="w-40"
              bind:value={action.value}
              on:change={action.onChange}
            >
              {#each action.options as option}
                <option value={option.value}>{option.name}</option>
              {/each}
            </Select>
          </div>
        {/if}
      {/each}
    </div>
  </div>
</div>
