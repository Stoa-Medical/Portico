export const getAgents = async () => {
  return [
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
};

export const saveAgent = async (agentConfig) => {};

export const getAgentSteps = (agentId) => {
  return [
    {
      id: 1,
      name: "Data Collection",
      type: "Python",
      lastEdited: "2 hours ago",
    },
    {
      id: 2,
      name: "Text Analysis",
      type: "Prompt",
      lastEdited: "1 day ago",
    },
    {
      id: 3,
      name: "Data Visualization",
      type: "Python",
      lastEdited: "3 days ago",
    },
  ];
};
