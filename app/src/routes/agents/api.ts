import supabase from "$lib/supabase";
import { getUserId, getUserIdIfEnforced } from "$lib/user";

// TODO: Should be connected to a prompt step rather than the agent:
type AgentLLMConfig = {
  temperature: number;
  maxTokens: number;
  topP: number;
  frequencyPenalty: number;
  presencePenalty: number;
};

export type Agent = {
  id: number;
  name: string;
  agent_state: string;
  type: string;
  // lastActive: string;
  description: string;
  owner_id: string;
  // settings: AgentLLMConfig;
  // capabilities: string[];
  // isActive: boolean;
  // model: string;
  // created_at: string;
};

export type Step = {
  id: number | string;
  global_uuid: string;
  agent_id: number;
  name: string;
  description?: string;
  step_content: string;
  step_type: "python" | "prompt" | string;
};

export type RuntimeSession = {
  id: number;
  global_uuid: string;
  requested_by_agent_id: number;
  created_at: string;
  updated_at: string;
  rts_status: "queued" | "running" | "completed" | "failed";
  initial_data: any; // JSON blob
  latest_step_idx: number;
  latest_result: any | null; // nullable JSON
};

// Omit both "id" and "owner_id" fields for creation:
export type CreateStepPayload = Omit<Step, "id" | "global_uuid">;

// Omit both "id" and "owner_id" fields for creation:
export type CreateAgentPayload = Omit<Agent, "id" | "owner_id">;

// Allow partial Step and Agent updates:
export type UpdateStepPayload = Partial<Step> & {
  id: number;
  agent_id: number;
};
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
  // Delete dependent steps:
  const { error: stepDeleteError } = await supabase
    .from("steps")
    .delete()
    .eq("agent_id", agentIdToDelete);

  if (stepDeleteError) throw stepDeleteError;

  // Delete Agent:
  const { error: agentDeleteError } = await supabase
    .from("agents")
    .delete()
    .eq("id", agentIdToDelete);
  if (agentDeleteError) throw agentDeleteError;
  return getAgents();
};

export const getStep = async (stepId): Promise<Step[]> => {
  const { data, error } = await supabase
    .from("steps")
    .select("*")
    .eq("id", stepId);
  if (error) throw error;
  return data;
};

export const getSteps = async (agentId: number): Promise<Step[]> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase.from("steps").select("*").eq("agent_id", agentId);

  // If enforceAgentOwnership is enabled, only show steps from agents owned by this user
  if (userId) {
    // First get the agent to verify ownership
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("id", agentId)
      .eq("owner_id", userId);

    // If agent doesn't belong to user, return empty array
    if (!agentData || agentData.length === 0) {
      return [];
    }
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};

export const saveStep = async (step: Step): Promise<Step[]> => {
  const { id, ...rest } = step;
  const { error: insertError } = await supabase.from("steps").insert([rest]);
  if (insertError) throw insertError;
  return getStep(step.agent_id);
};

export const updateStep = async (
  updatedStep: UpdateStepPayload,
): Promise<Step[]> => {
  const { id, ...rest } = updatedStep;
  const { error } = await supabase
    .from("steps")
    .update(rest)
    .eq("id", updatedStep.id);
  if (error) throw error;
  return getStep(updatedStep.agent_id);
};

export const deleteStep = async (stepIdToDelete: number): Promise<void> => {
  const { error } = await supabase
    .from("steps")
    .delete()
    .eq("id", stepIdToDelete);
  if (error) throw error;
};

export const getRuntimeSessions = async (
  agentId: number,
): Promise<RuntimeSession[]> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase
    .from("runtime_sessions")
    .select("*")
    .eq("requested_by_agent_id", agentId)
    .order("created_at", { ascending: false });

  // If enforceAgentOwnership is enabled, verify the agent is owned by this user
  if (userId) {
    // First check if the agent belongs to the user
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("id", agentId)
      .eq("owner_id", userId);

    // If agent doesn't belong to user, return empty array
    if (!agentData || agentData.length === 0) {
      return [];
    }
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};
