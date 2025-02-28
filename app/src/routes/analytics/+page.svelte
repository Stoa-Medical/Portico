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
    Badge,
    Tabs,
    TabItem,
    Select,
    Dropdown,
    DropdownItem,
    DropdownDivider
  } from 'flowbite-svelte';
  import { ChartPieOutline, ChartBars3FromLeftOutline, ChartLineUpOutline, CalendarMonthOutline } from 'flowbite-svelte-icons';
  import { onMount } from 'svelte';
  
  // Time period options
  const timePeriods = [
    { value: '7d', name: 'Last 7 days' },
    { value: '30d', name: 'Last 30 days' },
    { value: '90d', name: 'Last 90 days' },
    { value: 'all', name: 'All time' }
  ];
  
  let selectedTimePeriod = '30d';
  
  // Mock data for agent performance
  const agentPerformance = [
    { 
      id: 1, 
      name: 'Agent Smith', 
      successRate: 92, 
      totalRuns: 245, 
      avgResponseTime: '1.2s',
      trend: 'up'
    },
    { 
      id: 2, 
      name: 'Agent Johnson', 
      successRate: 78, 
      totalRuns: 156, 
      avgResponseTime: '2.5s',
      trend: 'down'
    },
    { 
      id: 3, 
      name: 'Agent Brown', 
      successRate: 95, 
      totalRuns: 312, 
      avgResponseTime: '0.8s',
      trend: 'up'
    }
  ];
  
  // Mock data for step performance
  const stepPerformance = [
    { 
      id: 1, 
      name: 'Data Collection', 
      type: 'Python', 
      successRate: 98, 
      totalRuns: 412, 
      avgExecutionTime: '0.5s',
      agentName: 'Agent Smith'
    },
    { 
      id: 2, 
      name: 'Text Analysis', 
      type: 'Prompt', 
      successRate: 85, 
      totalRuns: 245, 
      avgExecutionTime: '2.1s',
      agentName: 'Agent Smith'
    },
    { 
      id: 3, 
      name: 'Data Visualization', 
      type: 'Python', 
      successRate: 92, 
      totalRuns: 178, 
      avgExecutionTime: '1.3s',
      agentName: 'Agent Brown'
    }
  ];
  
  // Mock data for charts
  let successRateChartData;
  let executionTimeChartData;
  let usageChartData;
  let errorDistributionData;
  
  // Chart rendering functions
  function renderSuccessRateChart() {
    // In a real app, this would use a charting library like Chart.js
    // For now, we'll just create a mock chart element
    const chartElement = document.getElementById('success-rate-chart');
    if (chartElement) {
      chartElement.innerHTML = `
        <div class="flex items-end h-40 gap-2">
          <div class="bg-green-500 w-10 h-[92%] rounded-t-md relative">
            <span class="absolute -top-6 left-0 text-xs">92%</span>
          </div>
          <div class="bg-yellow-500 w-10 h-[78%] rounded-t-md relative">
            <span class="absolute -top-6 left-0 text-xs">78%</span>
          </div>
          <div class="bg-green-600 w-10 h-[95%] rounded-t-md relative">
            <span class="absolute -top-6 left-0 text-xs">95%</span>
          </div>
        </div>
        <div class="flex justify-between mt-2 text-xs text-gray-500">
          <div>Agent Smith</div>
          <div>Agent Johnson</div>
          <div>Agent Brown</div>
        </div>
      `;
    }
  }
  
  function renderExecutionTimeChart() {
    const chartElement = document.getElementById('execution-time-chart');
    if (chartElement) {
      chartElement.innerHTML = `
        <div class="flex items-end h-40 gap-2">
          <div class="bg-blue-500 w-10 h-[60%] rounded-t-md relative">
            <span class="absolute -top-6 left-0 text-xs">1.2s</span>
          </div>
          <div class="bg-blue-500 w-10 h-[80%] rounded-t-md relative">
            <span class="absolute -top-6 left-0 text-xs">2.5s</span>
          </div>
          <div class="bg-blue-500 w-10 h-[40%] rounded-t-md relative">
            <span class="absolute -top-6 left-0 text-xs">0.8s</span>
          </div>
        </div>
        <div class="flex justify-between mt-2 text-xs text-gray-500">
          <div>Agent Smith</div>
          <div>Agent Johnson</div>
          <div>Agent Brown</div>
        </div>
      `;
    }
  }
  
  function renderUsageChart() {
    const chartElement = document.getElementById('usage-chart');
    if (chartElement) {
      chartElement.innerHTML = `
        <div class="h-40 w-full relative">
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="text-center">
              <div class="text-3xl font-bold">713</div>
              <div class="text-sm text-gray-500">Total Runs</div>
            </div>
          </div>
          <svg viewBox="0 0 100 100" class="h-full w-full">
            <circle cx="50" cy="50" r="40" fill="none" stroke="#e5e7eb" stroke-width="10" />
            <circle cx="50" cy="50" r="40" fill="none" stroke="#3b82f6" stroke-width="10" 
              stroke-dasharray="251.2" stroke-dashoffset="50.24" transform="rotate(-90 50 50)" />
          </svg>
        </div>
      `;
    }
  }
  
  function renderErrorDistributionChart() {
    const chartElement = document.getElementById('error-distribution-chart');
    if (chartElement) {
      chartElement.innerHTML = `
        <div class="h-40 w-full">
          <div class="flex h-full">
            <div class="bg-red-500 w-[25%] h-full relative flex items-center justify-center">
              <span class="text-white text-xs">API Errors</span>
            </div>
            <div class="bg-orange-500 w-[15%] h-full relative flex items-center justify-center">
              <span class="text-white text-xs">Timeout</span>
            </div>
            <div class="bg-yellow-500 w-[35%] h-full relative flex items-center justify-center">
              <span class="text-white text-xs">Input Errors</span>
            </div>
            <div class="bg-red-300 w-[25%] h-full relative flex items-center justify-center">
              <span class="text-white text-xs">Other</span>
            </div>
          </div>
        </div>
      `;
    }
  }
  
  // Initialize charts on mount
  onMount(() => {
    renderSuccessRateChart();
    renderExecutionTimeChart();
    renderUsageChart();
    renderErrorDistributionChart();
  });
  
  // Update charts when time period changes
  $: if (selectedTimePeriod) {
    // In a real app, this would fetch new data based on the time period
    setTimeout(() => {
      renderSuccessRateChart();
      renderExecutionTimeChart();
      renderUsageChart();
      renderErrorDistributionChart();
    }, 0);
  }
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <div class="mb-6">
    <Breadcrumb class="mb-4">
      <BreadcrumbItem href="/" home>Home</BreadcrumbItem>
      <BreadcrumbItem>Analytics</BreadcrumbItem>
    </Breadcrumb>
    
    <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
      <Heading tag="h1" class="text-2xl font-bold">Analytics Dashboard</Heading>
      
      <div class="flex items-center gap-2">
        <CalendarMonthOutline class="h-5 w-5 text-gray-500" />
        <Select class="w-40" bind:value={selectedTimePeriod}>
          {#each timePeriods as period}
            <option value={period.value}>{period.name}</option>
          {/each}
        </Select>
      </div>
    </div>
  </div>
  
  <!-- Summary Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Agents</div>
        <div class="text-2xl font-bold">3</div>
        <div class="text-green-500 text-sm mt-2">+1 this month</div>
      </div>
    </Card>
    
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Steps</div>
        <div class="text-2xl font-bold">8</div>
        <div class="text-green-500 text-sm mt-2">+3 this month</div>
      </div>
    </Card>
    
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Avg. Success Rate</div>
        <div class="text-2xl font-bold">88%</div>
        <div class="text-green-500 text-sm mt-2">+5% from last month</div>
      </div>
    </Card>
    
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Executions</div>
        <div class="text-2xl font-bold">713</div>
        <div class="text-green-500 text-sm mt-2">+142 this month</div>
      </div>
    </Card>
  </div>
  
  <!-- Charts Section -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
    <Card>
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold">Success Rate by Agent</Heading>
          <ChartBars3FromLeftOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="success-rate-chart" class="w-full"></div>
      </div>
    </Card>
    
    <Card>
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold">Avg. Response Time</Heading>
          <ChartLineUpOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="execution-time-chart" class="w-full"></div>
      </div>
    </Card>
    
    <Card>
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold">Usage Statistics</Heading>
          <ChartPieOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="usage-chart" class="w-full"></div>
      </div>
    </Card>
    
    <Card>
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold">Error Distribution</Heading>
          <ChartBars3FromLeftOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="error-distribution-chart" class="w-full"></div>
      </div>
    </Card>
  </div>
  
  <!-- Performance Tables -->
  <Tabs style="underline">
    <TabItem open title="Agent Performance">
      <Card>
        <Table hoverable={true}>
          <TableHead>
            <TableHeadCell>Agent Name</TableHeadCell>
            <TableHeadCell>Success Rate</TableHeadCell>
            <TableHeadCell>Total Runs</TableHeadCell>
            <TableHeadCell>Avg. Response Time</TableHeadCell>
            <TableHeadCell>Trend</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each agentPerformance as agent}
              <TableBodyRow>
                <TableBodyCell>{agent.name}</TableBodyCell>
                <TableBodyCell>
                  <div class="flex items-center">
                    <div class="w-16 bg-gray-200 rounded-full h-2.5 mr-2">
                      <div class="bg-{agent.successRate >= 90 ? 'green' : agent.successRate >= 70 ? 'yellow' : 'red'}-500 h-2.5 rounded-full" style="width: {agent.successRate}%"></div>
                    </div>
                    <span>{agent.successRate}%</span>
                  </div>
                </TableBodyCell>
                <TableBodyCell>{agent.totalRuns}</TableBodyCell>
                <TableBodyCell>{agent.avgResponseTime}</TableBodyCell>
                <TableBodyCell>
                  <Badge color={agent.trend === 'up' ? 'green' : 'red'}>
                    {agent.trend === 'up' ? '↑' : '↓'}
                  </Badge>
                </TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    </TabItem>
    
    <TabItem title="Step Performance">
      <Card>
        <Table hoverable={true}>
          <TableHead>
            <TableHeadCell>Step Name</TableHeadCell>
            <TableHeadCell>Type</TableHeadCell>
            <TableHeadCell>Agent</TableHeadCell>
            <TableHeadCell>Success Rate</TableHeadCell>
            <TableHeadCell>Total Runs</TableHeadCell>
            <TableHeadCell>Avg. Execution Time</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each stepPerformance as step}
              <TableBodyRow>
                <TableBodyCell>{step.name}</TableBodyCell>
                <TableBodyCell>
                  <Badge color={step.type === 'Python' ? 'blue' : 'purple'}>
                    {step.type}
                  </Badge>
                </TableBodyCell>
                <TableBodyCell>{step.agentName}</TableBodyCell>
                <TableBodyCell>
                  <div class="flex items-center">
                    <div class="w-16 bg-gray-200 rounded-full h-2.5 mr-2">
                      <div class="bg-{step.successRate >= 90 ? 'green' : step.successRate >= 70 ? 'yellow' : 'red'}-500 h-2.5 rounded-full" style="width: {step.successRate}%"></div>
                    </div>
                    <span>{step.successRate}%</span>
                  </div>
                </TableBodyCell>
                <TableBodyCell>{step.totalRuns}</TableBodyCell>
                <TableBodyCell>{step.avgExecutionTime}</TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    </TabItem>
  </Tabs>
</main> 