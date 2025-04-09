<script>
  import { page } from '$app/stores';
  import { 
    Button, 
    Heading, 
    Breadcrumb, 
    BreadcrumbItem,
    Badge,
  } from 'flowbite-svelte';
  import { ArrowLeftOutline, TrashBinOutline, PlayOutline } from 'flowbite-svelte-icons';
  import StepConfig from '$lib/components/StepConfig.svelte';
  
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
    
    <div class="flex flex-col sm:flex-row gap-4 mb-4">
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
    </div>
    <div class="flex flex-wrap gap-2 mb-6">
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
  
  <!-- Step Configuration -->
  <StepConfig bind:step={step} {stepTypes} {agents} />
</main>
