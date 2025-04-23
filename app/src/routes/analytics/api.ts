import supabase from "$lib/supabase";

export const getAnalyticsCounts = async () => {
  const [
    { count: agentCount, error: agentError },
    { count: sessionCount, error: sessionError },
  ] = await Promise.all([
    supabase.from("agents").select("*", { count: "exact", head: true }),
    supabase
      .from("runtime_sessions")
      .select("*", { count: "exact", head: true }),
  ]);

  if (agentError) throw agentError;
  if (sessionError) throw sessionError;

  return {
    agentCount: agentCount ?? 0,
    runtimeSessionCount: sessionCount ?? 0,
  };
};
