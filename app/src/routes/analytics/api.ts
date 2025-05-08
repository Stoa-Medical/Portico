import supabase from "$lib/supabase";
import { getStartDateFromTimePeriod } from "$lib/date";
import { getUserId } from "$lib/user";

const DEFAULT_TIME_PERIOD = "30d";

export const getAnalyticsCounts = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);
  const userId = await getUserId();

  const { data: agentsData, error: agentsError } = await supabase
    .from("agents")
    .select("id")
    .eq("owner_id", userId);

  if (agentsError) throw agentsError;

  const agentIds = agentsData?.map((a) => a.id) ?? [];

  if (agentIds.length === 0) {
    return { agentCount: 0, runtimeSessionCount: 0, stepCount: 0 };
  }

  const [
    { count: agentCount, error: agentError },
    { count: sessionCount, error: sessionError },
    { count: stepCount, error: stepError },
  ] = await Promise.all([
    supabase
      .from("agents")
      .select("id", { count: "exact", head: true })
      .eq("owner_id", userId),
    supabase
      .from("runtime_sessions")
      .select("id", { count: "exact", head: true })
      .gte("created_at", fromDate)
      .in("requested_by_agent_id", agentIds),
    supabase
      .from("steps")
      .select("id", { count: "exact", head: true })
      .in("agent_id", agentIds),
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

export const getAgentPerformance = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);
  const userId = await getUserId();

  const { data: agentData, error: agentError } = await supabase
    .from("agents")
    .select("id")
    .eq("owner_id", userId);

  if (agentError) throw agentError;
  const agentIds = agentData.map((a) => a.id);

  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("requested_by_agent_id, rts_status, total_execution_time")
    .gte("created_at", fromDate)
    .in("requested_by_agent_id", agentIds);

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

export const getStepPerformance = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);
  const userId = await getUserId();

  const { data: agentData, error: agentError } = await supabase
    .from("agents")
    .select("id")
    .eq("owner_id", userId);

  if (agentError) throw agentError;
  const agentIds = agentData.map((a) => a.id);

  const { data: sessionData, error: sessionError } = await supabase
    .from("runtime_sessions")
    .select("step_ids, step_execution_times")
    .gte("created_at", fromDate)
    .in("requested_by_agent_id", agentIds);

  if (sessionError) throw sessionError;

  const stepStats = new Map();
  sessionData.forEach(({ step_ids, step_execution_times }) => {
    if (!step_ids || !step_execution_times) return;
    step_ids.forEach((stepId, idx) => {
      const time = parseFloat(step_execution_times[idx] ?? 0);
      const stat = stepStats.get(stepId) || { runs: 0, totalTime: 0 };
      stat.runs++;
      stat.totalTime += time;
      stepStats.set(stepId, stat);
    });
  });

  const { data: stepsData } = await supabase
    .from("steps")
    .select("id, name, step_type, agent_id")
    .in("agent_id", agentIds);

  const { data: agentsData } = await supabase
    .from("agents")
    .select("id, name")
    .eq("owner_id", userId);

  const agentMap = new Map(agentsData?.map((a) => [a.id, a.name]));

  return (
    stepsData?.map((step) => {
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

export const getErrorDistribution = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);
  const userId = await getUserId();

  const { data: agentData, error: agentError } = await supabase
    .from("agents")
    .select("id")
    .eq("owner_id", userId);

  if (agentError) throw agentError;
  const agentIds = agentData.map((a) => a.id);

  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("rts_status")
    .gte("created_at", fromDate)
    .in("requested_by_agent_id", agentIds);

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
