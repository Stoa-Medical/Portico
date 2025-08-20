import supabase from "$lib/supabase";
import { getStartDateFromTimePeriod } from "$lib/date";
import { getUserId } from "$lib/user";

const DEFAULT_TIME_PERIOD = "30d";

export const getAnalyticsCounts = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);

  const [
    { count: agentCount, error: agentError },
    { count: workflowCount, error: workflowError },
    { count: sessionCount, error: sessionError },
    { count: stepCount, error: stepError },
  ] = await Promise.all([
    supabase.from("agents").select("id", { count: "exact", head: true }),
    supabase.from("workflows").select("id", { count: "exact", head: true }),
    supabase
      .from("runtime_sessions")
      .select("id", { count: "exact", head: true })
      .gte("created_at", fromDate)
      .not("workflow_id", "is", null),
    supabase
      .from("steps")
      .select("id", { count: "exact", head: true })
      .not("workflow_id", "is", null),
  ]);

  if (agentError) throw agentError;
  if (workflowError) throw workflowError;
  if (sessionError) throw sessionError;
  if (stepError) throw stepError;

  return {
    agentCount: agentCount ?? 0,
    workflowCount: workflowCount ?? 0,
    runtimeSessionCount: sessionCount ?? 0,
    stepCount: stepCount ?? 0,
  };
};

export const getWorkflowPerformance = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);

  const { data: workflowData, error: workflowError } = await supabase
    .from("workflows")
    .select("id, name");

  if (workflowError) throw workflowError;
  const workflowIds = workflowData?.map((w) => w.id) ?? [];

  if (workflowIds.length === 0) {
    return [];
  }

  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("workflow_id, rts_status, total_execution_time")
    .gte("created_at", fromDate)
    .in("workflow_id", workflowIds);

  if (error) throw error;

  const grouped = new Map();
  data.forEach(({ workflow_id, rts_status, total_execution_time }) => {
    const group = grouped.get(workflow_id) || {
      totalRuns: 0,
      successRuns: 0,
      totalTime: 0,
    };
    group.totalRuns++;
    if (rts_status === "completed") group.successRuns++;
    group.totalTime += parseFloat(total_execution_time ?? 0);
    grouped.set(workflow_id, group);
  });

  const workflowNameMap = new Map(workflowData.map((w) => [w.id, w.name]));

  return Array.from(grouped.entries()).map(
    ([workflowId, { totalRuns, successRuns, totalTime }]) => ({
      workflowId,
      workflowName: workflowNameMap.get(workflowId) || `Workflow ${workflowId}`,
      successRate: totalRuns ? Math.round((successRuns / totalRuns) * 100) : 0,
      totalRuns,
      avgResponseTime: totalRuns
        ? (totalTime / totalRuns).toFixed(2) + "s"
        : "0s",
    }),
  );
};

// Keep the old function for backward compatibility but mark as deprecated
export const getAgentPerformance = getWorkflowPerformance;

export const getStepPerformance = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);

  const { data: workflowData, error: workflowError } = await supabase
    .from("workflows")
    .select("id, name");

  if (workflowError) throw workflowError;
  const workflowIds = workflowData?.map((w) => w.id) ?? [];

  if (workflowIds.length === 0) {
    return [];
  }

  const { data: sessionData, error: sessionError } = await supabase
    .from("runtime_sessions")
    .select("step_ids, step_execution_times, workflow_id")
    .gte("created_at", fromDate)
    .in("workflow_id", workflowIds);

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
    .select("id, name, step_type, workflow_id")
    .in("workflow_id", workflowIds);

  const workflowMap = new Map(workflowData?.map((w) => [w.id, w.name]));

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
        workflowName: workflowMap.get(step.workflow_id) ?? "Unknown",
      };
    }) ?? []
  );
};

export const getErrorDistribution = async (
  timePeriod: string = DEFAULT_TIME_PERIOD,
) => {
  const fromDate = getStartDateFromTimePeriod(timePeriod);

  const { data: workflowData, error: workflowError } = await supabase
    .from("workflows")
    .select("id");

  if (workflowError) throw workflowError;
  const workflowIds = workflowData?.map((w) => w.id) ?? [];

  if (workflowIds.length === 0) {
    return {
      completed: 0,
      cancelled: 0,
      running: 0,
      waiting: 0,
    };
  }

  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("rts_status")
    .gte("created_at", fromDate)
    .in("workflow_id", workflowIds);

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
