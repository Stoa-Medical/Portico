<script lang="ts">
  import { onMount } from "svelte";
  import type { Workflow, Step, RuntimeSession } from "$lib/types";
  import {
    getWorkflows,
    getSteps,
    getRuntimeSessions,
    runWorkflow,
    saveWorkflow,
    updateWorkflow,
    deleteWorkflow,
    saveStep,
    updateStep,
    deleteStep,
    type CreateWorkflowPayload,
    type UpdateWorkflowPayload,
    type CreateStepPayload,
    type UpdateStepPayload,
  } from "./api";

  let workflows = $state<Workflow[]>([]);
  let selectedWorkflow = $state<Workflow | null>(null);
  let workflowSteps = $state<Step[]>([]);
  let workflowRuntimeSessions = $state<RuntimeSession[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let currentView = $state<"list" | "detail">("list");

  // Form states
  let showCreateWorkflowModal = $state(false);
  let showEditWorkflowModal = $state(false);
  let showCreateStepModal = $state(false);
  let showEditStepModal = $state(false);
  let editingStep = $state<Step | null>(null);

  // Form data
  let newWorkflow = $state<CreateWorkflowPayload>({
    name: "",
    description: "",
    workflow_state: "stable",
    created_by_agent_id: null,
    workflow_type: null,
    is_ephemeral: false,
  });

  let editWorkflow = $state<UpdateWorkflowPayload>({
    id: 0,
    name: "",
    description: "",
    workflow_state: "stable",
  });

  let newStep = $state<CreateStepPayload>({
    workflow_id: 0,
    name: "",
    description: "",
    step_content: "",
    step_type: "python",
  });

  let editStep = $state<UpdateStepPayload>({
    id: 0,
    workflow_id: 0,
    name: "",
    description: "",
    step_content: "",
    step_type: "python",
  });

  onMount(async () => {
    await loadWorkflows();
  });

  async function loadWorkflows() {
    try {
      loading = true;
      error = null;
      workflows = await getWorkflows();
    } catch (e) {
      error = `Failed to load workflows: ${e instanceof Error ? e.message : "Unknown error"}`;
    } finally {
      loading = false;
    }
  }

  async function selectWorkflow(workflow: Workflow) {
    try {
      loading = true;
      selectedWorkflow = workflow;
      currentView = "detail";

      // Load workflow data in parallel
      const [steps, runtimeSessions] = await Promise.all([
        getSteps(workflow.id),
        getRuntimeSessions(workflow.id),
      ]);

      workflowSteps = steps;
      workflowRuntimeSessions = runtimeSessions;
    } catch (e) {
      error = `Failed to load workflow details: ${e instanceof Error ? e.message : "Unknown error"}`;
    } finally {
      loading = false;
    }
  }

  function goBackToList() {
    currentView = "list";
    selectedWorkflow = null;
    workflowSteps = [];
    workflowRuntimeSessions = [];
  }

  async function handleCreateWorkflow() {
    try {
      await saveWorkflow(newWorkflow);
      await loadWorkflows();
      showCreateWorkflowModal = false;
      resetNewWorkflowForm();
    } catch (e) {
      error = `Failed to create workflow: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  async function handleEditWorkflow() {
    try {
      await updateWorkflow(editWorkflow);
      await loadWorkflows();
      showEditWorkflowModal = false;

      // Refresh selected workflow if it's the one being edited
      if (selectedWorkflow && selectedWorkflow.id === editWorkflow.id) {
        const updatedWorkflow = workflows.find((w) => w.id === editWorkflow.id);
        if (updatedWorkflow) {
          selectedWorkflow = updatedWorkflow;
        }
      }
    } catch (e) {
      error = `Failed to update workflow: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  async function handleDeleteWorkflow(workflowId: number) {
    if (
      !confirm(
        "Are you sure you want to delete this workflow? This will also delete all associated steps.",
      )
    ) {
      return;
    }

    try {
      await deleteWorkflow(workflowId);
      await loadWorkflows();

      // If we're viewing the deleted workflow, go back to list
      if (selectedWorkflow && selectedWorkflow.id === workflowId) {
        goBackToList();
      }
    } catch (e) {
      error = `Failed to delete workflow: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  async function handleRunWorkflow(workflowId: number) {
    try {
      await runWorkflow(workflowId);
      // Refresh runtime sessions to show the new execution
      if (selectedWorkflow && selectedWorkflow.id === workflowId) {
        workflowRuntimeSessions = await getRuntimeSessions(workflowId);
      }
    } catch (e) {
      error = `Failed to run workflow: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  async function handleCreateStep() {
    if (!selectedWorkflow) return;

    try {
      newStep.workflow_id = selectedWorkflow.id;
      workflowSteps = await saveStep(newStep);
      showCreateStepModal = false;
      resetNewStepForm();
    } catch (e) {
      error = `Failed to create step: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  async function handleEditStep() {
    if (!selectedWorkflow) return;

    try {
      workflowSteps = await updateStep(editStep);
      showEditStepModal = false;
    } catch (e) {
      error = `Failed to update step: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  async function handleDeleteStep(stepId: number) {
    if (
      !selectedWorkflow ||
      !confirm("Are you sure you want to delete this step?")
    ) {
      return;
    }

    try {
      await deleteStep(stepId, selectedWorkflow.id);
      workflowSteps = await getSteps(selectedWorkflow.id);
    } catch (e) {
      error = `Failed to delete step: ${e instanceof Error ? e.message : "Unknown error"}`;
    }
  }

  function openEditWorkflowModal(workflow: Workflow) {
    editWorkflow = {
      id: workflow.id,
      name: workflow.name || "",
      description: workflow.description || "",
      workflow_state: workflow.workflow_state,
    };
    showEditWorkflowModal = true;
  }

  function openEditStepModal(step: Step) {
    editingStep = step;
    editStep = {
      id: step.id,
      workflow_id: step.workflow_id,
      name: step.name,
      description: step.description || "",
      step_content: step.step_content,
      step_type: step.step_type,
    };
    showEditStepModal = true;
  }

  function resetNewWorkflowForm() {
    newWorkflow = {
      name: "",
      description: "",
      workflow_state: "stable",
      created_by_agent_id: null,
      workflow_type: null,
      is_ephemeral: false,
    };
  }

  function resetNewStepForm() {
    newStep = {
      workflow_id: 0,
      name: "",
      description: "",
      step_content: "",
      step_type: "python",
    };
  }

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleString();
  }

  function getStatusColor(status: string) {
    switch (status) {
      case "completed":
        return "text-green-400";
      case "running":
        return "text-blue-400";
      case "failed":
        return "text-red-400";
      case "queued":
        return "text-yellow-400";
      default:
        return "text-gray-400";
    }
  }

  function getWorkflowStateColor(state: string) {
    switch (state) {
      case "stable":
        return "text-green-400";
      case "unstable":
        return "text-yellow-400";
      case "inactive":
        return "text-gray-400";
      default:
        return "text-gray-400";
    }
  }
</script>

<div class="h-full flex flex-col">
  <div class="flex items-center justify-between mb-6">
    <div class="flex items-center gap-4">
      {#if currentView === "detail"}
        <button
          onclick={goBackToList}
          class="flex items-center gap-2 text-cyan-400 hover:text-cyan-300 transition-colors"
        >
          ← Back to Workflows
        </button>
      {:else}
        <h1 class="text-2xl font-bold text-white">Workflows</h1>
      {/if}
    </div>

    {#if currentView === "list"}
      <button
        onclick={() => (showCreateWorkflowModal = true)}
        class="bg-cyan-600 hover:bg-cyan-700 text-white px-4 py-2 rounded transition-colors"
      >
        Create Workflow
      </button>
    {:else if selectedWorkflow}
      <div class="flex gap-2">
        <button
          onclick={() => (showCreateStepModal = true)}
          class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded transition-colors"
        >
          Add Step
        </button>
        <button
          onclick={() => handleRunWorkflow(selectedWorkflow!.id)}
          class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded transition-colors"
        >
          Run Workflow
        </button>
      </div>
    {/if}
  </div>

  {#if error}
    <div
      class="bg-red-900/50 border border-red-700 text-red-200 p-4 rounded mb-4"
    >
      {error}
      <button
        onclick={() => (error = null)}
        class="ml-2 text-red-400 hover:text-red-300">×</button
      >
    </div>
  {/if}

  {#if loading}
    <div class="flex items-center justify-center flex-1">
      <div class="text-white">Loading...</div>
    </div>
  {:else if currentView === "list"}
    <!-- Workflows List View -->
    <div class="flex-1 overflow-auto">
      {#if workflows.length === 0}
        <div class="text-center text-gray-400 mt-8">
          <p>No workflows found. Create your first workflow to get started.</p>
        </div>
      {:else}
        <div class="grid gap-4">
          {#each workflows as workflow}
            <div
              class="bg-gray-800 rounded-lg p-4 border border-gray-700 hover:border-gray-600 transition-colors"
            >
              <div class="flex items-start justify-between">
                <div
                  class="flex-1 cursor-pointer"
                  onclick={() => selectWorkflow(workflow)}
                >
                  <h3 class="text-lg font-semibold text-white mb-1">
                    {workflow.name || `Workflow ${workflow.id}`}
                  </h3>
                  {#if workflow.description}
                    <p class="text-gray-300 mb-2">{workflow.description}</p>
                  {/if}
                  <div class="flex items-center gap-4 text-sm text-gray-400">
                    <span
                      class={`font-medium ${getWorkflowStateColor(workflow.workflow_state)}`}
                    >
                      {workflow.workflow_state}
                    </span>
                    <span>Created: {formatDate(workflow.created_at)}</span>
                    {#if workflow.workflow_type}
                      <span>Type: {workflow.workflow_type}</span>
                    {/if}
                    {#if workflow.is_ephemeral}
                      <span class="text-yellow-400">Ephemeral</span>
                    {/if}
                  </div>
                </div>
                <div class="flex gap-2 ml-4">
                  <button
                    onclick={(e) => {
                      e.stopPropagation();
                      handleRunWorkflow(workflow.id);
                    }}
                    class="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded text-sm transition-colors"
                  >
                    Run
                  </button>
                  <button
                    onclick={(e) => {
                      e.stopPropagation();
                      openEditWorkflowModal(workflow);
                    }}
                    class="bg-gray-600 hover:bg-gray-700 text-white px-3 py-1 rounded text-sm transition-colors"
                  >
                    Edit
                  </button>
                  <button
                    onclick={(e) => {
                      e.stopPropagation();
                      handleDeleteWorkflow(workflow.id);
                    }}
                    class="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded text-sm transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else if selectedWorkflow}
    <!-- Workflow Detail View -->
    <div class="flex-1 overflow-auto">
      <div class="mb-6">
        <div class="flex items-start justify-between mb-4">
          <div>
            <h1 class="text-2xl font-bold text-white mb-2">
              {selectedWorkflow.name || `Workflow ${selectedWorkflow.id}`}
            </h1>
            {#if selectedWorkflow.description}
              <p class="text-gray-300 mb-2">{selectedWorkflow.description}</p>
            {/if}
            <div class="flex items-center gap-4 text-sm text-gray-400">
              <span
                class={`font-medium ${getWorkflowStateColor(selectedWorkflow.workflow_state)}`}
              >
                {selectedWorkflow.workflow_state}
              </span>
              <span>Created: {formatDate(selectedWorkflow.created_at)}</span>
              {#if selectedWorkflow.workflow_type}
                <span>Type: {selectedWorkflow.workflow_type}</span>
              {/if}
              {#if selectedWorkflow.is_ephemeral}
                <span class="text-yellow-400">Ephemeral</span>
              {/if}
            </div>
          </div>
          <div class="flex gap-2">
            <button
              onclick={() => openEditWorkflowModal(selectedWorkflow!)}
              class="bg-gray-600 hover:bg-gray-700 text-white px-3 py-1 rounded text-sm transition-colors"
            >
              Edit Workflow
            </button>
            <button
              onclick={() => handleDeleteWorkflow(selectedWorkflow!.id)}
              class="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded text-sm transition-colors"
            >
              Delete Workflow
            </button>
          </div>
        </div>
      </div>

      <!-- Steps Section -->
      <div class="mb-8">
        <h2 class="text-xl font-semibold text-white mb-4">Steps</h2>
        {#if workflowSteps.length === 0}
          <div class="bg-gray-800 rounded-lg p-6 text-center text-gray-400">
            <p>No steps configured for this workflow.</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each workflowSteps as step, index}
              <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <span class="text-sm font-medium text-gray-400"
                        >#{index + 1}</span
                      >
                      <h3 class="text-lg font-medium text-white">
                        {step.name}
                      </h3>
                      <span
                        class="text-xs px-2 py-1 bg-gray-700 text-gray-300 rounded"
                      >
                        {step.step_type}
                      </span>
                    </div>
                    {#if step.description}
                      <p class="text-gray-300 mb-2">{step.description}</p>
                    {/if}
                    <pre
                      class="text-sm text-gray-400 bg-gray-900 p-2 rounded overflow-auto max-h-32">
{step.step_content}
                    </pre>
                  </div>
                  <div class="flex gap-2 ml-4">
                    <button
                      onclick={() => openEditStepModal(step)}
                      class="bg-gray-600 hover:bg-gray-700 text-white px-3 py-1 rounded text-sm transition-colors"
                    >
                      Edit
                    </button>
                    <button
                      onclick={() =>
                        handleDeleteStep(
                          typeof step.id === "string"
                            ? parseInt(step.id)
                            : step.id,
                        )}
                      class="bg-red-600 hover:bg-red-700 text-white px-3 py-1 rounded text-sm transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Runtime Sessions Section -->
      <div>
        <h2 class="text-xl font-semibold text-white mb-4">Runtime Sessions</h2>
        {#if workflowRuntimeSessions.length === 0}
          <div class="bg-gray-800 rounded-lg p-6 text-center text-gray-400">
            <p>No execution history for this workflow.</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each workflowRuntimeSessions as session}
              <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <span class="text-sm font-medium text-gray-400"
                        >Session {session.id}</span
                      >
                      <span
                        class={`text-sm font-medium ${getStatusColor(session.rts_status)}`}
                      >
                        {session.rts_status}
                      </span>
                      {#if session.initiator_agent_id}
                        <span
                          class="text-xs px-2 py-1 bg-gray-700 text-gray-300 rounded"
                        >
                          Agent: {session.initiator_agent_id}
                        </span>
                      {/if}
                    </div>
                    <div class="grid grid-cols-2 gap-4 text-sm text-gray-400">
                      <div>
                        <span class="font-medium">Started:</span>
                        {formatDate(session.created_at)}
                      </div>
                      <div>
                        <span class="font-medium">Updated:</span>
                        {formatDate(session.updated_at)}
                      </div>
                      <div>
                        <span class="font-medium">Latest Step:</span>
                        {session.latest_step_idx + 1}
                      </div>
                      {#if session.total_execution_time}
                        <div>
                          <span class="font-medium">Execution Time:</span>
                          {session.total_execution_time}ms
                        </div>
                      {/if}
                    </div>
                    {#if session.latest_result}
                      <div class="mt-2">
                        <span class="text-sm font-medium text-gray-400"
                          >Latest Result:</span
                        >
                        <pre
                          class="text-xs text-gray-400 bg-gray-900 p-2 rounded mt-1 overflow-auto max-h-24">
{JSON.stringify(session.latest_result, null, 2)}
                        </pre>
                      </div>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- Create Workflow Modal -->
{#if showCreateWorkflowModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg p-6 w-full max-w-md">
      <h2 class="text-xl font-bold text-white mb-4">Create New Workflow</h2>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleCreateWorkflow();
        }}
      >
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Name</label
            >
            <input
              type="text"
              bind:value={newWorkflow.name}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Description</label
            >
            <textarea
              bind:value={newWorkflow.description}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none h-20"
            ></textarea>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >State</label
            >
            <select
              bind:value={newWorkflow.workflow_state}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
            >
              <option value="stable">Stable</option>
              <option value="unstable">Unstable</option>
              <option value="inactive">Inactive</option>
            </select>
          </div>
          <div class="flex items-center">
            <input
              type="checkbox"
              bind:checked={newWorkflow.is_ephemeral}
              class="mr-2"
              id="ephemeral"
            />
            <label for="ephemeral" class="text-sm text-gray-300"
              >Ephemeral workflow</label
            >
          </div>
        </div>
        <div class="flex gap-2 mt-6">
          <button
            type="submit"
            class="bg-cyan-600 hover:bg-cyan-700 text-white px-4 py-2 rounded transition-colors"
          >
            Create
          </button>
          <button
            type="button"
            onclick={() => {
              showCreateWorkflowModal = false;
              resetNewWorkflowForm();
            }}
            class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit Workflow Modal -->
{#if showEditWorkflowModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg p-6 w-full max-w-md">
      <h2 class="text-xl font-bold text-white mb-4">Edit Workflow</h2>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleEditWorkflow();
        }}
      >
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Name</label
            >
            <input
              type="text"
              bind:value={editWorkflow.name}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Description</label
            >
            <textarea
              bind:value={editWorkflow.description}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none h-20"
            ></textarea>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >State</label
            >
            <select
              bind:value={editWorkflow.workflow_state}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
            >
              <option value="stable">Stable</option>
              <option value="unstable">Unstable</option>
              <option value="inactive">Inactive</option>
            </select>
          </div>
        </div>
        <div class="flex gap-2 mt-6">
          <button
            type="submit"
            class="bg-cyan-600 hover:bg-cyan-700 text-white px-4 py-2 rounded transition-colors"
          >
            Save
          </button>
          <button
            type="button"
            onclick={() => (showEditWorkflowModal = false)}
            class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Create Step Modal -->
{#if showCreateStepModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg p-6 w-full max-w-2xl">
      <h2 class="text-xl font-bold text-white mb-4">Add New Step</h2>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleCreateStep();
        }}
      >
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Name</label
            >
            <input
              type="text"
              bind:value={newStep.name}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Description</label
            >
            <input
              type="text"
              bind:value={newStep.description}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Type</label
            >
            <select
              bind:value={newStep.step_type}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
            >
              <option value="python">Python</option>
              <option value="prompt">Prompt</option>
              <option value="webscrape">Web Scrape</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Content</label
            >
            <textarea
              bind:value={newStep.step_content}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none h-32"
              required
            ></textarea>
          </div>
        </div>
        <div class="flex gap-2 mt-6">
          <button
            type="submit"
            class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded transition-colors"
          >
            Add Step
          </button>
          <button
            type="button"
            onclick={() => {
              showCreateStepModal = false;
              resetNewStepForm();
            }}
            class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Edit Step Modal -->
{#if showEditStepModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg p-6 w-full max-w-2xl">
      <h2 class="text-xl font-bold text-white mb-4">Edit Step</h2>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleEditStep();
        }}
      >
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Name</label
            >
            <input
              type="text"
              bind:value={editStep.name}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
              required
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Description</label
            >
            <input
              type="text"
              bind:value={editStep.description}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Type</label
            >
            <select
              bind:value={editStep.step_type}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none"
            >
              <option value="python">Python</option>
              <option value="prompt">Prompt</option>
              <option value="webscrape">Web Scrape</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1"
              >Content</label
            >
            <textarea
              bind:value={editStep.step_content}
              class="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-cyan-500 focus:outline-none h-32"
              required
            ></textarea>
          </div>
        </div>
        <div class="flex gap-2 mt-6">
          <button
            type="submit"
            class="bg-cyan-600 hover:bg-cyan-700 text-white px-4 py-2 rounded transition-colors"
          >
            Save Changes
          </button>
          <button
            type="button"
            onclick={() => (showEditStepModal = false)}
            class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
