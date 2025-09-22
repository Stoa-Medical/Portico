<script lang="ts">
  import { PageHeader, SuccessChart, ResponseTimeChart } from "$lib/components";
  import {
    Card,
    Heading,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Tabs,
    TabItem,
  } from "flowbite-svelte";
  import {
    ChartPieOutline,
    ChartBars3FromLeftOutline,
    ChartLineUpOutline,
    CalendarMonthOutline,
  } from "flowbite-svelte-icons";
  import {
    getAnalyticsCounts,
    getWorkflowPerformance,
    getErrorDistribution,
  } from "./api";
  import { onMount } from "svelte";

  const breadcrumbs = [
    { label: "Home", url: "/" },
    { label: "Analytics", url: "/analytics" },
  ];

  const timePeriods = [
    { value: "7d", name: "Last 7 days" },
    { value: "30d", name: "Last 30 days" },
    { value: "90d", name: "Last 90 days" },
  ];

  let selectedTimePeriod = "30d";
  let agentCount = 0;
  let workflowCount = 0;
  let runtimeSessionCount = 0;
  let stepCount = 0;
  let workflowPerformance: Array<{
    workflowId: number;
    workflowName: string;
    successRate: number;
    totalRuns: number;
    avgResponseTime: string;
  }> = [];

  let errorDistribution = {
    completed: 0,
    cancelled: 0,
    running: 0,
    waiting: 0,
  };

  function renderUsageChart() {
    const el = document.getElementById("usage-chart");
    if (!el) return;
    el.innerHTML = `
      <div class="h-40 w-full relative">
        <div class="absolute inset-0 flex items-center justify-center">
          <div class="text-center">
            <div class="text-3xl font-bold">${workflowPerformance.reduce((a, b) => a + b.totalRuns, 0)}</div>
            <div class="text-sm text-gray-500">Total Workflow Runs</div>
          </div>
        </div>
      </div>
    `;
  }

  function renderErrorDistributionChart() {
    const el = document.getElementById("error-distribution-chart");
    if (!el || !errorDistribution) return;

    const total =
      errorDistribution.completed +
      errorDistribution.cancelled +
      errorDistribution.running +
      errorDistribution.waiting;

    if (total === 0) {
      el.innerHTML = `<div class="h-40 flex items-center justify-center text-sm text-gray-400">No data available</div>`;
      return;
    }

    const segments = [
      {
        label: "Completed",
        value: errorDistribution.completed,
        color: "bg-green-500",
      },
      {
        label: "Cancelled",
        value: errorDistribution.cancelled,
        color: "bg-red-500",
      },
      {
        label: "Running",
        value: errorDistribution.running,
        color: "bg-yellow-500",
      },
      {
        label: "Waiting",
        value: errorDistribution.waiting,
        color: "bg-gray-400",
      },
    ];

    const bars = segments
      .map(({ label, value, color }) => {
        const percent = ((value / total) * 100).toFixed(2);
        return value > 0
          ? `<div class="${color} h-full relative flex items-center justify-center" style="width: ${percent}%;">
               <span class="text-white text-xs">${label} (${value})</span>
             </div>`
          : "";
      })
      .join("");

    el.innerHTML = `
      <div class="h-40 w-full">
        <div class="flex h-full overflow-hidden rounded-md shadow-sm">${bars}</div>
      </div>
    `;
  }

  const loadDataForTimePeriod = async (timePeriod) => {
    try {
      const analytics = await getAnalyticsCounts(timePeriod);
      agentCount = analytics.agentCount;
      workflowCount = analytics.workflowCount;
      runtimeSessionCount = analytics.runtimeSessionCount;
      stepCount = analytics.stepCount;

      const [workflowPerf, errorDist] = await Promise.all([
        getWorkflowPerformance(timePeriod),
        getErrorDistribution(timePeriod),
      ]);

      workflowPerformance = workflowPerf;
      errorDistribution = errorDist;

      renderUsageChart();
      renderErrorDistributionChart();
    } catch (e) {
      console.error("Error loading analytics data", e);
    }
  };

  // Fetch data on mount:
  onMount(() => loadDataForTimePeriod(selectedTimePeriod));

  // Update data on time period change:
  $: if (selectedTimePeriod) {
    setTimeout(() => {
      loadDataForTimePeriod(selectedTimePeriod);
    }, 0);
  }

  const actionBar = [
    {
      type: "select",
      icon: CalendarMonthOutline,
      value: selectedTimePeriod,
      options: timePeriods,
      onChange: (event) => {
        selectedTimePeriod = event.target.value;
      },
    },
  ];
</script>

<main class="container mx-auto p-4">
  <!-- 🔥 PageHeader Component with Dropdown -->
  <PageHeader title="Analytics" {breadcrumbs} {actionBar} />

  <!-- Summary Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mb-6">
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Agents</div>
        <div class="text-2xl font-bold" data-testid="total-agents">
          {agentCount}
        </div>
      </div>
    </Card>
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Workflows</div>
        <div class="text-2xl font-bold" data-testid="total-workflows">
          {workflowCount}
        </div>
      </div>
    </Card>
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Steps</div>
        <div class="text-2xl font-bold" data-testid="total-steps">
          {stepCount}
        </div>
      </div>
    </Card>
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Avg. Success Rate</div>
        <div class="text-2xl font-bold" data-testid="avg-success-rate">
          {workflowPerformance.length > 0
            ? Math.round(
                workflowPerformance.reduce(
                  (acc, cur) => acc + cur.successRate,
                  0,
                ) / workflowPerformance.length,
              )
            : 0}%
        </div>
      </div>
    </Card>
    <Card padding="sm">
      <div class="flex flex-col p-4">
        <div class="text-gray-500 text-sm mb-1">Total Executions</div>
        <div class="text-2xl font-bold" data-testid="total-executions">
          {runtimeSessionCount}
        </div>
      </div>
    </Card>
  </div>

  <!-- Charts Section -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
    <Card class="max-w-full">
      <div class="p-4">
        <div class="flex justify-between items-center mb-4">
          <Heading tag="h3" class="text-lg font-semibold">Success Rate</Heading>
          <ChartBars3FromLeftOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="success-rate-chart" class="w-full">
          <SuccessChart agentPerformance={workflowPerformance} />
        </div>
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
        <div id="execution-time-chart" class="w-full">
          <ResponseTimeChart agentPerformance={workflowPerformance} />
        </div>
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
          <Heading tag="h3" class="text-lg font-semibold">Run Status</Heading>
          <ChartBars3FromLeftOutline class="h-5 w-5 text-gray-500" />
        </div>
        <div id="error-distribution-chart" class="w-full"></div>
      </div>
    </Card>
  </div>

  <!-- Performance Tables -->
  <Tabs style="underline">
    <TabItem open title="Workflow Performance">
      <Card class="max-w-full">
        <Table hoverable>
          <TableHead>
            <TableHeadCell>Workflow</TableHeadCell>
            <TableHeadCell>Success Rate</TableHeadCell>
            <TableHeadCell>Total Runs</TableHeadCell>
            <TableHeadCell>Avg. Response Time</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each workflowPerformance as workflow}
              <TableBodyRow>
                <TableBodyCell>{workflow.workflowName}</TableBodyCell>
                <TableBodyCell>
                  <div class="flex items-center">
                    <div class="w-16 bg-gray-200 rounded-full h-2.5 mr-2">
                      <div
                        class="bg-{workflow.successRate >= 90
                          ? 'green'
                          : workflow.successRate >= 70
                            ? 'yellow'
                            : 'red'}-500 h-2.5 rounded-full"
                        style="width: {workflow.successRate}%"
                      ></div>
                    </div>
                    <span>{workflow.successRate}%</span>
                  </div>
                </TableBodyCell>
                <TableBodyCell>{workflow.totalRuns}</TableBodyCell>
                <TableBodyCell>{workflow.avgResponseTime}</TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    </TabItem>

    <!-- TODO: Add step performance view back once available via schema -->
    <!-- <TabItem title="Step Performance">
      <Card class="max-w-full">
        <Table hoverable={true}>
          <TableHead>
            <TableHeadCell>Step Name</TableHeadCell>
            <TableHeadCell>Type</TableHeadCell>
            <TableHeadCell>Agent</TableHeadCell>
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
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </Card>
    </TabItem> -->
  </Tabs>
</main>
