// Disabled due to shared library compilation issues
// use portico_shared::models::agents::{Agent, AgentCapabilities, AgentPolicy};
// use portico_shared::models::workflows::{WorkflowPlanner, PlanRequest, PlanResponse, WorkflowValidator};
// use portico_shared::{DatabaseItem, IdFields};
use sqlx::PgPool;
use tonic::Status;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Temporary placeholder types while shared library is being fixed
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRequest {
    pub agent_id: i32,
    pub objective: String,
    pub context: Option<Value>,
    pub constraints: Option<Value>,
    pub is_ephemeral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResponse {
    pub workflow_spec: WorkflowSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: Option<String>,
    pub steps: Vec<StepSpec>,
    pub is_ephemeral: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSpec {
    pub name: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub requires_approval: bool,
}

#[derive(Debug, Clone)]
pub struct PlanValidationResult {
    pub plan_response: PlanResponse,
    pub validation_result: ValidationResult,
}

impl PlanValidationResult {
    pub fn summary(&self) -> String {
        format!("Workflow '{}' with {} steps",
                self.plan_response.workflow_spec.name,
                self.plan_response.workflow_spec.steps.len())
    }
}

/// Engine-side workflow planning service that integrates with shared library components
pub struct WorkflowPlannerService {
    db_pool: PgPool,
    agent_cache: HashMap<i32, Agent>,
}

impl WorkflowPlannerService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            agent_cache: HashMap::new(),
        }
    }

    /// Plan a workflow based on agent capabilities and objective (placeholder implementation)
    pub async fn plan_workflow_for_agent(
        &mut self,
        agent_id: i32,
        objective: String,
        _context: Option<Value>,
        _constraints: Option<Value>,
        _is_ephemeral: bool,
    ) -> Result<PlanValidationResult, Status> {
        // Placeholder implementation until shared library is fixed

        // Create a simple workflow spec
        let workflow_spec = WorkflowSpec {
            name: format!("Workflow for: {}", objective.chars().take(50).collect::<String>()),
            description: Some(format!("Auto-generated workflow for objective: {}", objective)),
            workflow_type: Some("auto_generated".to_string()),
            steps: vec![
                StepSpec {
                    name: "analyze_objective".to_string(),
                    action: format!("Analyze: {}", objective),
                },
                StepSpec {
                    name: "execute_task".to_string(),
                    action: "Execute the planned task".to_string(),
                },
                StepSpec {
                    name: "report_results".to_string(),
                    action: "Report task completion".to_string(),
                },
            ],
            is_ephemeral: _is_ephemeral,
            version: "v1".to_string(),
        };

        let plan_response = PlanResponse { workflow_spec };

        // Simple validation - approve if objective is reasonable
        let validation_result = ValidationResult {
            is_valid: !objective.is_empty() && objective.len() < 500,
            errors: if objective.is_empty() {
                vec!["Objective cannot be empty".to_string()]
            } else if objective.len() >= 500 {
                vec!["Objective too long".to_string()]
            } else {
                vec![]
            },
            requires_approval: objective.to_lowercase().contains("delete")
                || objective.to_lowercase().contains("admin")
                || objective.to_lowercase().contains("critical"),
        };

        // 5. Return comprehensive result
        Ok(PlanValidationResult {
            plan_response,
            validation_result,
        })
    }

    /// Load agent from database with caching (placeholder implementation)
    async fn _load_agent(&mut self, agent_id: i32) -> Result<Agent, String> {
        // Placeholder implementation - just create a simple agent
        Ok(Agent {
            id: agent_id,
            name: format!("Agent {}", agent_id),
        })
    }

    /// Clear agent cache (useful after agent updates)
    pub fn clear_agent_cache(&mut self) {
        self.agent_cache.clear();
    }

    /// Remove specific agent from cache
    pub fn invalidate_agent(&mut self, agent_id: i32) {
        self.agent_cache.remove(&agent_id);
    }

    /// Get cached agent count for monitoring
    pub fn cached_agent_count(&self) -> usize {
        self.agent_cache.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_workflow_planner_service() {
        // Test would require test database setup
        // This is a placeholder for actual test implementation
    }

    #[test]
    fn test_plan_workflow_result() {
        // Test PlanWorkflowResult utility methods
        // This would require creating mock data structures
    }
}
