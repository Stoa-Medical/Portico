<script>
  import { formatRelativeDate } from "$lib/date.js";
  import { onMount } from "svelte";
  import { Card, Label, Input, Textarea, Select } from "flowbite-svelte";

  const defaultPythonTemplate = `# Write your custom Python code below.
# - Use the \`source\` dictionary to access data from previous steps: source["variable_name"]
# - Your function must return a dictionary with the output data for the next step.
#
def executeScript(source):
    return source;
`;

  export let step;
  export let stepTypes = ["prompt", "python"];
  export let agents = [];

  let monaco;
  let editorElement;
  let editorInstance;

  onMount(async () => {
    if (typeof window !== "undefined") {
      try {
        monaco = await import("monaco-editor");
        if (step.step_type === "python") {
          initMonacoEditor();
        }
      } catch (error) {
        console.error("Failed to load Monaco Editor:", error);
      }
    }

    return () => {
      editorInstance?.dispose();
    };
  });

  let prevStepType = step.step_type;

  $: if (step.step_type === "python") {
    if (!step.step_content?.trim() && step.step_type !== prevStepType) {
      step.step_content = defaultPythonTemplate;
    }
    if (editorElement && !editorInstance) {
      initMonacoEditor();
    }
    prevStepType = step.step_type;
  } else if (editorInstance) {
    editorInstance.dispose();
    editorInstance = null;
    prevStepType = step.step_type;
  }

  function initMonacoEditor() {
    if (!editorElement) return;

    editorInstance = monaco.editor.create(editorElement, {
      value: step.step_content || defaultPythonTemplate,
      language: "python",
      theme: "vs-dark",
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 14,
      wordWrap: "on",
      scrollBeyondLastLine: false,
      lineNumbers: "on",
    });

    editorInstance.onDidChangeModelContent(() => {
      step.step_content = editorInstance.getValue();
    });
  }
</script>

<Card class="max-w-full">
  <div class="space-y-6 py-4">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <Label for="name">Step Name</Label>
        <Input id="name" bind:value={step.name} />
      </div>

      <div>
        <Label for="type">Step Type</Label>
        <Select id="type" bind:value={step.step_type}>
          {#each stepTypes as type}
            <option value={type}>{type}</option>
          {/each}
        </Select>
      </div>
    </div>

    <div>
      <Label for="agent">Associated Agent</Label>
      <Select id="agent" bind:value={step.agent_id}>
        {#each agents as agent}
          <option value={agent.id}>{agent.name}</option>
        {/each}
      </Select>
    </div>

    <div>
      <div class="flex justify-between items-center mb-2">
        <Label for="content">
          {step.step_type === "python" ? "Python Code" : "Prompt Template"}
        </Label>
      </div>

      {#if step.step_type === "python"}
        <div
          bind:this={editorElement}
          class="border border-gray-300 dark:border-gray-700 rounded-lg min-h-[300px] font-mono"
          style="height: 300px;"
        ></div>
      {:else}
        <Textarea
          id="content"
          rows="15"
          bind:value={step.step_content}
          class="font-mono"
        />
      {/if}
    </div>

    {#if step.id !== "new"}
      <div>
        <p class="text-sm text-gray-500">
          Last edited: {formatRelativeDate(step.updated_at) || "Just now"} | Created:
          {formatRelativeDate(step.created_at) || "Just now"}
        </p>
      </div>
    {/if}
  </div>
</Card>
