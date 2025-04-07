<script>
  import { 
    Card, 
    Button, 
    Heading, 
    Breadcrumb, 
    BreadcrumbItem,
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
    Checkbox,
    Badge
  } from 'flowbite-svelte';
  import { PlusOutline } from 'flowbite-svelte-icons';
  
  // Sample data for steps
  let steps = [
    { id: 1, name: 'Data Collection', type: 'Python', agentId: 1, agentName: 'Agent Smith', lastEdited: '2 hours ago' },
    { id: 2, name: 'Text Analysis', type: 'Prompt', agentId: 1, agentName: 'Agent Smith', lastEdited: '1 day ago' },
    { id: 3, name: 'Data Visualization', type: 'Python', agentId: 2, agentName: 'Agent Johnson', lastEdited: '3 days ago' },
  ];
  
  // Modal state
  let showModal = false;
  
  // Form data for new step
  let newStep = {
    name: '',
    type: 'Prompt',
    agentId: '',
    content: ''
  };
  
  // Step types
  const stepTypes = ['Prompt', 'Python'];
  
  // Sample agents for dropdown
  const agents = [
    { id: 1, name: 'Agent Smith' },
    { id: 2, name: 'Agent Johnson' },
    { id: 3, name: 'Agent Brown' }
  ];
  
  // Handle form submission
  function handleSubmit() {
    // Add new step to the list
    const id = steps.length > 0 ? Math.max(...steps.map(s => s.id)) + 1 : 1;
    const agentName = agents.find(a => a.id === parseInt(newStep.agentId))?.name || 'Unknown Agent';
    
    steps = [
      ...steps, 
      {
        id,
        name: newStep.name,
        type: newStep.type,
        agentId: parseInt(newStep.agentId),
        agentName,
        lastEdited: 'Just now'
      }
    ];
    
    // Reset form and close modal
    resetForm();
    showModal = false;
  }
  
  // Reset form fields
  function resetForm() {
    newStep = {
      name: '',
      type: 'Prompt',
      agentId: '',
      content: ''
    };
  }
  
  // Navigate to step details
  function navigateToStep(id) {
    window.location.href = `/steps/${id}`;
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem>Steps</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
      <Heading tag="h1" class="text-2xl font-bold">Steps</Heading>
      <Button class="self-start" color="blue" on:click={() => showModal = true}>
        <PlusOutline class="mr-2 h-5 w-5" />
        Add Step
      </Button>
    </div>
  </div>
  
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
              <Badge color={step.type === 'Python' ? 'blue' : 'purple'}>
                {step.type}
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
        <Input id="name" placeholder="Enter step name" required bind:value={newStep.name} />
      </div>
      
      <div>
        <Label for="type" class="mb-2">Step Type</Label>
        <Select id="type" items={stepTypes} bind:value={newStep.type} />
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
          placeholder={newStep.type === 'Python' ? '# Enter Python code here' : 'Enter prompt text here'} 
          rows="5" 
          bind:value={newStep.content} 
        />
      </div>
      
      <div class="flex justify-end gap-4">
        <Button color="alternative" on:click={() => { showModal = false; resetForm(); }}>Cancel</Button>
        <Button type="submit" color="blue">Add Step</Button>
      </div>
    </form>
  </Modal>
</main>
