<script>
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

  // Selected resources for detail views
  let selectedAgent = null;
  let selectedStep = null;
  let selectedRuntimeSession = null;
  let currentTab = null;

  // Load agents data
  let agents;
  let steps;
  let originalAgent = null;
  let hasAgentChanges = false;

  const loadAgents = async () => {
    try {
      agents = await getAgents();
    } catch (err) {
      console.error("Failed to load agents");
    }
  };

  async function loadSteps(agentId) {
    try {
      steps = await getSteps(agentId);
    } catch (err) {
      console.error("Failed to load steps", err);
      steps = [];
    }
  }

  async function saveStepData() {
    if (!selectedStep || !selectedAgent) return;

    try {
      await updateStep(selectedStep);
      await loadSteps(selectedAgent.id);
      selectedStep = null;
    } catch (err) {
      console.error("Failed to save step", err);
    }
  }

  let runtimeSessions = [];

  async function loadRuntimeSessions(agentId) {
    try {
      runtimeSessions = await getRuntimeSessions(agentId);
    } catch (err) {
      console.error("Failed to load runtime sessions", err);
      runtimeSessions = [];
    }
  }

  $: if (selectedAgent && originalAgent) {
    hasAgentChanges =
      JSON.stringify(selectedAgent) !== JSON.stringify(originalAgent);
  }

  $: if (selectedAgent) {
    loadSteps(selectedAgent.id);
    loadRuntimeSessions(selectedAgent.id);
  }

  $: if (selectedAgent && currentTab) {
    updateUrl(selectedAgent.id, currentTab);
  }

  let showModal = false;

  let agentFormData = {
    name: "",
    type: "Assistant",
    description: "",
  };

  const agentTypes = [
    { value: "Assistant", name: "Assistant" },
    { value: "Researcher", name: "Researcher" },
    { value: "Analyst", name: "Analyst" },
    { value: "Custom", name: "Custom" },
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

  function selectAgent(agent) {
    if (selectedAgent && selectedAgent?.id === agent.id) {
      selectedAgent = null;
      originalAgent = null;
      currentTab = null;
      updateUrl(null, null);
    } else {
      originalAgent = structuredClone(agent);
      selectedAgent = structuredClone(agent);
      currentTab = "General";
      updateUrl(agent.id, currentTab);
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

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <PageHeader title="Agents" {breadcrumbs} actionBar={getActions()} />

  <!-- Master-Detail View -->
  <div class="grid grid-cols-2 gap-6 mt-6">
    <!-- Agents List (Master View) -->
    <div class={selectedAgent ? "hidden lg:block" : "block"}>
      <Card class="max-w-full">
        <Table hoverable={true} data-testid="agents-table">
          <TableHead>
            <TableHeadCell>Id</TableHeadCell>
            <TableHeadCell>Name</TableHeadCell>
            <TableHeadCell>Type</TableHeadCell>
            <TableHeadCell>Status</TableHeadCell>
            <TableHeadCell>Last Updated</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each agents as agent}
              <TableBodyRow
                on:click={() => selectAgent(agent)}
                data-testid={`agent-row-${agent.name}`}
                class="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 {selectedAgent?.id ===
                agent.id
                  ? 'bg-blue-50 dark:bg-blue-900/20'
                  : ''}"
              >
                <TableBodyCell>{agent.id}</TableBodyCell>
                <TableBodyCell>{agent.name}</TableBodyCell>
                <TableBodyCell>{agent.type}</TableBodyCell>
                <TableBodyCell>
                  <span
                    class={agent.agent_state === "stable"
                      ? "text-green-500"
                      : "text-gray-500"}
                  >
                    {agent.agent_state}
                  </span>
                </TableBodyCell>
                <DateTimeRow datetime={agent.updated_at} />
              </TableBodyRow>
            {/each}

            {#if agents?.length === 0}
              <TableBodyRow>
                <TableBodyCell
                  colspan="5"
                  class="text-center py-4 text-gray-500"
                >
                  No agents found. Click "Add Agent" to create one.
                </TableBodyCell>
              </TableBodyRow>
            {/if}
          </TableBody>
        </Table>
      </Card>
    </div>

    <!-- Agent Details (Detail View) -->
    {#if selectedAgent}
      <div class="col-span-1">
        <Card class="max-w-full">
          <div class="mb-4 flex items-center gap-3">
            <Button
              color="light"
              size="sm"
              class="lg:hidden"
              on:click={backToList}
            >
              <ArrowLeftOutline class="mr-2 h-4 w-4" />
              Back
            </Button>
            <Heading tag="h2" class="text-xl font-bold"
              >{selectedAgent.name}</Heading
            >
            <Badge
              color={selectedAgent.agent_state === "stable" ? "green" : "none"}
            >
              {selectedAgent.agent_state}
            </Badge>
          </div>

          <Tabs style="underline">
            <TabItem
              open={currentTab === "General"}
              title="General"
              on:click={() => changeTab("General")}
            >
              <div class="space-y-6 py-4">
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

                <!-- <div class="flex items-center gap-2">
                  <Toggle bind:checked={selectedAgent.isActive} />
                  <Label>Active Status</Label>
                </div> -->

                <div>
                  <Label class="mb-2 font-bold inline-block">Created At:</Label>
                  <span class="text-gray-700 text-sm dark:text-gray-300">
                    {readableDate(selectedAgent.created_at)}
                  </span>
                </div>
              </div>
            </TabItem>

            <TabItem
              open={currentTab === "Steps"}
              title="Steps"
              on:click={() => changeTab("Steps")}
            >
              <div class="space-y-6 py-4">
                <div class="flex justify-between items-center mb-4">
                  <p class="text-gray-700 dark:text-gray-300">
                    Each step can be a Python script or a Prompt Template.
                  </p>
                  <Button
                    class="bg-sea text-black"
                    href={`/steps/new?agentId=${selectedAgent.id}&agentName=${encodeURIComponent(selectedAgent.name)}`}
                  >
                    <PlusOutline class="mr-2 h-5 w-5" />
                    Add Step
                  </Button>
                </div>

                {#if steps && steps.length > 0}
                  <Table hoverable={true} data-testid="steps-table">
                    <TableHead>
                      <TableHeadCell>Id</TableHeadCell>
                      <TableHeadCell>Name</TableHeadCell>
                      <TableHeadCell>Type</TableHeadCell>
                      <!-- <TableHeadCell>Last Edited</TableHeadCell> -->
                      <TableHeadCell>Actions</TableHeadCell>
                    </TableHead>
                    <TableBody>
                      {#each steps as step}
                        <TableBodyRow>
                          <TableBodyCell>{step.id}</TableBodyCell>
                          <TableBodyCell>{step.name}</TableBodyCell>
                          <TableBodyCell>
                            <Badge
                              color={step.step_type === "Python"
                                ? "blue"
                                : "purple"}
                            >
                              {step.step_type}
                            </Badge>
                          </TableBodyCell>
                          <!-- <TableBodyCell>{step.lastEdited}</TableBodyCell> -->
                          <TableBodyCell>
                            <div class="flex gap-2">
                              {#if selectedStep?.id === step.id}
                                <Button
                                  size="xs"
                                  color="light"
                                  on:click={() => (selectedStep = null)}
                                >
                                  Close
                                </Button>
                                <Button
                                  size="xs"
                                  color="light"
                                  on:click={async () => {
                                    await saveStepData();
                                    selectedStep = null;
                                  }}
                                >
                                  Save
                                </Button>
                                <Button
                                  size="xs"
                                  class="bg-[#CE5A5A]"
                                  on:click={async () => {
                                    await deleteStep(selectedStep.id);
                                    await loadSteps(selectedAgent.id);
                                    selectedStep = null;
                                  }}
                                >
                                  Delete
                                </Button>
                              {:else}
                                <Button
                                  size="xs"
                                  color="light"
                                  on:click={() => (selectedStep = step)}
                                >
                                  View
                                </Button>
                              {/if}
                            </div>
                          </TableBodyCell>
                        </TableBodyRow>
                        {#if selectedStep?.id === step.id}
                          <tr>
                            <td
                              colspan="4"
                              class="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg"
                            >
                              <StepConfig bind:step={selectedStep} {agents} />
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
                      No steps found for this agent
                    </p>
                    <Button
                      class="bg-sea text-black"
                      href={`/steps/new?agentId=${selectedAgent.id}&agentName=${encodeURIComponent(selectedAgent.name)}`}
                    >
                      <PlusOutline class="mr-2 h-5 w-5" />
                      Create First Step
                    </Button>
                  </div>
                {/if}
              </div>
            </TabItem>
            <TabItem
              open={currentTab === "Sessions"}
              title="Runtime Sessions"
              on:click={() => changeTab("Sessions")}
            >
              <div class="space-y-6 py-4">
                {#if runtimeSessions.length > 0}
                  <Table hoverable={true}>
                    <TableHead>
                      <TableHeadCell>ID</TableHeadCell>
                      <TableHeadCell>Status</TableHeadCell>
                      <TableHeadCell>Created</TableHeadCell>
                      <TableHeadCell>Last Updated</TableHeadCell>
                      <TableHeadCell>Actions</TableHeadCell>
                    </TableHead>
                    <TableBody>
                      {#each runtimeSessions as session}
                        <TableBodyRow>
                          <TableBodyCell>{session.id}</TableBodyCell>
                          <TableBodyCell>
                            <Badge color="green">{session.rts_status}</Badge>
                          </TableBodyCell>
                          <DateTimeRow datetime={session.created_at} />
                          <DateTimeRow datetime={session.updated_at} />
                          <TableBodyCell>
                            <div class="flex gap-2">
                              {#if selectedRuntimeSession?.id === session.id}
                                <Button
                                  size="xs"
                                  color="none"
                                  on:click={() =>
                                    (selectedRuntimeSession = null)}
                                >
                                  Close
                                </Button>
                              {:else}
                                <Button
                                  size="xs"
                                  color="light"
                                  on:click={() =>
                                    (selectedRuntimeSession = session)}
                                >
                                  View
                                </Button>
                              {/if}
                            </div>
                          </TableBodyCell>
                        </TableBodyRow>

                        {#if selectedRuntimeSession?.id === session.id}
                          <tr>
                            <td
                              colspan="6"
                              class="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg"
                            >
                              <div class="space-y-4">
                                <h3
                                  class="text-lg font-semibold text-gray-800 dark:text-white"
                                >
                                  Runtime Session ID: {session.id}
                                </h3>

                                {#if session.initial_data}
                                  <div>
                                    <Label class="font-bold">Initial Data</Label
                                    >
                                    <pre
                                      class="border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 p-3 rounded-md overflow-x-auto text-sm mt-2">
{JSON.stringify(session.initial_data, null, 2)}</pre>
                                  </div>
                                {/if}

                                {#if session.latest_result}
                                  <div class="mt-6">
                                    <Label class="font-bold"
                                      >Latest Result</Label
                                    >
                                    <pre
                                      class="border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 p-3 rounded-md overflow-x-auto text-sm mt-2">
{JSON.stringify(session.latest_result, null, 2)}</pre>
                                  </div>
                                {/if}
                              </div>
                            </td>
                          </tr>
                        {/if}
                      {/each}
                    </TableBody>
                  </Table>
                {:else}
                  <div class="text-center py-6 text-gray-500">
                    No runtime sessions found for this agent.
                  </div>
                {/if}
              </div>
            </TabItem>
          </Tabs>
        </Card>
      </div>
    {/if}
  </div>

  <!-- Add Agent Modal -->
  <Modal title="Add New Agent" bind:open={showModal} autoclose>
    <form on:submit={handleSubmit} class="space-y-4">
      <div>
        <Label for="name" class="mb-2">Agent Name</Label>
        <Input
          id="name"
          placeholder="Enter agent name"
          required
          bind:value={agentFormData.name}
        />
      </div>

      <div>
        <Label for="type" class="mb-2">Agent Type</Label>
        <Select id="type" items={agentTypes} bind:value={agentFormData.type} />
      </div>

      <div>
        <Label for="description" class="mb-2">Description</Label>
        <Textarea
          id="description"
          placeholder="Enter agent description"
          rows="3"
          bind:value={agentFormData.description}
        />
      </div>

      <!-- <div class="flex items-center gap-2">
        <Checkbox id="isActive" bind:checked={agentFormData.isActive} />
        <Label for="isActive">Active</Label>
      </div> -->

      <div class="flex justify-end gap-4">
        <Button
          color="alternative"
          on:click={() => {
            showModal = false;
            resetForm();
          }}>Cancel</Button
        >
        <Button type="submit" on:click={handleSubmit} color="blue"
          >Create</Button
        >
      </div>
    </form>
  </Modal>
</main>
