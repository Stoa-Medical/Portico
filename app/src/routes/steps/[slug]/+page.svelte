<script>
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { 
    Card, 
    Button, 
    Heading, 
    Breadcrumb, 
    BreadcrumbItem,
    Label,
    Input,
    Textarea,
    Select,
    Badge,
    Toggle
  } from 'flowbite-svelte';
  import { ArrowLeftOutline, TrashBinOutline, PlayOutline } from 'flowbite-svelte-icons';
  
  // CodeMirror imports
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap } from '@codemirror/view';
  import { defaultKeymap } from '@codemirror/commands';
  import { python } from '@codemirror/lang-python';
  import { lintGutter, linter } from '@codemirror/lint';
  import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';
  
  // Get step ID from URL
  const stepId = $page.params.slug;
  
  // Mock function to fetch step data
  function getStepData(id) {
    // Sample step data
    return {
      id: parseInt(id),
      name: `Step ${id}`,
      type: id % 2 === 0 ? 'Prompt' : 'Python',
      agentId: 1,
      agentName: 'Agent Smith',
      content: id % 2 === 0 
        ? `You are an AI assistant that helps with data analysis.
Please analyze the following data and provide insights:
{{data}}

Focus on trends, anomalies, and potential actionable insights.`
        : `import pandas as pd
import matplotlib.pyplot as plt

# Load the data
def process_data(file_path):
    df = pd.read_csv(file_path)
    
    # Clean the data
    df = df.dropna()
    
    # Perform analysis
    summary = df.describe()
    
    # Create visualization
    plt.figure(figsize=(10, 6))
    df.plot(kind='bar')
    plt.title('Data Analysis')
    plt.savefig('analysis_result.png')
    
    return summary

# Main function
if __name__ == "__main__":
    result = process_data('data.csv')
    print(result)`,
      isActive: true,
      lastEdited: '2 hours ago',
      createdAt: '2023-10-15'
    };
  }
  
  // Load step data
  let step = getStepData(stepId);
  
  // Sample agents for dropdown
  const agents = [
    { id: 1, name: 'Agent Smith' },
    { id: 2, name: 'Agent Johnson' },
    { id: 3, name: 'Agent Brown' }
  ];
  
  // Step types
  const stepTypes = ['Prompt', 'Python'];
  
  // Editor reference
  let editorElement;
  let editorView;
  
  // Improved Python linter function
  function pythonLint(view) {
    const diagnostics = [];
    const text = view.state.doc.toString();
    const lines = text.split('\n');
    
    // Check for common Python issues
    lines.forEach((line, i) => {
      const lineNum = i + 1;
      const from = view.state.doc.line(lineNum).from;
      
      // Check for print statements without parentheses (Python 2 style)
      if (/^\s*print\s+[^(]/.test(line)) {
        diagnostics.push({
          from: from + line.indexOf('print'),
          to: from + line.indexOf('print') + 5,
          severity: 'warning',
          message: 'Use print() function instead of print statement (Python 3)'
        });
      }
      
      // Check for indentation issues
      if (line.match(/^ +/) && !line.match(/^    +/) && !line.match(/^        +/) && !line.match(/^            +/)) {
        diagnostics.push({
          from,
          to: from + line.length,
          severity: 'error',
          message: 'Indentation should be multiples of 4 spaces'
        });
      }
      
      // Check for unused imports
      if (line.match(/^import\s+\w+/) && !text.includes(line.match(/import\s+(\w+)/)[1])) {
        diagnostics.push({
          from,
          to: from + line.length,
          severity: 'info',
          message: 'Potentially unused import'
        });
      }
      
      // Check for missing colons in function/class definitions
      if ((line.match(/^\s*def\s+\w+\([^)]*\)(?!\s*:)/) || line.match(/^\s*class\s+\w+(?!\s*:)/)) && !line.includes(':')) {
        diagnostics.push({
          from,
          to: from + line.length,
          severity: 'error',
          message: 'Missing colon at the end of statement'
        });
      }
      
      // Check for variables that might be undefined
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
            severity: 'warning',
            message: `Variable '${variable}' might be used before assignment`
          });
        }
      }
    });
    
    return diagnostics;
  }
  
  // Initialize editor when component mounts
  onMount(() => {
    if (step.type === 'Python') {
      initCodeEditor();
    }
    
    return () => {
      if (editorView) {
        editorView.destroy();
      }
    };
  });
  
  // Initialize CodeMirror editor with improved configuration
  function initCodeEditor() {
    if (!editorElement) return;
    
    const startState = EditorState.create({
      doc: step.content,
      extensions: [
        python(),
        syntaxHighlighting(defaultHighlightStyle),
        lintGutter(),
        linter(pythonLint),
        keymap.of(defaultKeymap),
        EditorView.lineWrapping,
        EditorView.updateListener.of(update => {
          if (update.docChanged) {
            step.content = update.state.doc.toString();
          }
        }),
        EditorState.allowMultipleSelections.of(true),
        EditorView.theme({
          "&": {
            fontSize: "14px",
            height: "100%",
            minHeight: "300px"
          },
          ".cm-scroller": {
            overflow: "auto",
            fontFamily: "monospace"
          },
          ".cm-content": {
            caretColor: "#0e9"
          },
          ".cm-activeLine": {
            backgroundColor: "rgba(0, 0, 0, 0.05)"
          },
          ".cm-activeLineGutter": {
            backgroundColor: "rgba(0, 0, 0, 0.05)"
          },
          ".cm-gutters": {
            backgroundColor: "#f8f9fa",
            color: "#999",
            border: "none",
            borderRight: "1px solid #ddd"
          }
        })
      ]
    });
    
    editorView = new EditorView({
      state: startState,
      parent: editorElement
    });
  }
  
  // Watch for changes in step type to reinitialize editor
  $: if (step.type === 'Python' && editorElement && !editorView) {
    initCodeEditor();
  } else if (step.type !== 'Python' && editorView) {
    editorView.destroy();
    editorView = null;
  }
  
  // Handle form submission
  function saveChanges() {
    // In a real app, this would send data to an API
    alert('Step settings saved!');
  }
  
  // Handle step deletion
  function deleteStep() {
    if (confirm('Are you sure you want to delete this step?')) {
      // In a real app, this would send a delete request to an API
      window.location.href = '/steps';
    }
  }
  
  // Go back to steps list
  function goBack() {
    window.location.href = '/steps';
  }
  
  // Execute the step
  function executeStep() {
    alert(`Executing ${step.type} step: ${step.name}`);
    // In a real app, this would trigger the execution of the step
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem href="/steps">Steps</BreadcrumbItem>
      <BreadcrumbItem>Step {stepId}</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex justify-between items-center">
      <div class="flex items-center gap-3">
        <Button color="light" size="sm" on:click={goBack}>
          <ArrowLeftOutline class="mr-2 h-4 w-4" />
          Back
        </Button>
        <Heading tag="h1" class="text-2xl font-bold">{step.name}</Heading>
        <Badge color={step.type === 'Python' ? 'blue' : 'purple'}>
          {step.type}
        </Badge>
      </div>
      
      <div class="flex gap-2">
        <Button color="green" on:click={executeStep}>
          <PlayOutline class="mr-2 h-5 w-5" />
          Execute
        </Button>
        <Button color="red" on:click={deleteStep}>
          <TrashBinOutline class="mr-2 h-5 w-5" />
          Delete
        </Button>
        <Button color="blue" on:click={saveChanges}>
          Save
        </Button>
      </div>
    </div>
  </div>
  
  <!-- Step Configuration -->
  <Card>
    <div class="space-y-6 py-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <Label for="name" class="mb-2">Step Name</Label>
          <Input id="name" bind:value={step.name} />
        </div>
        
        <div>
          <Label for="type" class="mb-2">Step Type</Label>
          <Select id="type" items={stepTypes} bind:value={step.type} />
        </div>
      </div>
      
      <div>
        <Label for="agent" class="mb-2">Associated Agent</Label>
        <Select id="agent">
          {#each agents as agent}
            <option value={agent.id} selected={agent.id === step.agentId}>{agent.name}</option>
          {/each}
        </Select>
      </div>
      
      <div>
        <div class="flex justify-between items-center mb-2">
          <Label for="content">
            {step.type === 'Python' ? 'Python Code' : 'Prompt Template'}
          </Label>
          <div class="flex items-center gap-2">
            <Toggle bind:checked={step.isActive} />
            <span class="text-sm">Active</span>
          </div>
        </div>
        
        {#if step.type === 'Python'}
          <!-- CodeMirror Python Editor with Linting -->
          <div 
            bind:this={editorElement} 
            class="border border-gray-300 rounded-lg min-h-[300px] font-mono"
          ></div>
        {:else}
          <!-- Regular Textarea for Prompts -->
          <Textarea 
            id="content" 
            rows="15" 
            bind:value={step.content}
            class="font-mono"
          />
        {/if}
      </div>
      
      <div>
        <p class="text-sm text-gray-500">
          Last edited: {step.lastEdited} | Created: {step.createdAt}
        </p>
      </div>
    </div>
  </Card>
</main>

<style>
  /* Additional CodeMirror styling */
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
  
  /* Syntax highlighting colors */
  :global(.cm-keyword) { color: #07a; }
  :global(.cm-def) { color: #00f; }
  :global(.cm-variable) { color: #000; }
  :global(.cm-variable-2) { color: #05a; }
  :global(.cm-string) { color: #a11; }
  :global(.cm-comment) { color: #090; }
  :global(.cm-number) { color: #905; }
  :global(.cm-operator) { color: #a67f59; }
  :global(.cm-meta) { color: #555; }
  :global(.cm-builtin) { color: #30a; }
</style>
