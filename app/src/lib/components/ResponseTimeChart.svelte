<script>
  import { Chart } from "flowbite-svelte";

  const isTest = import.meta.env.MODE === "test";

  export let agentPerformance = [];

  // Helper to convert "1.23s" to 1.23
  function toSeconds(timeStr) {
    return parseFloat(timeStr.replace("s", "")) || 0;
  }

  $: options = {
    chart: {
      type: "bar",
      height: 300,
      toolbar: { show: false },
      fontFamily: "Inter, sans-serif",
    },
    series: [
      {
        name: "Avg. Response Time (s)",
        data: agentPerformance.map((agent) => ({
          x: agent.name ?? `Agent ${agent.agentId}`,
          y: toSeconds(agent.avgResponseTime),
        })),
      },
    ],
    plotOptions: {
      bar: {
        horizontal: false,
        columnWidth: "20%",
        distributed: true,
        borderRadius: 2,
      },
    },
    xaxis: {
      type: "category",
      labels: {
        rotate: -45,
        style: {
          fontSize: "12px",
          colors: "#888",
        },
      },
    },
    yaxis: {
      tickAmount: 5,
      labels: {
        formatter: (val) => `${val.toFixed(2)}s`,
        style: {
          colors: "#888",
        },
      },
    },
    colors: agentPerformance.map(() => "#3B82F6"), // solid blue for consistency
    dataLabels: {
      enabled: true,
      formatter: (val) => `${val.toFixed(2)}s`,
      style: {
        fontSize: "12px",
      },
    },
    tooltip: {
      y: {
        formatter: (val) => `${val.toFixed(2)}s`,
      },
    },
  };
</script>

{#if agentPerformance.length > 0 && !isTest}
  <Chart {options} />
{:else}
  <div class="text-center py-6 text-gray-400 text-sm">
    No response time data available
  </div>
{/if}
