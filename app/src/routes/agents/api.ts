import supabase from "$lib/supabase";
import { getUserId, getUserIdIfEnforced } from "$lib/user";
import type {
  Agent,
  AgentCapabilities,
  AgentPolicy,
  Workflow,
  WorkflowCompositionRequest,
} from "$lib/types";

// Omit id and timestamp fields for creation:
export type CreateAgentPayload = Omit<
  Agent,
  "id" | "global_uuid" | "created_at" | "updated_at"
>;

// Allow partial Agent updates:
export type UpdateAgentPayload = Partial<Agent> & { id: number };

export const getAgents = async (): Promise<Agent[]> => {
  // Note: ownership filtering removed as owner_id doesn't exist in schema
  // Future: implement organization-based filtering if needed
  const { data, error } = await supabase.from("agents").select("*");
  if (error) throw error;
  return data;
};

export const saveAgent = async (
  agent: CreateAgentPayload,
): Promise<Agent[]> => {
  const { error } = await supabase.from("agents").insert([agent]);
  if (error) throw error;
  return getAgents();
};

export const updateAgent = async (
  updatedAgent: UpdateAgentPayload,
): Promise<Agent[]> => {
  const { id, ...rest } = updatedAgent;
  const { error } = await supabase
    .from("agents")
    .update(rest)
    .eq("id", updatedAgent.id);
  if (error) throw error;
  return getAgents();
};

export const deleteAgent = async (
  agentIdToDelete: number,
): Promise<Agent[]> => {
  // Note: With the new architecture, we should also consider what to do with workflows
  // created by this agent. For now, we'll leave them orphaned.

  // Delete Agent:
  const { error: agentDeleteError } = await supabase
    .from("agents")
    .delete()
    .eq("id", agentIdToDelete);
  if (agentDeleteError) throw agentDeleteError;
  return getAgents();
};

// Agent capabilities and policy management
export const getAgentCapabilities = async (
  agentId: number,
): Promise<AgentCapabilities | null> => {
  const { data, error } = await supabase
    .from("agents")
    .select("capabilities_json")
    .eq("id", agentId)
    .single();

  if (error) throw error;
  return data.capabilities_json;
};

export const updateAgentCapabilities = async (
  agentId: number,
  capabilities: AgentCapabilities,
): Promise<void> => {
  const { error } = await supabase
    .from("agents")
    .update({ capabilities_json: capabilities })
    .eq("id", agentId);

  if (error) throw error;
};

export const getAgentPolicy = async (
  agentId: number,
): Promise<AgentPolicy | null> => {
  const { data, error } = await supabase
    .from("agents")
    .select("policy_json")
    .eq("id", agentId)
    .single();

  if (error) throw error;
  return data.policy_json;
};

export const updateAgentPolicy = async (
  agentId: number,
  policy: AgentPolicy,
): Promise<void> => {
  const { error } = await supabase
    .from("agents")
    .update({ policy_json: policy })
    .eq("id", agentId);

  if (error) throw error;
};

// Workflow composition - the new primary action for agents
export const requestPlanWorkflow = async (
  payload: WorkflowCompositionRequest,
): Promise<void> => {
  // Create a signal to trigger agent workflow composition
  // The bridge service will handle this signal and coordinate with the engine
  const { error } = await supabase.from("signals").insert([
    {
      initiator_agent_id: payload.agent_id,
      user_requested_uuid: crypto.randomUUID(),
      signal_type: "run",
      initial_data: {
        action: "compose_workflow",
        agent_id: payload.agent_id,
        objective: payload.objective,
        context: payload.context ?? {},
        constraints: payload.constraints ?? {},
        is_ephemeral: !!payload.is_ephemeral,
        auto_execute: !!payload.auto_execute,
      },
    },
  ]);

  if (error) throw error;
};

// Get workflows created by a specific agent
export const getAgentWorkflowHistory = async (
  agentId: number,
  includeEphemeral = false,
): Promise<Workflow[]> => {
  let query = supabase
    .from("workflows")
    .select("*")
    .eq("created_by_agent_id", agentId)
    .order("created_at", { ascending: false });

  if (!includeEphemeral) {
    query = query.eq("is_ephemeral", false);
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};

// Get execution metrics for workflows created by an agent
export const getAgentExecutionMetrics = async (
  agentId: number,
  timeRange?: { from: Date; to: Date },
) => {
  // Get workflows created by this agent
  const { data: workflows } = await supabase
    .from("workflows")
    .select("id")
    .eq("created_by_agent_id", agentId);

  const workflowIds = workflows?.map((w) => w.id) ?? [];

  if (workflowIds.length === 0) {
    return {
      totalWorkflowsCreated: 0,
      totalExecutions: 0,
      successRate: 0,
      avgExecutionTime: "0s",
    };
  }

  let query = supabase
    .from("runtime_sessions")
    .select("*")
    .in("workflow_id", workflowIds);

  if (timeRange) {
    query = query
      .gte("created_at", timeRange.from.toISOString())
      .lte("created_at", timeRange.to.toISOString());
  }

  const { data: sessions } = await query;

  // Calculate metrics
  const totalExecutions = sessions?.length ?? 0;
  const successfulExecutions =
    sessions?.filter((s) => s.rts_status === "completed").length ?? 0;
  const totalTime =
    sessions?.reduce(
      (sum, s) => sum + parseFloat(s.total_execution_time ?? 0),
      0,
    ) ?? 0;

  return {
    totalWorkflowsCreated: workflowIds.length,
    totalExecutions,
    successRate:
      totalExecutions > 0
        ? Math.round((successfulExecutions / totalExecutions) * 100)
        : 0,
    avgExecutionTime:
      totalExecutions > 0
        ? (totalTime / totalExecutions).toFixed(2) + "s"
        : "0s",
  };
};
