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
  // settings: AgentLLMConfig;
  // capabilities: string[];
  // isActive: boolean;
  // model: string;
  // createdAt: string;
};

export type Step = {
  id: number | string;
  agent_id: number;
  name: string;
  description?: string;
  step_content: string;
  step_type: "Python" | "Prompt" | string;
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

// Omit "id" field for creation:
export type CreateAgentPayload = Omit<Agent, "id">;

// Allow partial Step and Agent updates:
export type UpdateStepPayload = Partial<Step> & {
  id: number;
  agent_id: number;
};
export type UpdateAgentPayload = Partial<Agent> & { id: number };

const defaultPythonContent = `import pandas as pd
import matplotlib.pyplot as plt

# Load the data
def process_data(file_path):
    df = pd.read_csv(file_path)

    # Clean the data
    df = df.dropna()

    # Perform analysis
    summary = df.describe()

    # Create visualization
    plt.figure(figsize=(10, 6))
    df.plot(kind='bar')
    plt.title('Data Analysis')
    plt.savefig('analysis_result.png')

    return summary

# Main function
if __name__ == "__main__":
    result = process_data('data.csv')
    print(result)`;

let currentAgentSteps: Step[] = [
  {
    id: 1,
    agent_id: 1,
    name: "Data Collection",
    step_type: "Python",
    // lastEdited: "2 hours ago",
    step_content: defaultPythonContent,
  },
  {
    id: 2,
    agent_id: 1,
    name: "Text Analysis",
    step_type: "Prompt",
    // lastEdited: "1 day ago",
    step_content: `You are an AI assistant that helps with data analysis.
Please analyze the following data and provide insights:
{{data}}

Focus on trends, anomalies, and potential actionable insights.`,
  },
  {
    id: 3,
    agent_id: 1,
    name: "Data Visualization",
    step_type: "Python",
    // lastEdited: "3 days ago",
    step_content: defaultPythonContent,
  },
];

function generateCompletedSessionsForAgent(
  agentId: number,
  count: number,
): RuntimeSession[] {
  return Array.from({ length: count }, (_, i) => {
    const stepCount = currentAgentSteps.filter(
      (step) => step.agent_id === agentId,
    ).length;
    return {
      id: agentId * 100 + i + 1,
      globalUuid: crypto.randomUUID(),
      requestedByAgentId: agentId,
      createdTimestamp: new Date().toISOString(),
      lastUpdatedTimestamp: new Date().toISOString(),
      runtimeSessionStatus: "completed",
      initialData: {
        input: `Sample input for Agent ${agentId}, Session ${i + 1}`,
      },
      latestStepIdx: stepCount,
      latestResult: {
        summary: `Completed ${stepCount} steps successfully.`,
        output: `Result data for Agent ${agentId}, Session ${i + 1}`,
      },
    };
  });
}

let currentRuntimeSessions: RuntimeSession[] = [
  ...generateCompletedSessionsForAgent(1, 5),
  ...generateCompletedSessionsForAgent(2, 5),
  ...generateCompletedSessionsForAgent(3, 5),
];

export const getAgents = async (): Promise<Agent[]> => {
  const { data, error } = await supabase.from("agents").select("*");
  if (error) throw error;
  return data;
};

export const saveAgent = async (
  agent: CreateAgentPayload,
): Promise<Agent[]> => {
  const { error } = await supabase.from("agents").insert([{ ...agent }]);
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
      : step;

  const { error: insertError } = await supabase
    .from("steps")
    .insert([stepToInsert]);

  if (insertError) throw insertError;

  return getStep(step.agent_id);
};

export const updateStep = async (
  updatedStep: UpdateStepPayload,
): Promise<Step[]> => {
  const { error } = await supabase
    .from("steps")
    .update(updatedStep)
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
  return currentRuntimeSessions.filter(
    (session) => session.requestedByAgentId === agentId,
  );
};
