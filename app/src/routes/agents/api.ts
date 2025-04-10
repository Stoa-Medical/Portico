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
  status: string;
  type: string;
  lastActive: string;
  description: string;
  settings: AgentLLMConfig;
  capabilities: string[];
  isActive: boolean;
  model: string;
  apiKey: string;
  createdAt: string;
};

export type Step = {
  id: number;
  agentId: number;
  name: string;
  content: string;
  type: "Python" | "Prompt" | string;
  lastEdited: string;
};

export type RuntimeSession = {
  id: number;
  globalUuid: string;
  requestedByAgentId: number;
  createdTimestamp: string;
  lastUpdatedTimestamp: string;
  runtimeSessionStatus: "queued" | "running" | "completed" | "failed"; // match your enum
  initialData: any; // JSON blob
  latestStepIdx: number;
  latestResult: any | null; // nullable JSON
};

// Omit "id" field for creation:
export type CreateAgentPayload = Omit<Agent, "id">;

// Allow partial Step and Agent updates:
export type UpdateStepPayload = Partial<Step> & { id: number };
export type UpdateAgentPayload = Partial<Agent> & { id: number };

let currentAgents: Agent[] = [
  {
    id: 1,
    name: "Agent Smith",
    status: "Active",
    type: "Assistant",
    lastActive: "2 hours ago",
    description:
      "This is a sample agent description that explains what this agent does and how it works.",
    settings: {
      temperature: 0.7,
      maxTokens: 2048,
      topP: 0.9,
      frequencyPenalty: 0.5,
      presencePenalty: 0.5,
    },
    capabilities: ["Text Generation", "Question Answering", "Summarization"],
    isActive: true,
    model: "gpt-4",
    apiKey: "sk-••••••••••••••••••••••••",
    createdAt: "2023-10-15",
  },
  {
    id: 2,
    name: "Agent Johnson",
    status: "Idle",
    type: "Researcher",
    lastActive: "1 day ago",
    description:
      "Research agent that collects and analyzes information from various sources.",
    settings: {
      temperature: 0.5,
      maxTokens: 4096,
      topP: 0.8,
      frequencyPenalty: 0.3,
      presencePenalty: 0.3,
    },
    capabilities: ["Research", "Data Analysis", "Summarization"],
    isActive: false,
    model: "claude-3-opus",
    apiKey: "sk-••••••••••••••••••••••••",
    createdAt: "2023-11-20",
  },
  {
    id: 3,
    name: "Agent Brown",
    status: "Active",
    type: "Analyst",
    lastActive: "5 minutes ago",
    description: "Specialized in data analysis and visualization.",
    settings: {
      temperature: 0.3,
      maxTokens: 2048,
      topP: 0.7,
      frequencyPenalty: 0.2,
      presencePenalty: 0.2,
    },
    capabilities: ["Data Analysis", "Visualization", "Reporting"],
    isActive: true,
    model: "gpt-3.5-turbo",
    apiKey: "sk-••••••••••••••••••••••••",
    createdAt: "2024-01-05",
  },
];

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
    agentId: 1,
    name: "Data Collection",
    type: "Python",
    lastEdited: "2 hours ago",
    content: defaultPythonContent,
  },
  {
    id: 2,
    agentId: 1,
    name: "Text Analysis",
    type: "Prompt",
    lastEdited: "1 day ago",
    content: `You are an AI assistant that helps with data analysis.
Please analyze the following data and provide insights:
{{data}}

Focus on trends, anomalies, and potential actionable insights.`,
  },
  {
    id: 3,
    agentId: 1,
    name: "Data Visualization",
    type: "Python",
    lastEdited: "3 days ago",
    content: defaultPythonContent,
  },
];

function generateCompletedSessionsForAgent(
  agentId: number,
  count: number,
): RuntimeSession[] {
  return Array.from({ length: count }, (_, i) => {
    const stepCount = currentAgentSteps.filter(
      (step) => step.agentId === agentId,
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
  return currentAgents;
};

export const saveAgent = async (
  agent: CreateAgentPayload,
): Promise<Agent[]> => {
  currentAgents = currentAgents.concat({
    ...agent,
    id: currentAgents.length + 1,
  });
  return currentAgents;
};

export const updateAgent = async (
  updatedAgent: UpdateAgentPayload,
): Promise<Agent[]> => {
  currentAgents = currentAgents.map((agent) =>
    agent.id === updatedAgent.id ? { ...agent, ...updatedAgent } : agent,
  );
  return currentAgents;
};

export const deleteAgent = async (
  agentIdToDelete: number,
): Promise<Agent[]> => {
  currentAgents = currentAgents.filter((agent) => agent.id !== agentIdToDelete);
  return currentAgents;
};

export const getSteps = (agentId: number): Step[] => {
  return currentAgentSteps.filter((step) => step.agentId === agentId);
};

export const saveStep = async (step: Step): Promise<Step[]> => {
  currentAgentSteps = currentAgentSteps.concat({
    ...step,
    id: currentAgentSteps.length + 1,
  });
  return currentAgentSteps;
};

export const updateStep = async (
  updatedStep: UpdateStepPayload,
): Promise<Step[]> => {
  currentAgentSteps = currentAgentSteps.map((step) =>
    step.id === updatedStep.id ? { ...step, ...updatedStep } : step,
  );
  return currentAgentSteps;
};

export const deleteStep = async (stepIdToDelete: number): Promise<Step[]> => {
  currentAgentSteps = currentAgentSteps.filter(
    (step) => step.id !== stepIdToDelete,
  );
  return currentAgentSteps;
};

export const getRuntimeSessions = async (
  agentId: number,
): Promise<RuntimeSession[]> => {
  return currentRuntimeSessions.filter(
    (session) => session.requestedByAgentId === agentId,
  );
};
