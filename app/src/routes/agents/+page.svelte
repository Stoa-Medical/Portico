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
    Toggle,
    Badge,
    Accordion,
    AccordionItem,
    Tabs,
    TabItem
  } from 'flowbite-svelte';
  import { PlusOutline, ArrowLeftOutline, TrashBinOutline } from 'flowbite-svelte-icons';
  
  // Sample data for agents
  let agents = [
    { id: 1, name: 'Agent Smith', status: 'Active', type: 'Assistant', lastActive: '2 hours ago', 
      description: 'This is a sample agent description that explains what this agent does and how it works.',
      settings: {
        temperature: 0.7,
        maxTokens: 2048,
        topP: 0.9,
        frequencyPenalty: 0.5,
        presencePenalty: 0.5
      },
      capabilities: ['Text Generation', 'Question Answering', 'Summarization'],
      isActive: true,
      model: 'gpt-4',
      apiKey: 'sk-••••••••••••••••••••••••',
      createdAt: '2023-10-15'
    },
    { id: 2, name: 'Agent Johnson', status: 'Idle', type: 'Researcher', lastActive: '1 day ago',
      description: 'Research agent that collects and analyzes information from various sources.',
      settings: {
        temperature: 0.5,
        maxTokens: 4096,
        topP: 0.8,
        frequencyPenalty: 0.3,
        presencePenalty: 0.3
      },
      capabilities: ['Research', 'Data Analysis', 'Summarization'],
      isActive: false,
      model: 'claude-3-opus',
      apiKey: 'sk-••••••••••••••••••••••••',
      createdAt: '2023-11-20'
    },
    { id: 3, name: 'Agent Brown', status: 'Active', type: 'Analyst', lastActive: '5 minutes ago',
      description: 'Specialized in data analysis and visualization.',
      settings: {
        temperature: 0.3,
        maxTokens: 2048,
        topP: 0.7,
        frequencyPenalty: 0.2,
        presencePenalty: 0.2
      },
      capabilities: ['Data Analysis', 'Visualization', 'Reporting'],
      isActive: true,
      model: 'gpt-3.5-turbo',
      apiKey: 'sk-••••••••••••••••••••••••',
      createdAt: '2024-01-05'
    },
  ];
  
  // Selected agent for detail view
  let selectedAgent = null;
  
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
  
  // Available models
  const models = [
    'gpt-4',
    'gpt-3.5-turbo',
    'claude-3-opus',
    'claude-3-sonnet',
    'llama-3'
  ];
  
  // Available capabilities
  const availableCapabilities = [
    'Text Generation',
    'Question Answering',
    'Summarization',
    'Translation',
    'Code Generation',
    'Data Analysis',
    'Research',
    'Visualization',
    'Reporting'
  ];
  
  // Handle form submission
  function handleSubmit() {
    // Add new agent to the list
    const id = agents.length > 0 ? Math.max(...agents.map(a => a.id)) + 1 : 1;
    
    const newAgentComplete = {
      id,
      name: newAgent.name,
      status: newAgent.isActive ? 'Active' : 'Inactive',
      type: newAgent.type,
      lastActive: 'Just now',
      description: newAgent.description,
      settings: {
        temperature: 0.7,
        maxTokens: 2048,
        topP: 0.9,
        frequencyPenalty: 0.5,
        presencePenalty: 0.5
      },
      capabilities: ['Text Generation'],
      isActive: newAgent.isActive,
      model: 'gpt-4',
      apiKey: 'sk-••••••••••••••••••••••••',
      createdAt: new Date().toISOString().split('T')[0]
    };
    
    agents = [...agents, newAgentComplete];
    
    // Reset form and close modal
    resetForm();
    showModal = false;
    
    // Select the newly created agent
    selectedAgent = newAgentComplete;
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
  
  // Select an agent to view details
  function selectAgent(agent) {
    selectedAgent = agent;
  }
  
  // Go back to list view
  function backToList() {
    selectedAgent = null;
  }
  
  // Handle agent deletion
  function deleteAgent() {
    if (confirm('Are you sure you want to delete this agent?')) {
      agents = agents.filter(a => a.id !== selectedAgent.id);
      selectedAgent = null;
    }
  }
  
  // Save changes to agent
  function saveChanges() {
    // In a real app, this would send data to an API
    agents = agents.map(a => a.id === selectedAgent.id ? selectedAgent : a);
    alert('Agent settings saved!');
  }
  
  // Toggle capability selection
  function toggleCapability(capability) {
    if (selectedAgent.capabilities.includes(capability)) {
      selectedAgent.capabilities = selectedAgent.capabilities.filter(c => c !== capability);
    } else {
      selectedAgent.capabilities = [...selectedAgent.capabilities, capability];
    }
  }
  
  // Mock function to get steps associated with this agent
  function getAgentSteps(agentId) {
    return [
      { id: 1, name: 'Data Collection', type: 'Python', lastEdited: '2 hours ago' },
      { id: 2, name: 'Text Analysis', type: 'Prompt', lastEdited: '1 day ago' },
      { id: 3, name: 'Data Visualization', type: 'Python', lastEdited: '3 days ago' },
    ].filter(step => step.id % agentId === 0 || step.id === agentId);
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem>Agents</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
      <Heading tag="h1" class="text-2xl font-bold">Agents</Heading>
      {#if !selectedAgent}
        <Button class="self-start" color="blue" on:click={() => showModal = true}>
          <PlusOutline class="mr-2 h-5 w-5" />
          Add Agent
        </Button>
      {:else}
        <div class="flex flex-wrap gap-2">
          <Button color="red" on:click={deleteAgent}>
            <TrashBinOutline class="mr-2 h-5 w-5" />
            Delete
          </Button>
          <Button color="blue" on:click={saveChanges}>
            Save Changes
          </Button>
        </div>
      {/if}
    </div>
  </div>
  
  <!-- Master-Detail View -->
  <div class="grid grid-cols-1 {selectedAgent ? 'lg:grid-cols-3 gap-6' : ''}">
    <!-- Agents List (Master View) -->
    <div class="{selectedAgent ? 'hidden lg:block' : 'block'}">
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
                on:click={() => selectAgent(agent)}
                class="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 {selectedAgent?.id === agent.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''}"
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
    </div>
    
    <!-- Agent Details (Detail View) -->
    {#if selectedAgent}
      <div class="col-span-1 lg:col-span-2">
        <Card>
          <div class="mb-4 flex items-center gap-3">
            <Button color="light" size="sm" class="lg:hidden" on:click={backToList}>
              <ArrowLeftOutline class="mr-2 h-4 w-4" />
              Back
            </Button>
            <Heading tag="h2" class="text-xl font-bold">{selectedAgent.name}</Heading>
            <Badge color={selectedAgent.isActive ? 'green' : 'gray'}>
              {selectedAgent.status}
            </Badge>
          </div>
          
          <Tabs style="underline">
            <TabItem open title="General">
              <div class="space-y-6 py-4">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div>
                    <Label for="name" class="mb-2">Agent Name</Label>
                    <Input id="name" bind:value={selectedAgent.name} />
                  </div>
                  
                  <div>
                    <Label for="type" class="mb-2">Agent Type</Label>
                    <Select id="type" items={agentTypes} bind:value={selectedAgent.type} />
                  </div>
                </div>
                
                <div>
                  <Label for="description" class="mb-2">Description</Label>
                  <Textarea id="description" rows="4" bind:value={selectedAgent.description} />
                </div>
                
                <div class="flex items-center gap-2">
                  <Toggle bind:checked={selectedAgent.isActive} />
                  <Label>Active Status</Label>
                </div>
                
                <div>
                  <Label class="mb-2">Created On</Label>
                  <p class="text-gray-700 dark:text-gray-300">{selectedAgent.createdAt}</p>
                </div>
              </div>
            </TabItem>
            
            <TabItem title="Model Settings">
              <div class="space-y-6 py-4">
                <div>
                  <Label for="model" class="mb-2">AI Model</Label>
                  <Select id="model" items={models} bind:value={selectedAgent.model} />
                </div>
                
                <div>
                  <Label for="apiKey" class="mb-2">API Key</Label>
                  <Input id="apiKey" type="password" bind:value={selectedAgent.apiKey} />
                </div>
                
                <Accordion>
                  <AccordionItem>
                    <span slot="header">Advanced Settings</span>
                    <div class="space-y-4 pt-2">
                      <div>
                        <Label for="temperature" class="mb-2">Temperature: {selectedAgent.settings.temperature}</Label>
                        <Input id="temperature" type="range" min="0" max="1" step="0.1" bind:value={selectedAgent.settings.temperature} />
                      </div>
                      
                      <div>
                        <Label for="maxTokens" class="mb-2">Max Tokens: {selectedAgent.settings.maxTokens}</Label>
                        <Input id="maxTokens" type="range" min="256" max="4096" step="256" bind:value={selectedAgent.settings.maxTokens} />
                      </div>
                      
                      <div>
                        <Label for="topP" class="mb-2">Top P: {selectedAgent.settings.topP}</Label>
                        <Input id="topP" type="range" min="0" max="1" step="0.1" bind:value={selectedAgent.settings.topP} />
                      </div>
                      
                      <div>
                        <Label for="frequencyPenalty" class="mb-2">Frequency Penalty: {selectedAgent.settings.frequencyPenalty}</Label>
                        <Input id="frequencyPenalty" type="range" min="0" max="2" step="0.1" bind:value={selectedAgent.settings.frequencyPenalty} />
                      </div>
                      
                      <div>
                        <Label for="presencePenalty" class="mb-2">Presence Penalty: {selectedAgent.settings.presencePenalty}</Label>
                        <Input id="presencePenalty" type="range" min="0" max="2" step="0.1" bind:value={selectedAgent.settings.presencePenalty} />
                      </div>
                    </div>
                  </AccordionItem>
                </Accordion>
              </div>
            </TabItem>
            
            <TabItem title="Capabilities">
              <div class="space-y-6 py-4">
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                  Select the capabilities this agent should have:
                </p>
                
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {#each availableCapabilities as capability}
                    <div class="flex items-center gap-2">
                      <Checkbox 
                        id={`capability-${capability}`} 
                        checked={selectedAgent.capabilities.includes(capability)}
                        on:change={() => toggleCapability(capability)}
                      />
                      <Label for={`capability-${capability}`}>{capability}</Label>
                    </div>
                  {/each}
                </div>
              </div>
            </TabItem>
            
            <TabItem title="Steps">
              <div class="space-y-6 py-4">
                <div class="flex justify-between items-center mb-4">
                  <p class="text-gray-700 dark:text-gray-300">
                    Steps define the workflow for this agent. Each step can be a Python script or a prompt template.
                  </p>
                  <Button color="blue" href={`/steps/new?agentId=${selectedAgent.id}&agentName=${encodeURIComponent(selectedAgent.name)}`}>
                    <PlusOutline class="mr-2 h-5 w-5" />
                    Add Step
                  </Button>
                </div>
                
                {#if getAgentSteps(selectedAgent.id).length > 0}
                  <Table hoverable={true}>
                    <TableHead>
                      <TableHeadCell>Name</TableHeadCell>
                      <TableHeadCell>Type</TableHeadCell>
                      <TableHeadCell>Last Edited</TableHeadCell>
                      <TableHeadCell>Actions</TableHeadCell>
                    </TableHead>
                    <TableBody>
                      {#each getAgentSteps(selectedAgent.id) as step}
                        <TableBodyRow>
                          <TableBodyCell>{step.name}</TableBodyCell>
                          <TableBodyCell>
                            <Badge color={step.type === 'Python' ? 'blue' : 'purple'}>
                              {step.type}
                            </Badge>
                          </TableBodyCell>
                          <TableBodyCell>{step.lastEdited}</TableBodyCell>
                          <TableBodyCell>
                            <div class="flex gap-2">
                              <Button size="xs" color="light" href={`/steps/${step.id}`}>
                                View
                              </Button>
                            </div>
                          </TableBodyCell>
                        </TableBodyRow>
                      {/each}
                    </TableBody>
                  </Table>
                {:else}
                  <div class="text-center py-8 border rounded-lg bg-gray-50 dark:bg-gray-800">
                    <p class="text-gray-500 dark:text-gray-400 mb-4">No steps found for this agent</p>
                    <Button color="blue" href={`/steps/new?agentId=${selectedAgent.id}&agentName=${encodeURIComponent(selectedAgent.name)}`}>
                      <PlusOutline class="mr-2 h-5 w-5" />
                      Create First Step
                    </Button>
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
