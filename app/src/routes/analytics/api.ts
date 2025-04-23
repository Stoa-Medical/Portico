import supabase from "$lib/supabase";

export const getAnalyticsCounts = async () => {
  const [
    { count: agentCount, error: agentError },
    { count: sessionCount, error: sessionError },
    { count: stepCount, error: stepError },
  ] = await Promise.all([
    supabase.from("agents").select("*", { count: "exact", head: true }),
    supabase
      .from("runtime_sessions")
      .select("*", { count: "exact", head: true }),
    supabase.from("steps").select("*", { count: "exact", head: true }),
  ]);

  if (agentError) throw agentError;
  if (sessionError) throw sessionError;
  if (stepError) throw stepError;

  return {
    agentCount: agentCount ?? 0,
    runtimeSessionCount: sessionCount ?? 0,
    stepCount: stepCount ?? 0,
  };
};

export const getAgentPerformance = async () => {
  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("requested_by_agent_id, rts_status, total_execution_time");

  if (error) throw error;

  const grouped = new Map();

  data.forEach(
    ({ requested_by_agent_id, rts_status, total_execution_time }) => {
      const group = grouped.get(requested_by_agent_id) || {
        totalRuns: 0,
        successRuns: 0,
        totalTime: 0,
      };
      group.totalRuns++;
      if (rts_status === "completed") group.successRuns++;
      group.totalTime += parseFloat(total_execution_time ?? 0);
      grouped.set(requested_by_agent_id, group);
    },
  );

  return Array.from(grouped.entries()).map(
    ([agentId, { totalRuns, successRuns, totalTime }]) => ({
      agentId,
      successRate: totalRuns ? Math.round((successRuns / totalRuns) * 100) : 0,
      totalRuns,
      avgResponseTime: totalRuns
        ? (totalTime / totalRuns).toFixed(2) + "s"
        : "0s",
    }),
  );
};

export const getStepPerformance = async () => {
  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("step_ids, step_execution_times");

  if (error) throw error;

  const stepStats = new Map();

  data.forEach(({ step_ids, step_execution_times }) => {
    if (!step_ids || !step_execution_times) return;
    step_ids.forEach((stepId, idx) => {
      const time = parseFloat(step_execution_times[idx] ?? 0);
      const stat = stepStats.get(stepId) || { runs: 0, totalTime: 0 };
      stat.runs++;
      stat.totalTime += time;
      stepStats.set(stepId, stat);
    });
  });

  const stepsResponse = await supabase
    .from("steps")
    .select("id, name, step_type, agent_id");

  const agentsResponse = await supabase.from("agents").select("id, name");

  const agentMap = new Map(agentsResponse.data?.map((a) => [a.id, a.name]));

  return (
    stepsResponse.data?.map((step) => {
      const stat = stepStats.get(step.id) || { runs: 0, totalTime: 0 };
      return {
        id: step.id,
        name: step.name,
        type: step.step_type,
        totalRuns: stat.runs,
        avgExecutionTime: stat.runs
          ? (stat.totalTime / stat.runs).toFixed(2) + "s"
          : "0s",
        agentName: agentMap.get(step.agent_id) ?? "Unknown",
      };
    }) ?? []
  );
};

export const getErrorDistribution = async () => {
  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("rts_status");

  if (error) throw error;

  const counts = {
    completed: 0,
    cancelled: 0,
    running: 0,
    waiting: 0,
  };

  data.forEach(({ rts_status }) => {
    counts[rts_status] = (counts[rts_status] ?? 0) + 1;
  });

  return counts;
};
