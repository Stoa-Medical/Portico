import supabase from "$lib/supabase";
import { getUserId, getUserIdIfEnforced } from "$lib/user";
import type { Workflow, Step, RuntimeSession } from "$lib/types";

// Types for creating new workflows and steps
export type CreateWorkflowPayload = Omit<
  Workflow,
  "id" | "created_at" | "updated_at"
>;
export type CreateStepPayload = Omit<
  Step,
  "id" | "global_uuid" | "created_at" | "updated_at"
>;

// Types for updating workflows and steps
export type UpdateWorkflowPayload = Partial<Workflow> & { id: number };
export type UpdateStepPayload = Partial<Step> & {
  id: number;
  workflow_id: number;
};

// Workflow CRUD operations
export const getWorkflows = async (): Promise<Workflow[]> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase
    .from("workflows")
    .select("*")
    .order("created_at", { ascending: false });

  // If enforceAgentOwnership is enabled, only show workflows created by user's agents
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("owner_id", userId);

    const agentIds = agentData?.map((a) => a.id) ?? [];
    if (agentIds.length > 0) {
      query = query.in("created_by_agent_id", agentIds);
    } else {
      // No owned agents, return empty array
      return [];
    }
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};

export const getWorkflow = async (
  workflowId: number,
): Promise<Workflow | null> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase.from("workflows").select("*").eq("id", workflowId);

  // Verify ownership if enforcement is enabled
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("owner_id", userId);

    const agentIds = agentData?.map((a) => a.id) ?? [];
    if (agentIds.length > 0) {
      query = query.in("created_by_agent_id", agentIds);
    } else {
      return null;
    }
  }

  const { data, error } = await query.single();
  if (error) {
    if (error.code === "PGRST116") return null; // Not found
    throw error;
  }
  return data;
};

export const saveWorkflow = async (
  workflow: CreateWorkflowPayload,
): Promise<Workflow[]> => {
  const { error } = await supabase
    .from("workflows")
    .insert([
      { ...workflow, workflow_state: workflow.workflow_state || "stable" },
    ]);
  if (error) throw error;
  return getWorkflows();
};

export const updateWorkflow = async (
  updatedWorkflow: UpdateWorkflowPayload,
): Promise<Workflow[]> => {
  const { id, ...rest } = updatedWorkflow;
  const { error } = await supabase.from("workflows").update(rest).eq("id", id);
  if (error) throw error;
  return getWorkflows();
};

export const deleteWorkflow = async (
  workflowId: number,
): Promise<Workflow[]> => {
  // Delete dependent steps first
  const { error: stepDeleteError } = await supabase
    .from("steps")
    .delete()
    .eq("workflow_id", workflowId);

  if (stepDeleteError) throw stepDeleteError;

  // Delete workflow
  const { error: workflowDeleteError } = await supabase
    .from("workflows")
    .delete()
    .eq("id", workflowId);
  if (workflowDeleteError) throw workflowDeleteError;

  return getWorkflows();
};

// Step operations for workflows
export const getSteps = async (workflowId: number): Promise<Step[]> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase.from("steps").select("*").eq("workflow_id", workflowId);

  // If enforceAgentOwnership is enabled, verify the workflow is owned by user's agents
  if (userId) {
    const workflow = await getWorkflow(workflowId);
    if (!workflow) {
      return [];
    }
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};

export const getStep = async (stepId: number | string): Promise<Step[]> => {
  const { data, error } = await supabase
    .from("steps")
    .select("*")
    .eq("id", stepId);
  if (error) throw error;
  return data;
};

export const saveStep = async (step: CreateStepPayload): Promise<Step[]> => {
  const { error: insertError } = await supabase.from("steps").insert([step]);
  if (insertError) throw insertError;
  return getSteps(step.workflow_id);
};

export const updateStep = async (
  updatedStep: UpdateStepPayload,
): Promise<Step[]> => {
  const { id, ...rest } = updatedStep;
  const { error } = await supabase.from("steps").update(rest).eq("id", id);
  if (error) throw error;
  return getSteps(updatedStep.workflow_id);
};

export const deleteStep = async (
  stepId: number,
  workflowId: number,
): Promise<void> => {
  const { error } = await supabase.from("steps").delete().eq("id", stepId);
  if (error) throw error;
};

// Runtime sessions for workflows
export const getRuntimeSessions = async (
  workflowId: number,
): Promise<RuntimeSession[]> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase
    .from("runtime_sessions")
    .select("*")
    .eq("workflow_id", workflowId)
    .order("created_at", { ascending: false });

  // If enforceAgentOwnership is enabled, verify the workflow is owned by user's agents
  if (userId) {
    const workflow = await getWorkflow(workflowId);
    if (!workflow) {
      return [];
    }
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};

// Workflow execution
export const runWorkflow = async (
  workflowId: number,
  initialData: any = {},
  initiatorAgentId?: number,
): Promise<void> => {
  const userId = await getUserId();

  // Verify workflow exists and ownership if enforcement is enabled
  const userIdIfEnforced = await getUserIdIfEnforced();
  if (userIdIfEnforced) {
    const workflow = await getWorkflow(workflowId);
    if (!workflow) {
      throw new Error("Workflow not found or access denied");
    }
  }

  // Create a signal to trigger workflow execution
  const { error } = await supabase.from("signals").insert([
    {
      workflow_id: workflowId,
      initiator_agent_id: initiatorAgentId ?? null,
      user_requested_uuid: crypto.randomUUID(),
      signal_type: "run",
      initial_data: JSON.stringify(initialData),
    },
  ]);

  if (error) throw error;
};

// Helper function to get workflows created by specific agent
export const getWorkflowsByAgent = async (
  agentId: number,
): Promise<Workflow[]> => {
  const userId = await getUserIdIfEnforced();
  let query = supabase
    .from("workflows")
    .select("*")
    .eq("created_by_agent_id", agentId)
    .order("created_at", { ascending: false });

  // Verify agent ownership if enforcement is enabled
  if (userId) {
    const { data: agentData } = await supabase
      .from("agents")
      .select("id")
      .eq("id", agentId)
      .eq("owner_id", userId);

    if (!agentData || agentData.length === 0) {
      return [];
    }
  }

  const { data, error } = await query;
  if (error) throw error;
  return data;
};
