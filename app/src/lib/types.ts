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
 * Agent type - Composer/Orchestrator role
 * Agents create and manage workflows based on their capabilities
 */
export type Agent = {
  id: number;
  global_uuid: string;
  name: string;
  description?: string | null;
  capabilities_json?: AgentCapabilities | null;
  policy_json?: AgentPolicy | null;
  created_at: string;
  updated_at: string;
};

/**
 * Agent capabilities - defines what an agent can do
 */
export type AgentCapabilities = {
  tools: string[];
  models: string[];
  max_steps?: number;
  can_create_ephemeral: boolean;
  can_auto_execute?: boolean;
  workflow_patterns?: string[];
  metadata?: Record<string, any>;
};

/**
 * Agent policy - defines constraints and rules
 */
export type AgentPolicy = {
  max_workflows_per_hour?: number;
  max_ephemeral_workflows?: number;
  allowed_patterns?: string[];
  allowed_workflow_types?: string[];
  execution_constraints?: {
    max_runtime_seconds?: number;
    max_retries?: number;
  };
  security_constraints?: Record<string, any>;
  metadata?: Record<string, any>;
};

/**
 * Signal type for workflow execution requests
 */
export type Signal = {
  id: number;
  global_uuid: string;
  workflow_id?: number | null;
  initiator_agent_id?: number | null;
  user_requested_uuid: string;
  signal_type: "run" | "sync" | "fyi";
  initial_data?: any;
  response_data?: any;
  error_message?: string | null;
  created_at: string;
  updated_at: string;
  rts_id?: number | null;
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
