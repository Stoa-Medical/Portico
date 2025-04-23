<script>
  import {
    Card,
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
  } from "flowbite-svelte";
  import {
    ChartPieOutline,
    ChartBars3FromLeftOutline,
    ChartLineUpOutline,
    CalendarMonthOutline,
  } from "flowbite-svelte-icons";
  import { getAnalyticsCounts } from "./api";
  import { onMount } from "svelte";

  // Time period options
  const timePeriods = [
    { value: "7d", name: "Last 7 days" },
    { value: "30d", name: "Last 30 days" },
    { value: "90d", name: "Last 90 days" },
  ];

  let selectedTimePeriod = "30d";
  let agentCount = 0;
  let runtimeSessionCount = 0;

  // Mock data for agent performance
  const agentPerformance = [
    {
      id: 1,
      name: "Agent Smith",
      successRate: 92,
      totalRuns: 245,
      avgResponseTime: "1.2s",
      trend: "up",
    },
    {
      id: 2,
      name: "Agent Johnson",
      successRate: 78,
      totalRuns: 156,
      avgResponseTime: "2.5s",
      trend: "down",
    },
    {
      id: 3,
      name: "Agent Brown",
      successRate: 95,
      totalRuns: 312,
      avgResponseTime: "0.8s",
      trend: "up",
    },
  ];

  // Mock data for step performance
  const stepPerformance = [
    {
      id: 1,
      name: "Data Collection",
      type: "Python",
      successRate: 98,
      totalRuns: 412,
      avgExecutionTime: "0.5s",
      agentName: "Agent Smith",
    },
    {
      id: 2,
      name: "Text Analysis",
      type: "Prompt",
      successRate: 85,
      totalRuns: 245,
      avgExecutionTime: "2.1s",
      agentName: "Agent Smith",
    },
    {
      id: 3,
      name: "Data Visualization",
      type: "Python",
      successRate: 92,
      totalRuns: 178,
      avgExecutionTime: "1.3s",
      agentName: "Agent Brown",
    },
  ];

  // Chart rendering functions
  function renderSuccessRateChart() {
    const chartElement = document.getElementById("success-rate-chart");
    if (!chartElement) return;

    const agents = [
      { name: "Agent Smith", percent: 92, color: "bg-green-500" },
      { name: "Agent Johnson", percent: 78, color: "bg-yellow-500" },
      { name: "Agent Brown", percent: 95, color: "bg-green-600" },
    ];

    chartElement.innerHTML = `
    <div class="flex justify-around items-end h-48 w-full">
      ${agents
        .map(
          (a) => `
        <div class="flex flex-col items-center w-1/4">
          <div class="relative h-full flex items-end">
            <div class="${a.color} w-10 rounded-t-md transition-all duration-300"
                 style="height: ${a.percent}%; min-height: 2rem;">
              <span class="absolute -top-5 left-1/2 -translate-x-1/2 text-xs text-white">${a.percent}%</span>
            </div>
          </div>
          <div class="text-xs mt-2 text-center text-gray-400">${a.name}</div>
        </div>`,
        )
        .join("")}
    </div>
  `;
  }

  function renderExecutionTimeChart() {
    const chartElement = document.getElementById("execution-time-chart");
    if (!chartElement) return;

    const agents = [
      { name: "Agent Smith", value: "1.2s", height: 60 },
      { name: "Agent Johnson", value: "2.5s", height: 80 },
      { name: "Agent Brown", value: "0.8s", height: 40 },
    ];

    chartElement.innerHTML = `
    <div class="flex justify-around items-end h-48 w-full">
      ${agents
        .map(
          (a) => `
        <div class="flex flex-col items-center w-1/4">
          <div class="relative h-full flex items-end">
            <div class="bg-blue-500 w-10 rounded-t-md transition-all duration-300"
                 style="height: ${a.height}%; min-height: 2rem;">
              <span class="absolute -top-5 left-1/2 -translate-x-1/2 text-xs text-white">${a.value}</span>
            </div>
          </div>
          <div class="text-xs mt-2 text-center text-gray-400">${a.name}</div>
        </div>`,
        )
        .join("")}
    </div>
  `;
  }

  function renderUsageChart() {
    const chartElement = document.getElementById("usage-chart");
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
    const chartElement = document.getElementById("error-distribution-chart");
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
  onMount(async () => {
    renderSuccessRateChart();
    renderExecutionTimeChart();
    renderUsageChart();
    renderErrorDistributionChart();

    try {
      const counts = await getAnalyticsCounts();
      agentCount = counts.agentCount;
      runtimeSessionCount = counts.runtimeSessionCount;
    } catch (err) {
      console.error("Failed to load analytics counts:", err);
    }
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

    <div
      class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4"
    >
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
        <div class="text-2xl font-bold">{agentCount}</div>
      </div>
    </Card>

    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Steps</div>
        <div class="text-2xl font-bold">8</div>
      </div>
    </Card>

    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Avg. Success Rate</div>
        <div class="text-2xl font-bold">88%</div>
      </div>
    </Card>

    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Executions</div>
        <div class="text-2xl font-bold">{runtimeSessionCount}</div>
      </div>
    </Card>
  </div>

  <!-- Charts Section -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
    <Card class="max-w-full">
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold"
            >Success Rate by Agent</Heading
          >
          <ChartBars3FromLeftOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="success-rate-chart" class="w-full"></div>
      </div>
    </Card>

    <Card class="max-w-full">
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold"
            >Avg. Response Time</Heading
          >
          <ChartLineUpOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="execution-time-chart" class="w-full"></div>
      </div>
    </Card>

    <Card class="max-w-full">
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold"
            >Usage Statistics</Heading
          >
          <ChartPieOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="usage-chart" class="w-full"></div>
      </div>
    </Card>

    <Card class="max-w-full">
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold"
            >Error Distribution</Heading
          >
          <ChartBars3FromLeftOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="error-distribution-chart" class="w-full"></div>
      </div>
    </Card>
  </div>

  <!-- Performance Tables -->
  <Tabs style="underline">
    <TabItem open title="Agent Performance">
      <Card class="max-w-full">
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
                      <div
                        class="bg-{agent.successRate >= 90
                          ? 'green'
                          : agent.successRate >= 70
                            ? 'yellow'
                            : 'red'}-500 h-2.5 rounded-full"
                        style="width: {agent.successRate}%"
                      ></div>
                    </div>
                    <span>{agent.successRate}%</span>
                  </div>
                </TableBodyCell>
                <TableBodyCell>{agent.totalRuns}</TableBodyCell>
                <TableBodyCell>{agent.avgResponseTime}</TableBodyCell>
                <TableBodyCell>
                  <Badge color={agent.trend === "up" ? "green" : "red"}>
                    {agent.trend === "up" ? "↑" : "↓"}
                  </Badge>
                </TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    </TabItem>

    <TabItem title="Step Performance">
      <Card class="max-w-full">
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
                  <Badge color={step.type === "Python" ? "blue" : "purple"}>
                    {step.type}
                  </Badge>
                </TableBodyCell>
                <TableBodyCell>{step.agentName}</TableBodyCell>
                <TableBodyCell>
                  <div class="flex items-center">
                    <div class="w-16 bg-gray-200 rounded-full h-2.5 mr-2">
                      <div
                        class="bg-{step.successRate >= 90
                          ? 'green'
                          : step.successRate >= 70
                            ? 'yellow'
                            : 'red'}-500 h-2.5 rounded-full"
                        style="width: {step.successRate}%"
                      ></div>
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
