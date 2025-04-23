<script>
  import { Chart } from "flowbite-svelte";

  const isTest = import.meta.env.MODE === "test";

  export let agentPerformance = [];

  $: options = {
    chart: {
      type: "bar",
      height: 300,
      toolbar: { show: false },
      fontFamily: "Inter, sans-serif",
    },
    series: [
      {
        name: "Success Rate (%)",
        data: agentPerformance.map((agent) => ({
          x: agent.name ?? `Agent ${agent.agentId}`,
          y: agent.successRate,
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
      max: 100,
      tickAmount: 5,
      labels: {
        formatter: (val) => `${val}%`,
        style: {
          colors: "#888",
        },
      },
    },
    colors: agentPerformance.map((agent) => {
      const p = agent.successRate;
      return p >= 90 ? "#22C55E" : p >= 70 ? "#EAB308" : "#EF4444";
    }),
    dataLabels: {
      enabled: true,
      formatter: (val) => `${val}%`,
      style: {
        fontSize: "12px",
      },
    },
    tooltip: {
      y: {
        formatter: (val) => `${val}%`,
      },
    },
  };
</script>

{#if agentPerformance.length > 0 && !isTest}
  <Chart {options} />
{:else}
  <div class="text-center py-6 text-gray-400 text-sm">
    No agent performance data available
  </div>
{/if}
