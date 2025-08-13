/**
 * Result type for functional error handling
 * A monadic container for handling success/failure outcomes
 */
export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

/**
 * Workflow type - represents an executable workflow unit
 * Workflows execute tasks and are created by Agents
 */
export type Workflow = {
  id: number;
  name: string | null;
  description?: string | null;
  workflow_state: "inactive" | "stable" | "unstable";
  created_by_agent_id?: number | null;
  workflow_type?: string | null;
  is_ephemeral?: boolean;
  step_ids?: number[];
  created_at: string;
  updated_at: string;
};

/**
 * Step type - updated to belong to workflows instead of agents
 */
export type Step = {
  id: number | string;
  global_uuid: string;
  workflow_id: number; // Changed from agent_id
  name: string;
  description?: string;
  step_content: string;
  step_type: "python" | "prompt" | "webscrape";
  created_at?: string;
  updated_at?: string;
};

/**
 * Runtime Session type - updated for workflow execution
 */
export type RuntimeSession = {
  id: number;
  global_uuid: string;
  workflow_id: number; // Changed from requested_by_agent_id
  initiator_agent_id?: number | null; // Optional metadata about which agent initiated
  created_at: string;
  updated_at: string;
  rts_status: "queued" | "running" | "completed" | "failed";
  initial_data: any;
  latest_step_idx: number;
  latest_result: any | null;
  step_ids?: number[];
  step_execution_times?: number[];
  total_execution_time?: number;
};

/**
 * Agent type - updated for composition/orchestration role
 */
export type Agent = {
  id: number;
  name: string;
  agent_state: string;
  type: string;
  description: string;
  owner_id: string;
  capabilities?: AgentCapabilities;
  policy?: AgentPolicy;
  created_at?: string;
  updated_at?: string;
};

/**
 * Agent capabilities - defines what an agent can do
 */
export type AgentCapabilities = {
  tools: string[];
  models: string[];
  max_steps?: number;
  can_create_ephemeral: boolean;
  metadata?: Record<string, any>;
};

/**
 * Agent policy - defines constraints and rules
 */
export type AgentPolicy = {
  max_workflows_per_hour?: number;
  allowed_patterns: string[];
  security_constraints?: Record<string, any>;
  metadata?: Record<string, any>;
};

/**
 * Workflow composition request payload
 */
export type WorkflowCompositionRequest = {
  agent_id: number;
  objective: string;
  context?: Record<string, any>;
  constraints?: Record<string, any>;
  is_ephemeral?: boolean;
  auto_execute?: boolean;
};
