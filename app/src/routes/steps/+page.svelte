<script>
  import PageHeader from "./../../lib/components/PageHeader.svelte";
  import {
    Card,
    Button,
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
  } from "flowbite-svelte";
  import { PlusOutline } from "flowbite-svelte-icons";
  import { getSteps, saveStep, getAgents } from "../../routes/agents/api";

  let steps = [];
  let agents = [];

  let showModal = false;

  let newStep = {
    name: "",
    step_type: "Prompt",
    agent_id: "",
    step_content: "",
  };

  const stepTypes = ["Prompt", "Python"];

  // Fetch actual data on load
  const loadData = async () => {
    try {
      agents = await getAgents();
      steps = (await getSteps()) || [];
    } catch (err) {
      console.error("Failed to load steps or agents:", err);
    }
  };

  loadData();

  async function handleSubmit() {
    const newStepData = {
      name: newStep.name,
      step_type: newStep.step_type,
      agent_id: parseInt(newStep.agent_id),
      step_content: newStep.step_content,
    };

    try {
      await saveStep(newStepData);
      steps = await getSteps(); // refresh the list
      resetForm();
      showModal = false;
    } catch (err) {
      console.error("Error saving step:", err);
    }
  }

  function resetForm() {
    newStep = {
      name: "",
      type: "Prompt",
      agent_id: "",
      content: "",
    };
  }

  function navigateToStep(id) {
    window.location.href = `/steps/${id}`;
  }

  const breadcrumbs = [
    { label: "Home", url: "/" },
    { label: "Step Templates", url: "/steps" },
  ];
  const actionBar = [
    {
      label: "Add Step",
      icon: PlusOutline,
      color: "blue",
      onClick: () => (showModal = true),
    },
  ];
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <PageHeader title="Step Templates" {breadcrumbs} {actionBar} />

  <!-- Steps List -->
  <Card class="max-w-full">
    <Table hoverable={true}>
      <TableHead>
        <TableHeadCell>Name</TableHeadCell>
        <TableHeadCell>Type</TableHeadCell>
        <TableHeadCell>Agent</TableHeadCell>
        <TableHeadCell>Last Edited</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each steps as step}
          <TableBodyRow
            on:click={() => navigateToStep(step.id)}
            class="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            <TableBodyCell>{step.name}</TableBodyCell>
            <TableBodyCell>
              <Badge color={step.step_type === "Python" ? "blue" : "purple"}>
                {step.step_type}
              </Badge>
            </TableBodyCell>
            <TableBodyCell>{step.agentName}</TableBodyCell>
            <TableBodyCell>{step.lastEdited}</TableBodyCell>
          </TableBodyRow>
        {/each}

        {#if steps.length === 0}
          <TableBodyRow>
            <TableBodyCell colspan="4" class="text-center py-4 text-gray-500">
              No steps found. Click "Add Step" to create one.
            </TableBodyCell>
          </TableBodyRow>
        {/if}
      </TableBody>
    </Table>
  </Card>

  <!-- Add Step Modal -->
  <Modal title="Add New Step" bind:open={showModal} autoclose>
    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
      <div>
        <Label for="name" class="mb-2">Step Name</Label>
        <Input
          id="name"
          placeholder="Enter step name"
          required
          bind:value={newStep.name}
        />
      </div>

      <div>
        <Label for="type" class="mb-2">Step Type</Label>
        <Select id="type" items={stepTypes} bind:value={newStep.step_type} />
      </div>

      <div>
        <Label for="agent" class="mb-2">Associated Agent</Label>
        <Select id="agent" required>
          <option value="" disabled selected>Select an agent</option>
          {#each agents as agent}
            <option value={agent.id}>{agent.name}</option>
          {/each}
        </Select>
      </div>

      <div>
        <Label for="content" class="mb-2">Initial Content</Label>
        <Textarea
          id="content"
          placeholder={newStep.step_type === "Python"
            ? "# Enter Python code here"
            : "Enter prompt text here"}
          rows="5"
          bind:value={newStep.content}
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
        <Button type="submit" color="blue">Add Step</Button>
      </div>
    </form>
  </Modal>
</main>
