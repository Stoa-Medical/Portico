use shared::models::{
    Agent, WorkflowPlanner, PlanRequest, PlanResponse, WorkflowValidator,
    DatabaseItem, IdFields
};
use sqlx::PgPool;
use tonic::Status;
use serde_json::Value;
use std::collections::HashMap;

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

    /// Plan a workflow based on agent capabilities and objective
    pub async fn plan_workflow_for_agent(
        &mut self,
        agent_id: i32,
        objective: String,
        context: Option<Value>,
        constraints: Option<Value>,
        is_ephemeral: bool,
    ) -> Result<PlanWorkflowResult, Status> {
        // 1. Load agent from database (with caching)
        let agent = self.load_agent(agent_id).await
            .map_err(|e| Status::internal(format!("Failed to load agent {}: {}", agent_id, e)))?;

        // 2. Create plan request
        let plan_request = PlanRequest {
            agent_id,
            objective,
            context,
            constraints,
            is_ephemeral,
        };

        // 3. Use shared library WorkflowPlanner to create workflow spec
        let plan_response = WorkflowPlanner::plan_workflow(&agent, &plan_request)
            .map_err(|e| Status::internal(format!("Workflow planning failed: {}", e)))?;

        // 4. Validate the planned workflow
        let workflow_spec_json = serde_json::to_value(&plan_response.workflow_spec)
            .map_err(|e| Status::internal(format!("Failed to serialize workflow spec: {}", e)))?;

        let validation_result = WorkflowValidator::validate_workflow_spec(
            &agent,
            &workflow_spec_json,
            true // enforce strict validation in engine
        ).map_err(|e| Status::internal(format!("Workflow validation failed: {}", e)))?;

        // 5. Return comprehensive result
        Ok(PlanWorkflowResult {
            plan_response,
            validation_result,
            agent_name: agent.name.clone(),
        })
    }

    /// Load agent from database with caching
    async fn load_agent(&mut self, agent_id: i32) -> Result<Agent, sqlx::Error> {
        // Check cache first
        if let Some(cached_agent) = self.agent_cache.get(&agent_id) {
            return Ok(cached_agent.clone());
        }

        // Load from database
        let id_fields = IdFields {
            local_id: Some(agent_id),
            global_uuid: String::new(), // We'll search by local_id
        };

        let agent_opt = Agent::try_db_select_by_id(&self.db_pool, &id_fields).await?;

        match agent_opt {
            Some(agent) => {
                // Cache the agent
                self.agent_cache.insert(agent_id, agent.clone());
                Ok(agent)
            },
            None => Err(sqlx::Error::RowNotFound),
        }
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

/// Result of workflow planning operation
#[derive(Debug)]
pub struct PlanWorkflowResult {
    pub plan_response: PlanResponse,
    pub validation_result: shared::models::ValidationResult,
    pub agent_name: String,
}

impl PlanWorkflowResult {
    /// Check if the workflow can be auto-executed
    pub fn can_auto_execute(&self) -> bool {
        self.validation_result.is_valid && !self.validation_result.requires_approval
    }

    /// Get summary of planning results
    pub fn summary(&self) -> String {
        format!(
            "Agent '{}' planned workflow '{}' with {} steps. Validation: {}",
            self.agent_name,
            self.plan_response.workflow_spec.name,
            self.plan_response.workflow_spec.steps.len(),
            self.validation_result.summary()
        )
    }

    /// Get list of validation errors
    pub fn validation_errors(&self) -> Vec<String> {
        self.validation_result.errors.clone()
    }

    /// Get list of validation warnings
    pub fn validation_warnings(&self) -> Vec<String> {
        self.validation_result.warnings.clone()
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
