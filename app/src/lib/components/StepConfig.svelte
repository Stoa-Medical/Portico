<script>
  import { onMount } from "svelte";
  import {
    Card,
    Label,
    Input,
    Textarea,
    Select,
    Toggle,
  } from "flowbite-svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap } from "@codemirror/view";
  import { defaultKeymap } from "@codemirror/commands";
  import { python } from "@codemirror/lang-python";
  import { lintGutter, linter } from "@codemirror/lint";
  import {
    syntaxHighlighting,
    defaultHighlightStyle,
  } from "@codemirror/language";

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

  let editorElement;
  let editorView;

  // Python linter (same as original)
  function pythonLint(view) {
    const diagnostics = [];
    const text = view.state.doc.toString();
    const lines = text.split("\n");

    lines.forEach((line, i) => {
      const lineNum = i + 1;
      const from = view.state.doc.line(lineNum).from;

      if (/^\s*print\s+[^(]/.test(line)) {
        diagnostics.push({
          from: from + line.indexOf("print"),
          to: from + line.indexOf("print") + 5,
          severity: "warning",
          message: "Use print() function instead of print statement (Python 3)",
        });
      }

      if (
        line.match(/^ +/) &&
        !line.match(/^    +|^        +|^            +/)
      ) {
        diagnostics.push({
          from,
          to: from + line.length,
          severity: "error",
          message: "Indentation should be multiples of 4 spaces",
        });
      }

      if (
        line.match(/^import\s+\w+/) &&
        !text.includes(line.match(/import\s+(\w+)/)[1])
      ) {
        diagnostics.push({
          from,
          to: from + line.length,
          severity: "info",
          message: "Potentially unused import",
        });
      }

      if (
        (line.match(/^\s*def\s+\w+\([^)]*\)(?!\s*:)/) ||
          line.match(/^\s*class\s+\w+(?!\s*:)/)) &&
        !line.includes(":")
      ) {
        diagnostics.push({
          from,
          to: from + line.length,
          severity: "error",
          message: "Missing colon at the end of statement",
        });
      }

      const variableMatch = line.match(/^\s*(\w+)\s*=/);
      if (variableMatch) {
        const variable = variableMatch[1];
        const variableUsage = new RegExp(`[^\\w]${variable}[^\\w]`);
        let foundBefore = false;

        for (let j = 0; j < i; j++) {
          if (variableUsage.test(lines[j])) {
            foundBefore = true;
            break;
          }
        }

        if (foundBefore) {
          diagnostics.push({
            from: from + line.indexOf(variable),
            to: from + line.indexOf(variable) + variable.length,
            severity: "warning",
            message: `Variable '${variable}' might be used before assignment`,
          });
        }
      }
    });

    return diagnostics;
  }

  function initCodeEditor() {
    if (!editorElement) return;

    const startState = EditorState.create({
      doc: step.step_content,
      extensions: [
        python(),
        syntaxHighlighting(defaultHighlightStyle),
        lintGutter(),
        linter(pythonLint),
        keymap.of(defaultKeymap),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            step.step_content = update.state.doc.toString();
          }
        }),
        EditorState.allowMultipleSelections.of(true),
        EditorView.theme({
          "&": {
            fontSize: "14px",
            height: "100%",
            minHeight: "200px",
          },
          ".cm-scroller": {
            overflow: "auto",
            fontFamily: "monospace",
          },
          ".cm-content": {
            caretColor: "#0e9",
          },
          ".cm-activeLine": {
            backgroundColor: "rgba(0, 0, 0, 0.05)",
          },
          ".cm-activeLineGutter": {
            backgroundColor: "rgba(0, 0, 0, 0.05)",
          },
          ".cm-gutters": {
            backgroundColor: "#f8f9fa",
            color: "#999",
            border: "none",
            borderRight: "1px solid #ddd",
          },
        }),
      ],
    });

    editorView = new EditorView({
      state: startState,
      parent: editorElement,
    });
  }

  onMount(() => {
    if (step.step_type === "python") initCodeEditor();
    return () => {
      if (editorView) editorView.destroy();
    };
  });

  let prevStepType = step.step_type;

  $: if (step.step_type === "python") {
    if (!step.step_content?.trim() && step.step_type !== prevStepType) {
      step.step_content = defaultPythonTemplate;
    }
    if (editorElement && !editorView) {
      initCodeEditor();
    }
    prevStepType = step.step_type;
  } else if (editorView) {
    editorView.destroy();
    editorView = null;
    prevStepType = step.step_type;
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
        <!-- <div class="flex items-center gap-2"> -->
        <!-- <Toggle bind:checked={step.isActive} /> -->
        <!-- <span class="text-sm">Active</span> -->
        <!-- </div> -->
      </div>

      {#if step.step_type === "python"}
        <div
          bind:this={editorElement}
          class="border border-gray-300 rounded-lg min-h-[300px] font-mono"
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

    <div>
      <p class="text-sm text-gray-500">
        <!-- TODO: Add these into schema -->
        Last edited: {step.lastEdited || "Just now"} | Created: {step.createdAt ||
          "Just now"}
      </p>
    </div>
  </div>
</Card>

<style>
  :global(.cm-editor) {
    height: 300px;
    overflow: auto;
  }

  :global(.cm-gutters) {
    background-color: #f8f9fa;
    border-right: 1px solid #ddd;
  }

  :global(.cm-activeLineGutter) {
    background-color: #e9ecef;
  }

  :global(.cm-content) {
    padding: 4px 8px;
  }

  :global(.cm-line) {
    padding: 0 4px;
  }

  :global(.cm-keyword) {
    color: #07a;
  }
  :global(.cm-def) {
    color: #00f;
  }
  :global(.cm-variable) {
    color: #000;
  }
  :global(.cm-variable-2) {
    color: #05a;
  }
  :global(.cm-string) {
    color: #a11;
  }
  :global(.cm-comment) {
    color: #090;
  }
  :global(.cm-number) {
    color: #905;
  }
  :global(.cm-operator) {
    color: #a67f59;
  }
  :global(.cm-meta) {
    color: #555;
  }
  :global(.cm-builtin) {
    color: #30a;
  }
</style>
