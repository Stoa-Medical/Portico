<script lang="ts">
  import {
    Card,
    Heading,
    Label,
    Input,
    Badge,
    Button,
    Textarea,
  } from "flowbite-svelte";
  import { PlusOutline, CloseOutline } from "flowbite-svelte-icons";
  import type { AgentPolicy } from "$lib/types";

  interface Props {
    policy?: AgentPolicy | null;
    onChange?: (policy: AgentPolicy) => void;
  }

  let { policy = null, onChange = () => {} }: Props = $props();

  // Initialize with defaults if null
  let localPolicy = $state<AgentPolicy>(
    policy ?? {
      max_workflows_per_hour: 100,
      max_ephemeral_workflows: 50,
      allowed_patterns: [],
      allowed_workflow_types: [],
      execution_constraints: {
        max_runtime_seconds: 300,
        max_retries: 3,
      },
      security_constraints: {},
    },
  );

  let newPattern = $state("");
  let newWorkflowType = $state("");
  let securityConstraintsJson = $state(
    JSON.stringify(localPolicy.security_constraints ?? {}, null, 2),
  );

  const availableWorkflowTypes = [
    "data-processing",
    "web-scraping",
    "code-generation",
    "analysis",
    "integration",
    "transform",
    "custom",
  ];

  function addPattern() {
    if (
      newPattern &&
      localPolicy.allowed_patterns &&
      !localPolicy.allowed_patterns.includes(newPattern)
    ) {
      localPolicy.allowed_patterns = [
        ...localPolicy.allowed_patterns,
        newPattern,
      ];
      newPattern = "";
      onChange(localPolicy);
    }
  }

  function removePattern(pattern: string) {
    if (localPolicy.allowed_patterns) {
      localPolicy.allowed_patterns = localPolicy.allowed_patterns.filter(
        (p) => p !== pattern,
      );
      onChange(localPolicy);
    }
  }

  function addWorkflowType() {
    if (
      newWorkflowType &&
      localPolicy.allowed_workflow_types &&
      !localPolicy.allowed_workflow_types.includes(newWorkflowType)
    ) {
      localPolicy.allowed_workflow_types = [
        ...localPolicy.allowed_workflow_types,
        newWorkflowType,
      ];
      newWorkflowType = "";
      onChange(localPolicy);
    }
  }

  function removeWorkflowType(type: string) {
    if (localPolicy.allowed_workflow_types) {
      localPolicy.allowed_workflow_types =
        localPolicy.allowed_workflow_types.filter((t) => t !== type);
      onChange(localPolicy);
    }
  }

  function updateMaxWorkflows(value: number) {
    localPolicy.max_workflows_per_hour = value;
    onChange(localPolicy);
  }

  function updateMaxEphemeral(value: number) {
    localPolicy.max_ephemeral_workflows = value;
    onChange(localPolicy);
  }

  function updateMaxRuntime(value: number) {
    if (!localPolicy.execution_constraints) {
      localPolicy.execution_constraints = {};
    }
    localPolicy.execution_constraints.max_runtime_seconds = value;
    onChange(localPolicy);
  }

  function updateMaxRetries(value: number) {
    if (!localPolicy.execution_constraints) {
      localPolicy.execution_constraints = {};
    }
    localPolicy.execution_constraints.max_retries = value;
    onChange(localPolicy);
  }

  function updateSecurityConstraints() {
    try {
      localPolicy.security_constraints = JSON.parse(securityConstraintsJson);
      onChange(localPolicy);
    } catch (e) {
      console.error("Invalid JSON for security constraints");
    }
  }
</script>

<Card>
  <Heading tag="h4" class="mb-4">Agent Policy</Heading>

  <!-- Rate Limits Section -->
  <div class="grid grid-cols-2 gap-4 mb-6">
    <div>
      <Label class="mb-2">Max Workflows per Hour</Label>
      <Input
        type="number"
        bind:value={localPolicy.max_workflows_per_hour}
        on:change={(e) => updateMaxWorkflows(parseInt(e.target.value) || 100)}
        min="0"
        max="10000"
      />
    </div>
    <div>
      <Label class="mb-2">Max Ephemeral Workflows</Label>
      <Input
        type="number"
        bind:value={localPolicy.max_ephemeral_workflows}
        on:change={(e) => updateMaxEphemeral(parseInt(e.target.value) || 50)}
        min="0"
        max="1000"
      />
    </div>
  </div>

  <!-- Allowed Patterns Section -->
  <div class="mb-6">
    <Label class="mb-2">Allowed Patterns</Label>
    <div class="flex gap-2 mb-2">
      <Input
        bind:value={newPattern}
        placeholder="Enter allowed pattern"
        on:keypress={(e) => e.key === "Enter" && addPattern()}
      />
      <Button size="sm" on:click={addPattern}>
        <PlusOutline class="w-4 h-4" />
      </Button>
    </div>
    <div class="flex flex-wrap gap-2">
      {#each localPolicy.allowed_patterns ?? [] as pattern}
        <Badge color="blue">
          {pattern}
          <button on:click={() => removePattern(pattern)} class="ml-1">
            <CloseOutline class="w-3 h-3" />
          </button>
        </Badge>
      {/each}
      {#if !localPolicy.allowed_patterns?.length}
        <span class="text-sm text-gray-500">No restrictions (all allowed)</span>
      {/if}
    </div>
  </div>

  <!-- Allowed Workflow Types Section -->
  <div class="mb-6">
    <Label class="mb-2">Allowed Workflow Types</Label>
    <div class="flex gap-2 mb-2">
      <Input
        bind:value={newWorkflowType}
        placeholder="Enter workflow type"
        list="workflow-types-list"
        on:keypress={(e) => e.key === "Enter" && addWorkflowType()}
      />
      <datalist id="workflow-types-list">
        {#each availableWorkflowTypes as type}
          <option value={type} />
        {/each}
      </datalist>
      <Button size="sm" on:click={addWorkflowType}>
        <PlusOutline class="w-4 h-4" />
      </Button>
    </div>
    <div class="flex flex-wrap gap-2">
      {#each localPolicy.allowed_workflow_types ?? [] as type}
        <Badge color="green">
          {type}
          <button on:click={() => removeWorkflowType(type)} class="ml-1">
            <CloseOutline class="w-3 h-3" />
          </button>
        </Badge>
      {/each}
      {#if !localPolicy.allowed_workflow_types?.length}
        <span class="text-sm text-gray-500">No restrictions (all allowed)</span>
      {/if}
    </div>
  </div>

  <!-- Execution Constraints Section -->
  <div class="grid grid-cols-2 gap-4 mb-6">
    <div>
      <Label class="mb-2">Max Runtime (seconds)</Label>
      <Input
        type="number"
        bind:value={localPolicy.execution_constraints.max_runtime_seconds}
        on:change={(e) => updateMaxRuntime(parseInt(e.target.value) || 300)}
        min="1"
        max="3600"
      />
    </div>
    <div>
      <Label class="mb-2">Max Retries</Label>
      <Input
        type="number"
        bind:value={localPolicy.execution_constraints.max_retries}
        on:change={(e) => updateMaxRetries(parseInt(e.target.value) || 3)}
        min="0"
        max="10"
      />
    </div>
  </div>

  <!-- Security Constraints Section -->
  <div class="mb-6">
    <Label class="mb-2">Security Constraints (JSON)</Label>
    <Textarea
      bind:value={securityConstraintsJson}
      on:blur={updateSecurityConstraints}
      rows="4"
      class="font-mono text-sm"
    />
  </div>
</Card>
