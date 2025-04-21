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
  agent_id: number;
  name: string;
  description?: string;
  step_content: string;
  step_type: "python" | "prompt" | string;
};

export type RuntimeSession = {
  id: number;
  globalUuid: string;
  requestedByAgentId: number;
  createdTimestamp: string;
  lastUpdatedTimestamp: string;
  runtimeSessionStatus: "queued" | "running" | "completed" | "failed";
  initialData: any; // JSON blob
  latestStepIdx: number;
  latestResult: any | null; // nullable JSON
};

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

  const { name, type, ...usedFields } = agent;

  const { error } = await supabase
    .from("agents")
    .insert([{ ...usedFields, owner_id: user.id }]);

  if (error) throw error;
  return getAgents();
};

export const updateAgent = async (
  updatedAgent: UpdateAgentPayload,
): Promise<Agent[]> => {
  const { error } = await supabase
    .from("agents")
    .update(updatedAgent)
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

export const saveStep = async (step: Step): Promise<Step[]> => {
  const { id, ...rest } = step;

  // Get next sequence_number for this agent:
  const { data: existingSteps, error: fetchError } = await supabase
    .from("steps")
    .select("sequence_number")
    .eq("agent_id", step.agent_id)
    .order("sequence_number", { ascending: false })
    .limit(1);

  if (fetchError) throw fetchError;

  const nextSequenceNumber = (existingSteps?.[0]?.sequence_number ?? 0) + 1;

  // Insert step with metadata:
  const stepToInsert =
    id === "new"
      ? {
          ...rest,
          sequence_number: nextSequenceNumber,
        }
      : { step, sequence_number: nextSequenceNumber };

  const { error: insertError } = await supabase
    .from("steps")
    .insert([stepToInsert]);

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
