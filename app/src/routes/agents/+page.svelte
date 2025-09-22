<script lang="ts">
  import {
    Card,
    Button,
    Heading,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Modal,
    Label,
    Input,
    Textarea,
    Badge,
    Tabs,
    TabItem,
  } from "flowbite-svelte";
  import {
    PlusOutline,
    ArrowLeftOutline,
    TrashBinOutline,
  } from "flowbite-svelte-icons";
  import { PageHeader, StepConfig, DateTimeRow } from "$lib/components";
  import { readableDate } from "$lib/date";
  import {
    getAgents,
    deleteAgent,
    saveAgent,
    updateAgent,
    requestPlanWorkflow,
    getAgentCapabilities,
    updateAgentCapabilities,
    getAgentPolicy,
    updateAgentPolicy,
    getAgentWorkflowHistory,
    getAgentExecutionMetrics,
  } from "./api";
  import { getWorkflowsByAgent } from "../workflows/api";
  import type { WorkflowCompositionRequest } from "$lib/types";
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";

  // Selected resources for detail views
  let selectedAgent = $state<any | null>(null);
  let currentTab = $state<string | null>(null);

  // Data stores
  let agents = $state<any[] | undefined>(undefined);
  let originalAgent = $state<any | null>(null);
  let agentWorkflows = $state<any[]>([]);
  let showComposeModal = $state(false);

  // Agent metrics and configuration
  let agentMetrics = $state<any | null>(null);
  let agentCapabilities = $state<any | null>(null);
  let agentPolicy = $state<any | null>(null);
  let compositionRequest = $state<WorkflowCompositionRequest>({
    agent_id: 0,
    objective: "",
    context: {},
    constraints: {},
    is_ephemeral: false,
    auto_execute: false,
  });

  const hasAgentChanges = $derived(
    selectedAgent && originalAgent
      ? JSON.stringify(selectedAgent) !== JSON.stringify(originalAgent)
      : false,
  );

  $effect(() => {
    if (selectedAgent) {
      loadAgentWorkflows(selectedAgent.id);
    }
  });

  $effect(() => {
    if (selectedAgent && currentTab) {
      updateUrl(selectedAgent.id, currentTab);
    }
  });

  let showModal = $state(false);

  let agentFormData = $state({
    name: "",
    description: "",
  });

  // Workflow patterns for capabilities
  const workflowPatterns = [
    "data-processing",
    "web-scraping",
    "code-generation",
    "analysis",
    "integration",
  ];

  async function loadAgents() {
    try {
      agents = await getAgents();
    } catch (err) {
      console.error("Failed to load agents");
    }
  }

  async function loadAgentWorkflows(agentIdInput: string | number) {
    try {
      const id =
        typeof agentIdInput === "string"
          ? parseInt(agentIdInput, 10)
          : agentIdInput;
      if (isNaN(id)) {
        console.error("Invalid agentId for loadAgentWorkflows:", agentIdInput);
        agentWorkflows = [];
        return;
      }
      agentWorkflows = await getWorkflowsByAgent(id);
      // Load additional agent data
      const [capabilities, policy, metrics] = await Promise.all([
        getAgentCapabilities(id),
        getAgentPolicy(id),
        getAgentExecutionMetrics(id),
      ]);
      agentCapabilities = capabilities;
      agentPolicy = policy;
      agentMetrics = metrics;
    } catch (err) {
      console.error("Failed to load agent data", err);
      agentWorkflows = [];
    }
  }

  function updateUrl(agentId, tab) {
    const url = new URL(window.location.href);
    if (agentId) {
      url.searchParams.set("agentId", agentId);
    } else {
      url.searchParams.delete("agentId");
    }
    if (tab) {
      url.searchParams.set("tab", tab);
    } else {
      url.searchParams.delete("tab");
    }
    window.history.replaceState({}, "", url);
  }

  function selectAgent(agentProxy: any) {
    if (selectedAgent && selectedAgent?.id === agentProxy.id) {
      selectedAgent = null;
      originalAgent = null;
      currentTab = null;
      updateUrl(null, null);
    } else {
      const agentToClone = {
        id: agentProxy.id,
        name: agentProxy.name,
        description: agentProxy.description,
        created_at: agentProxy.created_at,
        updated_at: agentProxy.updated_at,
      };

      originalAgent = structuredClone(agentToClone);
      selectedAgent = structuredClone(agentToClone);
      currentTab = "General";
      updateUrl(agentProxy.id, currentTab);
    }
  }

  function changeTab(tab) {
    currentTab = tab;
    updateUrl(selectedAgent?.id, currentTab);
  }

  function backToList() {
    selectedAgent = null;
    currentTab = "General";
    updateUrl(null, null);
  }

  async function deleteAgentClick() {
    if (confirm("Are you sure you want to delete this agent?")) {
      const deleteAgentResponse = await deleteAgent(selectedAgent.id);
      agents = deleteAgentResponse;
      selectedAgent = null;
      updateUrl(null, null);
    }
  }

  async function saveChanges() {
    await updateAgent(selectedAgent);
    await loadAgents();
  }

  function openComposeModal() {
    if (!selectedAgent) return;

    compositionRequest = {
      agent_id: selectedAgent.id,
      objective: "",
      context: {},
      constraints: {},
      is_ephemeral: false,
      auto_execute: false,
    };
    showComposeModal = true;
  }

  async function handleComposeWorkflow() {
    if (!selectedAgent) return;

    try {
      await requestPlanWorkflow(compositionRequest);

      // Refresh agent workflows to show the new composition
      setTimeout(() => {
        loadAgentWorkflows(selectedAgent.id);
      }, 1000);

      showComposeModal = false;
      resetComposeForm();
      alert("Workflow composition request submitted successfully!");
    } catch (err) {
      console.error("Failed to request workflow composition", err);
      alert(
        "Failed to request workflow composition: " +
          (err instanceof Error ? err.message : String(err)),
      );
    }
  }

  function resetComposeForm() {
    compositionRequest = {
      agent_id: 0,
      objective: "",
      context: {},
      constraints: {},
      is_ephemeral: false,
      auto_execute: false,
    };
  }

  const breadcrumbs = [
    { label: "Home", url: "/" },
    { label: "Agents", url: "/agents" },
  ];

  const getActions = () =>
    selectedAgent
      ? [
          {
            label: "Compose Workflow",
            onClick: openComposeModal,
            color: "green",
            type: "button",
          },
          {
            label: "Save Changes",
            onClick: saveChanges,
            color: "blue",
            disabled: !hasAgentChanges,
            type: "button",
          },
          {
            label: "Delete",
            onClick: deleteAgentClick,
            icon: TrashBinOutline,
            color: "red",
            type: "button",
          },
        ]
      : [
          {
            label: "New Agent",
            onClick: () => (showModal = true),
            icon: PlusOutline,
            color: "blue",
            type: "button",
          },
        ];

  async function handleSubmit() {
    const newAgent = {
      name: agentFormData.name,
      description: agentFormData.description,
    };

    agents = await saveAgent(newAgent);
    resetForm();
    showModal = false;
    selectAgent(newAgent);
  }

  function resetForm() {
    agentFormData = {
      name: "",
      description: "",
    };
  }

  onMount(async () => {
    await loadAgents();

    const url = new URL(window.location.href);
    const agentId = url.searchParams.get("agentId");
    const tab = url.searchParams.get("tab");

    if (agentId && agents?.length) {
      const agent = agents.filter((a) => a.id === +agentId)[0];
      if (agent) {
        originalAgent = structuredClone(agent);
        selectedAgent = structuredClone(agent);
        currentTab = tab || "General";

        // Loading data for the agent
        loadAgentWorkflows(agent.id);
      }
    }
  });
</script>

<main class="container mx-auto p-4">
  <div class="flex-shrink-0">
    <PageHeader title="Agents" {breadcrumbs} actionBar={getActions()} />
  </div>

  <div class="flex flex-grow min-h-0">
    <!-- Agent List Pane -->
    <div
      class={`transition-all duration-300 ease-in-out pr-4 ${selectedAgent ? "w-2/5" : "w-full"}`}
    >
      <Card class="max-w-full">
        <div>
          <Table hoverable={true}>
            <TableHead>
              <TableHeadCell>Name</TableHeadCell>
              <TableHeadCell>Type</TableHeadCell>
              <TableHeadCell>Description</TableHeadCell>
              <TableHeadCell>Status</TableHeadCell>
              <TableHeadCell>Last Updated</TableHeadCell>
            </TableHead>
            <TableBody>
              {#if agents}
                {#each agents as agent (agent.id)}
                  <TableBodyRow
                    on:click={() => selectAgent(agent)}
                    class={`cursor-pointer transition-all duration-200 ${
                      selectedAgent?.id === agent.id
                        ? "bg-gradient-to-r from-sea/40 to-sea/20 border-l-4 border-sea font-medium shadow-sm"
                        : "hover:bg-gray-50 dark:hover:bg-gray-700 border-l-4 border-transparent"
                    }`}
                  >
                    <TableBodyCell
                      class={selectedAgent?.id === agent.id ? "text-sea" : ""}
                      >{agent.name}</TableBodyCell
                    >
                    <TableBodyCell>Composer</TableBodyCell>
                    <TableBodyCell class="truncate max-w-xs"
                      >{agent.description}</TableBodyCell
                    >
                    <TableBodyCell>
                      <Badge color="green">Active</Badge>
                    </TableBodyCell>
                    <TableBodyCell
                      >{readableDate(agent.updated_at)}</TableBodyCell
                    >
                  </TableBodyRow>
                {/each}
              {/if}
              {#if !agents || agents.length === 0}
                <TableBodyRow>
                  <TableBodyCell colspan="5" class="text-center py-10">
                    <Heading tag="h4" class="mb-2">No agents found.</Heading>
                    <p class="mb-4 text-gray-500 dark:text-gray-400">
                      Get started by creating a new agent.
                    </p>
                    <Button on:click={() => (showModal = true)} color="blue">
                      <PlusOutline class="mr-2 h-5 w-5" />
                      New Agent
                    </Button>
                  </TableBodyCell>
                </TableBodyRow>
              {/if}
            </TableBody>
          </Table>
        </div>
      </Card>
    </div>

    <!-- Agent Detail Pane -->
    {#if selectedAgent}
      <div class="w-3/5" transition:fly={{ x: 200, duration: 300 }}>
        <Card class="max-w-full">
          <div class="flex justify-between items-center mb-4">
            <div class="flex items-center gap-2">
              <Button on:click={backToList} color="light" size="sm">
                <ArrowLeftOutline class="mr-2 h-4 w-4" />
                Back to List
              </Button>
              <Heading tag="h3">{selectedAgent.name}</Heading>
            </div>
          </div>

          <Tabs style="underline">
            <TabItem
              open={currentTab === "General"}
              title="General"
              on:click={() => changeTab("General")}
            >
              <div class="py-4">
                <div class="space-y-6">
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                      <Label for="name" class="mb-2">Agent Name</Label>
                      <Input id="name" bind:value={selectedAgent.name} />
                    </div>
                    <div>
                      <Label for="type" class="mb-2">Agent Type</Label>
                      <Input
                        id="type"
                        value="Composer"
                        disabled
                        placeholder="All agents are composers"
                      />
                    </div>
                  </div>
                  <div>
                    <Label for="description" class="mb-2">Description</Label>
                    <Textarea
                      id="description"
                      rows="4"
                      bind:value={selectedAgent.description}
                    />
                  </div>
                  <div class="grid grid-cols-6">
                    <div>
                      <span
                        class="text-sm font-medium text-gray-700 dark:text-gray-300"
                        >Created At:</span
                      >
                      <DateTimeRow datetime={selectedAgent.created_at} />
                    </div>
                    <div>
                      <span
                        class="text-sm font-medium text-gray-700 dark:text-gray-300"
                        >Last Updated:</span
                      >
                      <DateTimeRow datetime={selectedAgent.updated_at} />
                    </div>
                  </div>
                </div>
              </div>
            </TabItem>

            <TabItem
              open={currentTab === "Workflows"}
              title="Created Workflows"
              on:click={() => changeTab("Workflows")}
            >
              <div class="py-4">
                <div class="space-y-4">
                  <div class="flex justify-between items-center">
                    <Heading tag="h4">Workflows Created by this Agent</Heading>
                    <Button
                      size="sm"
                      on:click={openComposeModal}
                      class="bg-sea text-black"
                    >
                      <PlusOutline class="mr-2 h-5 w-5" />
                      Compose New Workflow
                    </Button>
                  </div>
                  {#if agentWorkflows && agentWorkflows.length > 0}
                    <Table hoverable={true}>
                      <TableHead>
                        <TableHeadCell>Workflow Name</TableHeadCell>
                        <TableHeadCell>State</TableHeadCell>
                        <TableHeadCell>Type</TableHeadCell>
                        <TableHeadCell>Created</TableHeadCell>
                        <TableHeadCell>Actions</TableHeadCell>
                      </TableHead>
                      <TableBody>
                        {#each agentWorkflows as workflow (workflow.id)}
                          <tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
                            <TableBodyCell
                              >{workflow.name ||
                                `Workflow ${workflow.id}`}</TableBodyCell
                            >
                            <TableBodyCell>
                              <Badge
                                color={workflow.workflow_state === "stable"
                                  ? "green"
                                  : workflow.workflow_state === "unstable"
                                    ? "yellow"
                                    : "dark"}
                              >
                                {workflow.workflow_state}
                              </Badge>
                            </TableBodyCell>
                            <TableBodyCell
                              >{workflow.workflow_type || "—"}</TableBodyCell
                            >
                            <TableBodyCell
                              >{readableDate(
                                workflow.created_at,
                              )}</TableBodyCell
                            >
                            <TableBodyCell>
                              <Button
                                size="xs"
                                color="blue"
                                on:click={() => {
                                  window.location.href = `/workflows?workflowId=${workflow.id}`;
                                }}
                              >
                                View Details
                              </Button>
                            </TableBodyCell>
                          </tr>
                        {/each}
                      </TableBody>
                    </Table>
                  {:else}
                    <div
                      class="text-center py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-800"
                    >
                      <p class="text-gray-500 dark:text-gray-400 mb-4">
                        No workflows created by this agent yet.
                      </p>
                      <Button
                        class="bg-sea text-black"
                        on:click={openComposeModal}
                      >
                        <PlusOutline class="mr-2 h-5 w-5" /> Compose First Workflow
                      </Button>
                    </div>
                  {/if}
                </div>
              </div>
            </TabItem>

            <TabItem
              open={currentTab === "Capabilities"}
              title="Capabilities & Policy"
              on:click={() => changeTab("Capabilities")}
            >
              <div class="py-4">
                <div class="space-y-6">
                  <div>
                    <Heading tag="h4" class="mb-4">Agent Capabilities</Heading>
                    <div
                      class="p-4 border border-gray-300 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-800"
                    >
                      <p class="text-gray-500 dark:text-gray-400 text-center">
                        Capabilities management will be implemented here.<br />
                        This will include available tools, models, step limits, and
                        ephemeral workflow permissions.
                      </p>
                    </div>
                  </div>

                  <div>
                    <Heading tag="h4" class="mb-4">Agent Policy</Heading>
                    <div
                      class="p-4 border border-gray-300 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-800"
                    >
                      <p class="text-gray-500 dark:text-gray-400 text-center">
                        Policy management will be implemented here.<br />
                        This will include rate limits, security constraints, and
                        allowed patterns.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </TabItem>
          </Tabs>
        </Card>
      </div>
    {/if}
  </div>

  <!-- Add Agent Modal -->
  <Modal title="Add New Agent" bind:open={showModal} autoclose>
    <form
      onsubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}
      class="space-y-4"
    >
      <div>
        <Label for="modalAgentName" class="mb-2">Agent Name</Label>
        <Input
          id="modalAgentName"
          placeholder="Enter agent name"
          required
          bind:value={agentFormData.name}
        />
      </div>
      <div>
        <Label for="modalAgentType" class="mb-2">Agent Type</Label>
        <Input
          id="modalAgentType"
          value="Composer"
          disabled
          placeholder="All agents are composers"
        />
      </div>
      <div>
        <Label for="modalAgentDescription" class="mb-2">Description</Label>
        <Textarea
          id="modalAgentDescription"
          placeholder="Enter agent description"
          rows="3"
          bind:value={agentFormData.description}
        />
      </div>
      <div class="flex justify-end gap-4">
        <Button
          color="alternative"
          on:click={() => {
            showModal = false;
            resetForm();
          }}>Cancel</Button
        >
        <Button type="submit" color="blue">Create</Button>
      </div>
    </form>
  </Modal>

  <!-- Compose Workflow Modal -->
  <Modal title="Compose New Workflow" bind:open={showComposeModal} autoclose>
    <div class="space-y-4">
      <div>
        <Label for="composeAgentName" class="mb-2">Agent</Label>
        <Input
          id="composeAgentName"
          value={selectedAgent?.name || ""}
          readonly
          class="bg-gray-50 dark:bg-gray-700"
        />
      </div>
      <div>
        <Label for="composeObjective" class="mb-2">Objective *</Label>
        <Textarea
          id="composeObjective"
          placeholder="Describe what you want the workflow to accomplish..."
          rows="3"
          required
          bind:value={compositionRequest.objective}
        />
        <p class="text-sm text-gray-500 mt-1">
          Be specific about the goal and expected outcomes.
        </p>
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <Label class="mb-2 flex items-center">
            <input
              type="checkbox"
              class="mr-2"
              bind:checked={compositionRequest.is_ephemeral}
            />
            Ephemeral Workflow
          </Label>
          <p class="text-xs text-gray-500">
            Temporary workflow that will be automatically cleaned up.
          </p>
        </div>
        <div>
          <Label class="mb-2 flex items-center">
            <input
              type="checkbox"
              class="mr-2"
              bind:checked={compositionRequest.auto_execute}
            />
            Auto Execute
          </Label>
          <p class="text-xs text-gray-500">
            Automatically run the workflow after composition.
          </p>
        </div>
      </div>
      <div class="flex justify-end gap-4">
        <Button
          color="alternative"
          on:click={() => {
            showComposeModal = false;
            resetComposeForm();
          }}
        >
          Cancel
        </Button>
        <Button
          color="green"
          on:click={handleComposeWorkflow}
          disabled={!compositionRequest.objective.trim()}
        >
          Compose Workflow
        </Button>
      </div>
    </div>
  </Modal>
</main>
