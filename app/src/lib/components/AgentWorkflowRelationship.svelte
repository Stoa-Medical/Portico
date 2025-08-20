<script lang="ts">
  import { Card, Badge, Button, Heading } from "flowbite-svelte";
  import type { Agent, Workflow } from "$lib/types";

  interface Props {
    agent: Agent;
    workflows: Workflow[];
    onComposeWorkflow?: (agentId: number) => void;
  }

  let { agent, workflows, onComposeWorkflow = () => {} }: Props = $props();

  const ephemeralCount = $derived(
    workflows.filter((w) => w.is_ephemeral).length,
  );

  const stableCount = $derived(workflows.filter((w) => !w.is_ephemeral).length);

  const totalCount = $derived(workflows.length);
</script>

<Card>
  <div class="text-center mb-4">
    <Heading tag="h4">{agent.name}</Heading>
    <p class="text-sm text-gray-500 dark:text-gray-400">Composer Agent</p>
    {#if agent.description}
      <p class="text-xs text-gray-600 dark:text-gray-300 mt-1">
        {agent.description}
      </p>
    {/if}
  </div>

  <div class="space-y-2 mb-4">
    <div class="flex justify-between items-center">
      <span class="text-sm">Total Workflows:</span>
      <Badge color="primary">{totalCount}</Badge>
    </div>
    <div class="flex justify-between items-center">
      <span class="text-sm">Stable Workflows:</span>
      <Badge color="green">{stableCount}</Badge>
    </div>
    <div class="flex justify-between items-center">
      <span class="text-sm">Ephemeral Workflows:</span>
      <Badge color="yellow">{ephemeralCount}</Badge>
    </div>
  </div>

  {#if agent.capabilities_json}
    <div class="mb-4 p-3 bg-gray-50 dark:bg-gray-800 rounded">
      <p class="text-xs font-semibold mb-2">Capabilities:</p>
      <div class="flex flex-wrap gap-1">
        {#each agent.capabilities_json.tools ?? [] as tool}
          <Badge size="xs" color="blue">{tool}</Badge>
        {/each}
      </div>
    </div>
  {/if}

  <Button size="sm" on:click={() => onComposeWorkflow(agent.id)} class="w-full">
    Compose New Workflow
  </Button>
</Card>
