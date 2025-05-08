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
    Select,
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
    getSteps,
    updateStep,
    getAgents,
    deleteAgent,
    saveAgent,
    updateAgent,
    getRuntimeSessions,
    deleteStep,
  } from "./api";
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";

  // Selected resources for detail views
  let selectedAgent = $state<any | null>(null);
  let selectedStep = $state<any | null>(null);
  let selectedRuntimeSession = $state<any | null>(null);
  let currentTab = $state<string | null>(null);

  // Data stores
  let agents = $state<any[] | undefined>(undefined);
  let steps = $state<any[] | undefined>(undefined);
  let originalAgent = $state<any | null>(null);
  let runtimeSessions = $state<any[]>([]);

  const hasAgentChanges = $derived(
    selectedAgent && originalAgent
      ? JSON.stringify(selectedAgent) !== JSON.stringify(originalAgent)
      : false,
  );

  $effect(() => {
    if (selectedAgent) {
      loadSteps(selectedAgent.id);
      loadRuntimeSessions(selectedAgent.id);
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
    type: "Assistant",
    description: "",
    isActive: true,
  });

  const agentTypes = [
    { value: "Assistant", name: "Assistant" },
    { value: "Researcher", name: "Researcher" },
    { value: "Analyst", name: "Analyst" },
    { value: "Custom", name: "Custom" },
  ];

  async function loadAgents() {
    try {
      agents = await getAgents();
    } catch (err) {
      console.error("Failed to load agents");
    }
  }

  async function loadSteps(agentIdInput: string | number) {
    try {
      const id =
        typeof agentIdInput === "string"
          ? parseInt(agentIdInput, 10)
          : agentIdInput;
      if (isNaN(id)) {
        console.error("Invalid agentId for loadSteps:", agentIdInput);
        steps = [];
        return;
      }
      steps = await getSteps(id);
    } catch (err) {
      console.error("Failed to load steps", err);
      steps = [];
    }
  }

  async function saveStepData() {
    if (!selectedStep || !selectedAgent) return;

    try {
      await updateStep(selectedStep);
      if (selectedAgent) await loadSteps(selectedAgent.id);
      selectedStep = null;
    } catch (err) {
      console.error("Failed to save step", err);
    }
  }

  async function loadRuntimeSessions(agentIdInput: string | number) {
    try {
      const id =
        typeof agentIdInput === "string"
          ? parseInt(agentIdInput, 10)
          : agentIdInput;
      if (isNaN(id)) {
        console.error("Invalid agentId for loadRuntimeSessions:", agentIdInput);
        runtimeSessions = [];
        return;
      }
      runtimeSessions = await getRuntimeSessions(id);
    } catch (err) {
      console.error("Failed to load runtime sessions", err);
      runtimeSessions = [];
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
        type: agentProxy.type,
        description: agentProxy.description,
        agent_state: agentProxy.agent_state,
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

  const breadcrumbs = [
    { label: "Home", url: "/" },
    { label: "Agents", url: "/agents" },
  ];

  const getActions = () =>
    selectedAgent
      ? [
          {
            label: "Delete",
            onClick: deleteAgentClick,
            icon: TrashBinOutline,
            color: "red",
            type: "button",
          },
          {
            label: "Save Changes",
            onClick: saveChanges,
            color: "blue",
            disabled: !hasAgentChanges,
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
      description: agentFormData.description,
      agent_state: agentFormData.isActive ? "stable" : "inactive",
      name: agentFormData.name,
      type: agentFormData.type,
    };

    agents = await saveAgent(newAgent);
    resetForm();
    showModal = false;
    selectAgent(newAgent);
  }

  function resetForm() {
    agentFormData = {
      name: "",
      type: "Assistant",
      description: "",
      isActive: true,
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
        loadSteps(agent.id);
        loadRuntimeSessions(agent.id);
      }
    }
  });
</script>

<main class="flex flex-col h-full">
  <div class="flex-shrink-0">
    <PageHeader title="Agents" {breadcrumbs} actionBar={getActions()} />
  </div>

  <div class="flex mt-6 flex-grow min-h-0">
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
                    <TableBodyCell>{agent.type}</TableBodyCell>
                    <TableBodyCell class="truncate max-w-xs"
                      >{agent.description}</TableBodyCell
                    >
                    <TableBodyCell>
                      <Badge
                        color={agent.agent_state === "stable"
                          ? "green"
                          : "yellow"}
                      >
                        {agent.agent_state}
                      </Badge>
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
                      <Select
                        id="type"
                        items={agentTypes}
                        bind:value={selectedAgent.type}
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
            </TabItem>

            <TabItem
              open={currentTab === "Steps"}
              title="Steps"
              on:click={() => changeTab("Steps")}
            >
              <div class="py-4">
                <div class="space-y-4">
                  <div class="flex justify-between items-center">
                    <Heading tag="h4">Agent Steps</Heading>
                    <Button
                      size="sm"
                      href={`/steps/new?agentId=${selectedAgent.id}&agentName=${encodeURIComponent(selectedAgent.name)}`}
                      class="bg-sea text-black"
                    >
                      <PlusOutline class="mr-2 h-5 w-5" />
                      Add Step
                    </Button>
                  </div>
                  {#if steps && steps.length > 0}
                    <Table hoverable={true}>
                      <TableHead>
                        <TableHeadCell>Step Name</TableHeadCell>
                        <TableHeadCell>Type</TableHeadCell>
                        <TableHeadCell>Last Edited</TableHeadCell>
                        <TableHeadCell>Actions</TableHeadCell>
                      </TableHead>
                      <TableBody>
                        {#each steps as step (step.id)}
                          <tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
                            <TableBodyCell>{step.name}</TableBodyCell>
                            <TableBodyCell>
                              <Badge
                                color={step.step_type === "python"
                                  ? "blue"
                                  : "purple"}>{step.step_type}</Badge
                              >
                            </TableBodyCell>
                            <TableBodyCell
                              >{readableDate(step.updated_at)}</TableBodyCell
                            >
                            <TableBodyCell>
                              <Button
                                size="xs"
                                color="alternative"
                                href={`/steps/${step.id}?agentId=${selectedAgent.id}`}
                              >
                                Edit
                              </Button>
                              <Button
                                size="xs"
                                color="red"
                                class="ml-2"
                                on:click={async () => {
                                  if (
                                    confirm(
                                      "Are you sure you want to delete this step?",
                                    )
                                  ) {
                                    await deleteStep(step.id);
                                    await loadSteps(selectedAgent.id);
                                  }
                                }}
                              >
                                <TrashBinOutline class="w-4 h-4" />
                              </Button>
                            </TableBodyCell>
                          </tr>
                          {#if selectedStep?.id === step.id}
                            <tr>
                              <td
                                colspan="4"
                                class="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg"
                              >
                                <StepConfig
                                  bind:step={selectedStep}
                                  {agents}
                                  on:save={saveStepData}
                                />
                              </td>
                            </tr>
                          {/if}
                        {/each}
                      </TableBody>
                    </Table>
                  {:else}
                    <div
                      class="text-center py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-800"
                    >
                      <p class="text-gray-500 dark:text-gray-400 mb-4">
                        No steps found for this agent.
                      </p>
                      <Button
                        class="bg-sea text-black"
                        href={`/steps/new?agentId=${selectedAgent.id}&agentName=${encodeURIComponent(selectedAgent.name)}`}
                      >
                        <PlusOutline class="mr-2 h-5 w-5" /> Create First Step
                      </Button>
                    </div>
                  {/if}
                </div>
              </div>
            </TabItem>

            <TabItem
              open={currentTab === "Sessions"}
              title="Runtime Sessions"
              on:click={() => changeTab("Sessions")}
            >
              <div class="py-4">
                <div class="space-y-4">
                  <Heading tag="h4">Runtime Sessions</Heading>
                  {#if runtimeSessions && runtimeSessions.length > 0}
                    <Table hoverable={true}>
                      <TableHead>
                        <TableHeadCell>Session ID</TableHeadCell>
                        <TableHeadCell>Status</TableHeadCell>
                        <TableHeadCell>Started At</TableHeadCell>
                        <TableHeadCell>Last Activity</TableHeadCell>
                      </TableHead>
                      <TableBody>
                        {#each runtimeSessions as session (session.id)}
                          <tr>
                            <TableBodyCell>{session.id}</TableBodyCell>
                            <TableBodyCell>
                              <Badge
                                color={session.status === "completed"
                                  ? "green"
                                  : session.status === "running"
                                    ? "blue"
                                    : "yellow"}
                              >
                                {session.status}
                              </Badge>
                            </TableBodyCell>
                            <TableBodyCell
                              >{readableDate(session.created_at)}</TableBodyCell
                            >
                            <TableBodyCell
                              >{readableDate(session.updated_at)}</TableBodyCell
                            >
                          </tr>
                        {/each}
                      </TableBody>
                    </Table>
                  {:else}
                    <div class="text-center py-6 text-gray-500">
                      No runtime sessions found for this agent.
                    </div>
                  {/if}
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
    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
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
        <Select
          id="modalAgentType"
          items={agentTypes}
          bind:value={agentFormData.type}
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
</main>
