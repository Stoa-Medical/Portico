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
    Checkbox
  } from 'flowbite-svelte';
  import { PlusOutline } from 'flowbite-svelte-icons';
  
  // Sample data for agents
  let agents = [
    { id: 1, name: 'Agent Smith', status: 'Active', type: 'Assistant', lastActive: '2 hours ago' },
    { id: 2, name: 'Agent Johnson', status: 'Idle', type: 'Researcher', lastActive: '1 day ago' },
    { id: 3, name: 'Agent Brown', status: 'Active', type: 'Analyst', lastActive: '5 minutes ago' },
  ];
  
  // Modal state
  let showModal = false;
  
  // Form data for new agent
  let newAgent = {
    name: '',
    type: 'Assistant',
    description: '',
    isActive: true
  };
  
  // Agent types for dropdown
  const agentTypes = ['Assistant', 'Researcher', 'Analyst', 'Custom'];
  
  // Handle form submission
  function handleSubmit() {
    // Add new agent to the list
    const id = agents.length > 0 ? Math.max(...agents.map(a => a.id)) + 1 : 1;
    
    agents = [
      ...agents, 
      {
        id,
        name: newAgent.name,
        status: newAgent.isActive ? 'Active' : 'Inactive',
        type: newAgent.type,
        lastActive: 'Just now'
      }
    ];
    
    // Reset form and close modal
    resetForm();
    showModal = false;
  }
  
  // Reset form fields
  function resetForm() {
    newAgent = {
      name: '',
      type: 'Assistant',
      description: '',
      isActive: true
    };
  }
  
  // Navigate to agent details
  function navigateToAgent(id) {
    window.location.href = `/agents/${id}`;
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem>Agents</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex justify-between items-center">
      <Heading tag="h1" class="text-2xl font-bold">Agents</Heading>
      <Button color="blue" on:click={() => showModal = true}>
        <PlusOutline class="mr-2 h-5 w-5" />
        Add Agent
      </Button>
    </div>
  </div>
  
  <!-- Agents List -->
  <Card>
    <Table hoverable={true}>
      <TableHead>
        <TableHeadCell>Name</TableHeadCell>
        <TableHeadCell>Type</TableHeadCell>
        <TableHeadCell>Status</TableHeadCell>
        <TableHeadCell>Last Active</TableHeadCell>
      </TableHead>
      <TableBody>
        {#each agents as agent}
          <TableBodyRow 
            on:click={() => navigateToAgent(agent.id)}
            class="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            <TableBodyCell>{agent.name}</TableBodyCell>
            <TableBodyCell>{agent.type}</TableBodyCell>
            <TableBodyCell>
              <span class={agent.status === 'Active' ? 'text-green-500' : 'text-gray-500'}>
                {agent.status}
              </span>
            </TableBodyCell>
            <TableBodyCell>{agent.lastActive}</TableBodyCell>
          </TableBodyRow>
        {/each}
        
        {#if agents.length === 0}
          <TableBodyRow>
            <TableBodyCell colspan="4" class="text-center py-4 text-gray-500">
              No agents found. Click "Add Agent" to create one.
            </TableBodyCell>
          </TableBodyRow>
        {/if}
      </TableBody>
    </Table>
  </Card>
  
  <!-- Add Agent Modal -->
  <Modal title="Add New Agent" bind:open={showModal} autoclose>
    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
      <div>
        <Label for="name" class="mb-2">Agent Name</Label>
        <Input id="name" placeholder="Enter agent name" required bind:value={newAgent.name} />
      </div>
      
      <div>
        <Label for="type" class="mb-2">Agent Type</Label>
        <Select id="type" items={agentTypes} bind:value={newAgent.type} />
      </div>
      
      <div>
        <Label for="description" class="mb-2">Description</Label>
        <Textarea id="description" placeholder="Enter agent description" rows="3" bind:value={newAgent.description} />
      </div>
      
      <div class="flex items-center gap-2">
        <Checkbox id="isActive" bind:checked={newAgent.isActive} />
        <Label for="isActive">Active</Label>
      </div>
      
      <div class="flex justify-end gap-4">
        <Button color="alternative" on:click={() => { showModal = false; resetForm(); }}>Cancel</Button>
        <Button type="submit" color="blue">Add Agent</Button>
      </div>
    </form>
  </Modal>
</main>
