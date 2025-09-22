use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Agent represents a composition/orchestration entity that can plan workflows
/// Agents do not execute; they compose and initiate workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub name: String,
    pub description: Option<String>,
    pub capabilities: AgentCapabilities,
    pub policy: AgentPolicy,
}

/// Capabilities define what tools, models, and operations an agent can use
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentCapabilities {
    /// Available tools the agent can include in workflows
    pub tools: Vec<String>,
    /// Available LLM models the agent can use
    pub models: Vec<String>,
    /// Maximum steps allowed in a single workflow
    pub max_steps: Option<u32>,
    /// Whether the agent can create ephemeral workflows
    pub can_create_ephemeral: bool,
    /// Custom capability metadata
    pub metadata: Value,
}

/// Policy defines constraints and rules for agent behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentPolicy {
    /// Maximum number of workflows the agent can create per hour
    pub max_workflows_per_hour: Option<u32>,
    /// Allowed workflow template patterns
    pub allowed_patterns: Vec<String>,
    /// Security constraints
    pub security_constraints: Value,
    /// Custom policy metadata
    pub metadata: Value,
}

impl Agent {
    pub fn new(
        identifiers: IdFields,
        timestamps: TimestampFields,
        name: String,
        description: Option<String>,
    ) -> Self {
        Self {
            identifiers,
            timestamps,
            name,
            description,
            capabilities: AgentCapabilities::default(),
            policy: AgentPolicy::default(),
        }
    }

    /// Check if the agent can use a specific tool
    pub fn can_use_tool(&self, tool_name: &str) -> bool {
        self.capabilities.tools.iter().any(|t| t == tool_name)
    }

    /// Check if the agent can use a specific model
    pub fn can_use_model(&self, model_name: &str) -> bool {
        self.capabilities.models.iter().any(|m| m == model_name)
    }

    /// Validate if a workflow spec complies with this agent's policies
    pub fn validate_workflow_spec(&self, workflow_spec: &Value) -> Result<(), String> {
        // Extract steps from workflow spec
        let steps = workflow_spec
            .get("steps")
            .and_then(|s| s.as_array())
            .ok_or("Workflow must have steps array")?;

        // Check max steps constraint
        if let Some(max_steps) = self.capabilities.max_steps {
            if steps.len() > max_steps as usize {
                return Err(format!(
                    "Workflow has {} steps, but agent is limited to {}",
                    steps.len(),
                    max_steps
                ));
            }
        }

        // Validate each step
        for (i, step) in steps.iter().enumerate() {
            self.validate_step(step).map_err(|e| format!("Step {}: {}", i, e))?;
        }

        Ok(())
    }

    /// Validate a single step against agent capabilities
    fn validate_step(&self, step: &Value) -> Result<(), String> {
        let step_type = step
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or("Step must have a type")?;

        let config = step
            .get("config")
            .ok_or("Step must have a config")?;

        match step_type {
            "tool" => {
                let tool_name = config
                    .get("tool")
                    .and_then(|t| t.as_str())
                    .ok_or("Tool step must specify tool name")?;

                if !self.can_use_tool(tool_name) {
                    return Err(format!("Agent cannot use tool: {}", tool_name));
                }
            }
            "llm" => {
                if let Some(model) = config.get("model").and_then(|m| m.as_str()) {
                    if !self.can_use_model(model) {
                        return Err(format!("Agent cannot use model: {}", model));
                    }
                }
            }
            "branch" => {
                // Branch validation logic would go here
            }
            _ => {
                return Err(format!("Unknown step type: {}", step_type));
            }
        }

        Ok(())
    }
}
