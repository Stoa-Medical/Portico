import supabase from "$lib/supabase";
import { getUserId, getUserIdIfEnforced } from "$lib/user";
import type {
  Agent,
  AgentCapabilities,
  AgentPolicy,
  WorkflowCompositionRequest,
} from "$lib/types";

// Omit both "id" and "owner_id" fields for creation:
export type CreateAgentPayload = Omit<
  Agent,
  "id" | "owner_id" | "created_at" | "updated_at"
>;

// Allow partial Agent updates:
export type UpdateAgentPayload = Partial<Agent> & { id: number };

export const getAgents = async (): Promise<Agent[]> => {
  const userId = await getUserIdIfEnforced();
  const query = supabase.from("agents").select("*");

  // Only filter by owner_id if enforceAgentOwnership is enabled
  if (userId) {
    query.eq("owner_id", userId);
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};

export const saveAgent = async (
  agent: CreateAgentPayload,
): Promise<Agent[]> => {
  const userId = await getUserId();
  const { error } = await supabase
    .from("agents")
    .insert([{ ...agent, owner_id: userId, agent_state: "stable" }]);
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
  const userId = await getUserIdIfEnforced();

  // Verify ownership if enforcement is enabled
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("capabilities")
      .eq("id", agentId)
      .eq("owner_id", userId);

    if (!agentData || agentData.length === 0) {
      return null;
    }
    return agentData[0].capabilities;
  }

  const { data, error } = await supabase
    .from("agents")
    .select("capabilities")
    .eq("id", agentId)
    .single();

  if (error) throw error;
  return data.capabilities;
};

export const updateAgentCapabilities = async (
  agentId: number,
  capabilities: AgentCapabilities,
): Promise<void> => {
  const userId = await getUserIdIfEnforced();

  // Verify ownership if enforcement is enabled
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("id", agentId)
      .eq("owner_id", userId);

    if (!agentData || agentData.length === 0) {
      throw new Error("Agent not found or access denied");
    }
  }

  const { error } = await supabase
    .from("agents")
    .update({ capabilities })
    .eq("id", agentId);

  if (error) throw error;
};

export const getAgentPolicy = async (
  agentId: number,
): Promise<AgentPolicy | null> => {
  const userId = await getUserIdIfEnforced();

  // Verify ownership if enforcement is enabled
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("policy")
      .eq("id", agentId)
      .eq("owner_id", userId);

    if (!agentData || agentData.length === 0) {
      return null;
    }
    return agentData[0].policy;
  }

  const { data, error } = await supabase
    .from("agents")
    .select("policy")
    .eq("id", agentId)
    .single();

  if (error) throw error;
  return data.policy;
};

export const updateAgentPolicy = async (
  agentId: number,
  policy: AgentPolicy,
): Promise<void> => {
  const userId = await getUserIdIfEnforced();

  // Verify ownership if enforcement is enabled
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("id", agentId)
      .eq("owner_id", userId);

    if (!agentData || agentData.length === 0) {
      throw new Error("Agent not found or access denied");
    }
  }

  const { error } = await supabase
    .from("agents")
    .update({ policy })
    .eq("id", agentId);

  if (error) throw error;
};

// Workflow composition - the new primary action for agents
export const requestPlanWorkflow = async (
  payload: WorkflowCompositionRequest,
): Promise<void> => {
  const userId = await getUserId();

  // Verify agent ownership if enforcement is enabled
  const userIdIfEnforced = await getUserIdIfEnforced();
  if (userIdIfEnforced) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("id", payload.agent_id)
      .eq("owner_id", userIdIfEnforced);

    if (!agentData || agentData.length === 0) {
      throw new Error("Agent not found or access denied");
    }
  }

  // Create a composition request that the bridge service will pick up
  const { error } = await supabase.from("agent_composition_requests").insert([
    {
      agent_id: payload.agent_id,
      objective: payload.objective,
      context: payload.context ?? {},
      constraints: payload.constraints ?? {},
      is_ephemeral: !!payload.is_ephemeral,
      auto_execute: !!payload.auto_execute,
    },
  ]);

  if (error) throw error;
};
