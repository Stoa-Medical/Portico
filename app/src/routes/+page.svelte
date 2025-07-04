<script>
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { Card, Button } from "flowbite-svelte";
  import { onMount } from "svelte";
  import { getAnalyticsCounts, getAgentPerformance } from "./analytics/api";

  let agentPerf;
  let runtimeSessionCount = 0;
  let errorRate = 0;

  onMount(async () => {
    try {
      // Load analytics data for system status
      const analytics = await getAnalyticsCounts();
      agentPerf = await getAgentPerformance();
      runtimeSessionCount = analytics.runtimeSessionCount;

      errorRate =
        100 -
        agentPerf.reduce((acc, agent) => {
          return acc + agent.successRate / agentPerf.length;
        }, 0);
    } catch (e) {
      console.error("Error loading analytics data", e);
    }
  });
</script>

<main class="container mx-auto p-4">
  <!-- Page Header with Breadcrumb -->
  <PageHeader title="Welcome" breadcrumbs={[{ label: "Home", url: "/" }]} />

  <!-- Main Content -->
  <div class="grid grid-cols-1 gap-6">
    <Card class="overflow-x-auto max-w-full bg-card">
      <div class="p-4">
        <h2 class="mb-4 text-4xl">Get started on something new</h2>

        <p class="mt-2 mb-6">
          Create and manage agents to automate your workflows
        </p>

        <div
          class="flex flex-col space-y-4 sm:flex-row sm:space-y-0 sm:space-x-4 w-full"
        >
          <Button href="/agents" size="xl" class="bg-sea text-black"
            >Agents</Button
          >
          <Button href="/analytics" size="xl" class="bg-sea text-black"
            >Analytics</Button
          >
        </div>
      </div>
    </Card>

    <Card class="overflow-x-auto max-w-full bg-card">
      <div class="p-4">
        <h2 class="mb-6 text-4xl text-white font-sans">System status</h2>

        <div class="flex flex-wrap gap-4">
          <div
            class="p-4 rounded-md text-[18px] leading-[24px] tracking-normal font-normal text-white/60 bg-white/5 backdrop-blur-sm shadow-sm font-sans"
          >
            <span class="text-green-400 font-semibold"
              >{runtimeSessionCount}</span
            > executions
          </div>
          <div
            class="p-4 rounded-md text-[18px] leading-[24px] tracking-normal font-normal text-white/60 bg-white/5 backdrop-blur-sm shadow-sm font-sans"
          >
            <span class="text-red-400 font-semibold"
              >{errorRate.toFixed(4)}%</span
            > error rate
          </div>
          <div
            class="p-4 rounded-md text-[18px] leading-[24px] tracking-normal font-normal text-white/60 bg-white/5 backdrop-blur-sm shadow-sm font-sans"
          >
            <span class="text-sea font-semibold">{agentPerf?.length || 0}</span>
            active agents
          </div>
        </div>
      </div>
    </Card>
  </div>
</main>

<style>
</style>
