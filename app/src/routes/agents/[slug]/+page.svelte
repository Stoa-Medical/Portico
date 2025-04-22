<script>
  import { page } from "$app/stores";
  import {
    Card,
    Button,
    Heading,
    Tabs,
    TabItem,
    Label,
    Input,
    Textarea,
    Select,
    Toggle,
    Badge,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
  } from "flowbite-svelte";
  import {
    ArrowLeftOutline,
    TrashBinOutline,
    PlusOutline,
  } from "flowbite-svelte-icons";
  import { HistoryBreadcrumb } from "$lib/components";

  // Get agent ID from URL
  const agentId = $page.params.slug;

  // Mock function to fetch agent data (in a real app, this would be an API call)
  function getAgentData(id) {
    // Sample agent data
    return {
      id: parseInt(id),
      name: `Agent ${id}`,
      type: "Assistant",
      description:
        "This is a sample agent description that explains what this agent does and how it works.",
      status: "Active",
      lastActive: "2 hours ago",
      settings: {
        temperature: 0.7,
        maxTokens: 2048,
        topP: 0.9,
        frequencyPenalty: 0.5,
        presencePenalty: 0.5,
      },
      capabilities: ["Text Generation", "Question Answering", "Summarization"],
      isActive: true,
      model: "gpt-4",
      apiKey: "sk-••••••••••••••••••••••••",
      createdAt: "2023-10-15",
    };
  }

  // Load agent data
  let agent = getAgentData(agentId);

  // Mock function to get steps associated with this agent
  function getAgentSteps(agentId) {
    return [
      {
        id: 1,
        name: "Data Collection",
        step_type: "python",
        lastEdited: "2 hours ago",
      },
      {
        id: 2,
        name: "Text Analysis",
        step_type: "prompt",
        lastEdited: "1 day ago",
      },
      {
        id: 3,
        name: "Data Visualization",
        step_type: "python",
        lastEdited: "3 days ago",
      },
    ];
  }

  // Load steps data
  let steps = getAgentSteps(agentId);

  // Available models
  const models = [
    "gpt-4",
    "gpt-3.5-turbo",
    "claude-3-opus",
    "claude-3-sonnet",
    "llama-3",
  ].map((x) => ({ value: x, name: x }));

  // Available capabilities
  const availableCapabilities = [
    "Text Generation",
    "Question Answering",
    "Summarization",
    "Translation",
    "Code Generation",
    "Data Analysis",
  ];

  // Handle form submission
  function saveChanges() {
    // In a real app, this would send data to an API
    alert("Agent settings saved!");
  }

  // Handle agent deletion
  function deleteAgent() {
    if (confirm("Are you sure you want to delete this agent?")) {
      // In a real app, this would send a delete request to an API
      window.location.href = "/agents";
    }
  }

  // Go back to agents list
  function goBack() {
    window.location.href = "/agents";
  }

  // Navigate to step details
  function navigateToStep(id) {
    window.location.href = `/steps/${id}`;
  }

  // Create a new step for this agent
  function createNewStep() {
    window.location.href = `/steps/new?agentId=${agentId}&agentName=${encodeURIComponent(agent.name)}`;
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <HistoryBreadcrumb currentTitle={agent.name} />

    <div class="flex flex-col sm:flex-row gap-4 mb-4">
      <div class="flex items-center gap-3">
        <Button color="light" size="sm" on:click={goBack}>
          <ArrowLeftOutline class="mr-2 h-4 w-4" />
          Back
        </Button>
        <Heading tag="h1" class="text-2xl font-bold">{agent.name}</Heading>
        <Badge color={agent.isActive ? "green" : "none"}>
          {agent.status}
        </Badge>
      </div>
    </div>
    <div class="flex flex-wrap gap-2 mb-6">
      <Button color="red" on:click={deleteAgent}>
        <TrashBinOutline class="mr-2 h-5 w-5" />
        Delete
      </Button>
      <Button color="blue" on:click={saveChanges}>Save Changes</Button>
    </div>
  </div>

  <!-- Agent Configuration Tabs -->
  <Card class="max-w-full">
    <Tabs style="underline">
      <TabItem open title="General">
        <div class="space-y-6 py-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <Label for="name" class="mb-2">Agent Name</Label>
              <Input id="name" bind:value={agent.name} />
            </div>

            <div>
              <Label for="type" class="mb-2">Agent Type</Label>
              <Select
                id="type"
                items={["Assistant", "Researcher", "Analyst", "Custom"].map(
                  (x) => ({ value: x, name: x }),
                )}
                bind:value={agent.type}
              />
            </div>
          </div>

          <div>
            <Label for="description" class="mb-2">Description</Label>
            <Textarea
              id="description"
              rows="4"
              bind:value={agent.description}
            />
          </div>

          <div class="flex items-center gap-2">
            <Toggle bind:checked={agent.isActive} />
            <Label>Active Status</Label>
          </div>

          <div>
            <Label class="mb-2">Created On</Label>
            <p class="text-gray-700 dark:text-gray-300">{agent.createdAt}</p>
          </div>
        </div>
      </TabItem>

      <TabItem title="Steps">
        <div class="space-y-6 py-4">
          <div class="flex justify-between items-center mb-4">
            <p class="text-gray-700 dark:text-gray-300">
              Steps define the workflow for this agent. Each step can be a
              python script or a prompt template.
            </p>
            <Button class="bg-sea text-black" on:click={createNewStep}>
              <PlusOutline class="mr-2 h-5 w-5" />
              Add Step
            </Button>
          </div>

          {#if steps.length > 0}
            <Table hoverable={true}>
              <TableHead>
                <TableHeadCell>Name</TableHeadCell>
                <TableHeadCell>Type</TableHeadCell>
                <TableHeadCell>Last Edited</TableHeadCell>
                <TableHeadCell>Actions</TableHeadCell>
              </TableHead>
              <TableBody>
                {#each steps as step}
                  <TableBodyRow>
                    <TableBodyCell>{step.name}</TableBodyCell>
                    <TableBodyCell>
                      <Badge
                        color={step.step_type === "python" ? "blue" : "purple"}
                      >
                        {step.step_type}
                      </Badge>
                    </TableBodyCell>
                    <TableBodyCell>{step.lastEdited}</TableBodyCell>
                    <TableBodyCell>
                      <div class="flex gap-2">
                        <Button
                          size="xs"
                          color="light"
                          on:click={() => navigateToStep(step.id)}
                        >
                          View
                        </Button>
                      </div>
                    </TableBodyCell>
                  </TableBodyRow>
                {/each}
              </TableBody>
            </Table>
          {:else}
            <div
              class="text-center py-8 border rounded-lg bg-gray-50 dark:bg-gray-800"
            >
              <p class="text-gray-500 dark:text-gray-400 mb-4">
                No steps found for this agent
              </p>
              <Button class="bg-sea text-black" on:click={createNewStep}>
                <PlusOutline class="mr-2 h-5 w-5" />
                Create First Step
              </Button>
            </div>
          {/if}
        </div>
      </TabItem>

      <TabItem title="History">
        <div class="space-y-6 py-4">
          <p class="text-gray-700 dark:text-gray-300">
            This agent was last active {agent.lastActive}.
          </p>

          <div class="border rounded-lg p-4 bg-gray-50 dark:bg-gray-800">
            <p class="text-sm text-gray-500 dark:text-gray-400">
              Activity history will be displayed here. In a real application,
              this would show a log of agent actions and interactions.
            </p>
          </div>
        </div>
      </TabItem>
    </Tabs>
  </Card>
</main>
