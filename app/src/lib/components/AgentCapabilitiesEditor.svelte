<script lang="ts">
  import {
    Card,
    Heading,
    Label,
    Input,
    Checkbox,
    Badge,
    Button,
  } from "flowbite-svelte";
  import { PlusOutline, CloseOutline } from "flowbite-svelte-icons";
  import type { AgentCapabilities } from "$lib/types";

  interface Props {
    capabilities?: AgentCapabilities | null;
    onChange?: (capabilities: AgentCapabilities) => void;
  }

  let { capabilities = null, onChange = () => {} }: Props = $props();

  // Initialize with defaults if null
  let localCapabilities = $state<AgentCapabilities>(
    capabilities ?? {
      tools: [],
      models: [],
      max_steps: 10,
      can_create_ephemeral: false,
      can_auto_execute: false,
      workflow_patterns: [],
    },
  );

  let newTool = $state("");
  let newModel = $state("");
  let newPattern = $state("");

  // Available tools and models for suggestions
  const availableTools = [
    "python",
    "webscrape",
    "database",
    "api_call",
    "file_read",
    "file_write",
  ];

  const availableModels = ["gpt-4", "gpt-3.5-turbo", "claude", "llama"];

  const availablePatterns = [
    "data-processing",
    "web-scraping",
    "code-generation",
    "analysis",
    "integration",
    "transform",
  ];

  function addTool() {
    if (newTool && !localCapabilities.tools.includes(newTool)) {
      localCapabilities.tools = [...localCapabilities.tools, newTool];
      newTool = "";
      onChange(localCapabilities);
    }
  }

  function removeTool(tool: string) {
    localCapabilities.tools = localCapabilities.tools.filter((t) => t !== tool);
    onChange(localCapabilities);
  }

  function addModel() {
    if (newModel && !localCapabilities.models.includes(newModel)) {
      localCapabilities.models = [...localCapabilities.models, newModel];
      newModel = "";
      onChange(localCapabilities);
    }
  }

  function removeModel(model: string) {
    localCapabilities.models = localCapabilities.models.filter(
      (m) => m !== model,
    );
    onChange(localCapabilities);
  }

  function addPattern() {
    if (
      newPattern &&
      localCapabilities.workflow_patterns &&
      !localCapabilities.workflow_patterns.includes(newPattern)
    ) {
      localCapabilities.workflow_patterns = [
        ...localCapabilities.workflow_patterns,
        newPattern,
      ];
      newPattern = "";
      onChange(localCapabilities);
    }
  }

  function removePattern(pattern: string) {
    if (localCapabilities.workflow_patterns) {
      localCapabilities.workflow_patterns =
        localCapabilities.workflow_patterns.filter((p) => p !== pattern);
      onChange(localCapabilities);
    }
  }

  function updateMaxSteps(value: number) {
    localCapabilities.max_steps = value;
    onChange(localCapabilities);
  }

  function toggleEphemeral() {
    localCapabilities.can_create_ephemeral =
      !localCapabilities.can_create_ephemeral;
    onChange(localCapabilities);
  }

  function toggleAutoExecute() {
    localCapabilities.can_auto_execute = !localCapabilities.can_auto_execute;
    onChange(localCapabilities);
  }
</script>

<Card>
  <Heading tag="h4" class="mb-4">Agent Capabilities</Heading>

  <!-- Tools Section -->
  <div class="mb-6">
    <Label class="mb-2">Available Tools</Label>
    <div class="flex gap-2 mb-2">
      <Input
        bind:value={newTool}
        placeholder="Enter tool name"
        list="tools-list"
        on:keypress={(e) => e.key === "Enter" && addTool()}
      />
      <datalist id="tools-list">
        {#each availableTools as tool}
          <option value={tool} />
        {/each}
      </datalist>
      <Button size="sm" on:click={addTool}>
        <PlusOutline class="w-4 h-4" />
      </Button>
    </div>
    <div class="flex flex-wrap gap-2">
      {#each localCapabilities.tools as tool}
        <Badge color="blue">
          {tool}
          <button on:click={() => removeTool(tool)} class="ml-1">
            <CloseOutline class="w-3 h-3" />
          </button>
        </Badge>
      {/each}
    </div>
  </div>

  <!-- Models Section -->
  <div class="mb-6">
    <Label class="mb-2">Available Models</Label>
    <div class="flex gap-2 mb-2">
      <Input
        bind:value={newModel}
        placeholder="Enter model name"
        list="models-list"
        on:keypress={(e) => e.key === "Enter" && addModel()}
      />
      <datalist id="models-list">
        {#each availableModels as model}
          <option value={model} />
        {/each}
      </datalist>
      <Button size="sm" on:click={addModel}>
        <PlusOutline class="w-4 h-4" />
      </Button>
    </div>
    <div class="flex flex-wrap gap-2">
      {#each localCapabilities.models as model}
        <Badge color="green">
          {model}
          <button on:click={() => removeModel(model)} class="ml-1">
            <CloseOutline class="w-3 h-3" />
          </button>
        </Badge>
      {/each}
    </div>
  </div>

  <!-- Workflow Patterns Section -->
  <div class="mb-6">
    <Label class="mb-2">Workflow Patterns</Label>
    <div class="flex gap-2 mb-2">
      <Input
        bind:value={newPattern}
        placeholder="Enter workflow pattern"
        list="patterns-list"
        on:keypress={(e) => e.key === "Enter" && addPattern()}
      />
      <datalist id="patterns-list">
        {#each availablePatterns as pattern}
          <option value={pattern} />
        {/each}
      </datalist>
      <Button size="sm" on:click={addPattern}>
        <PlusOutline class="w-4 h-4" />
      </Button>
    </div>
    <div class="flex flex-wrap gap-2">
      {#each localCapabilities.workflow_patterns ?? [] as pattern}
        <Badge color="purple">
          {pattern}
          <button on:click={() => removePattern(pattern)} class="ml-1">
            <CloseOutline class="w-3 h-3" />
          </button>
        </Badge>
      {/each}
    </div>
  </div>

  <!-- Settings Section -->
  <div class="mb-6">
    <Label class="mb-2">Max Steps</Label>
    <Input
      type="number"
      bind:value={localCapabilities.max_steps}
      on:change={(e) => updateMaxSteps(parseInt(e.target.value) || 10)}
      min="1"
      max="100"
    />
  </div>

  <div class="space-y-2">
    <Checkbox
      bind:checked={localCapabilities.can_create_ephemeral}
      on:change={toggleEphemeral}
    >
      Can create ephemeral workflows
    </Checkbox>
    <Checkbox
      bind:checked={localCapabilities.can_auto_execute}
      on:change={toggleAutoExecute}
    >
      Can auto-execute workflows
    </Checkbox>
  </div>
</Card>
