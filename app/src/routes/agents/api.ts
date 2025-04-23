import supabase from "$lib/supabase";

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
  const { data, error } = await supabase.from("agents").select("*");
  if (error) throw error;
  return data;
};

export const saveAgent = async (
  agent: CreateAgentPayload,
): Promise<Agent[]> => {
  const {
    data: { user },
    error: authError,
  } = await supabase.auth.getUser();

  if (authError) throw authError;
  if (!user) throw new Error("User must be logged in to create an agent");

  const { error } = await supabase
    .from("agents")
    .insert([{ ...agent, owner_id: user.id }]);

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
  const { data, error } = await supabase
    .from("steps")
    .select("*")
    .eq("agent_id", agentId);
  if (error) throw error;
  return data;
};

export const saveStep = async (step: CreateStepPayload): Promise<Step[]> => {
  const { error: insertError } = await supabase.from("steps").insert([step]);
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
  const { data, error } = await supabase
    .from("runtime_sessions")
    .select("*")
    .eq("requested_by_agent_id", agentId)
    .order("created_at", { ascending: false });

  if (error) throw error;
  return data;
};
