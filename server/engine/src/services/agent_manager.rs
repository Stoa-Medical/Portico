use shared::models::{Agent, Workflow, DatabaseItem, IdFields};
use sqlx::PgPool;
use tonic::Status;
use serde_json::Value;
use uuid::Uuid;
use super::workflow_planner::{WorkflowPlannerService, PlanWorkflowResult};

/// Engine-side agent management service
pub struct AgentManagerService {
    db_pool: PgPool,
    workflow_planner: WorkflowPlannerService,
}

impl AgentManagerService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            workflow_planner: WorkflowPlannerService::new(db_pool.clone()),
            db_pool,
        }
    }

    /// Create a workflow from agent planning and optionally save it to database
    pub async fn create_workflow_from_plan(
        &mut self,
        agent_id: i32,
        objective: String,
        context: Option<Value>,
        constraints: Option<Value>,
        is_ephemeral: bool,
        save_to_db: bool,
    ) -> Result<WorkflowCreationResult, Status> {
        // 1. Plan the workflow
        let plan_result = self.workflow_planner.plan_workflow_for_agent(
            agent_id,
            objective,
            context,
            constraints,
            is_ephemeral
        ).await?;

        // 2. If validation failed, return without creating workflow
        if !plan_result.validation_result.is_valid {
            return Ok(WorkflowCreationResult {
                workflow_uuid: None,
                plan_result,
                was_saved: false,
                requires_approval: plan_result.validation_result.requires_approval,
            });
        }

        // 3. Create workflow from specification
        let workflow_uuid = Uuid::new_v4().to_string();
        let mut workflow = self.create_workflow_from_spec(
            &plan_result.plan_response.workflow_spec,
            agent_id,
            workflow_uuid.clone(),
            is_ephemeral
        ).await?;

        let mut was_saved = false;

        // 4. Save to database if requested and doesn't require approval
        if save_to_db && !plan_result.validation_result.requires_approval {
            workflow.try_db_create(&self.db_pool).await
                .map_err(|e| Status::internal(format!("Failed to save workflow: {}", e)))?;
            was_saved = true;
        }

        Ok(WorkflowCreationResult {
            workflow_uuid: Some(workflow_uuid),
            plan_result,
            was_saved,
            requires_approval: plan_result.validation_result.requires_approval,
        })
    }

    /// Create a Workflow struct from WorkflowSpec
    async fn create_workflow_from_spec(
        &self,
        workflow_spec: &shared::models::WorkflowSpec,
        agent_id: i32,
        workflow_uuid: String,
        is_ephemeral: bool,
    ) -> Result<Workflow, Status> {
        let now = chrono::Utc::now();

        // Create step IDs array (would be populated with actual step IDs in full implementation)
        let step_ids = Some(Vec::new()); // Placeholder - would create actual steps

        let workflow = Workflow {
            identifiers: IdFields {
                local_id: None, // Will be set by database
                global_uuid: workflow_uuid,
            },
            timestamps: shared::TimestampFields {
                created: now,
                updated: now,
            },
            name: Some(workflow_spec.name.clone()),
            workflow_type: workflow_spec.workflow_type.clone(),
            description: workflow_spec.description.clone(),
            workflow_state: shared::models::workflows::WorkflowState::Inactive,
            workflow_name: Some(workflow_spec.name.clone()),
            created_by_agent_id: Some(agent_id),
            version: "v1".to_string(),
            is_ephemeral,
            step_ids,
        };

        Ok(workflow)
    }

    /// Get agent statistics (temporary mock implementation)
    pub async fn get_agent_stats(&mut self, agent_id: i32) -> Result<AgentStats, Status> {
        // Temporary mock implementation - would use actual database queries in production
        println!("DEBUG: Getting mock agent stats for agent_id: {}", agent_id);

        Ok(AgentStats {
            agent_id,
            total_workflows: 5,     // Mock data
            completed_workflows: 3, // Mock data
            ephemeral_workflows: 1, // Mock data
            cached_agent_count: self.workflow_planner.cached_agent_count(),
        })
    }

    /// Clear caches for performance
    pub fn clear_caches(&mut self) {
        self.workflow_planner.clear_agent_cache();
    }

    /// Invalidate specific agent cache
    pub fn invalidate_agent_cache(&mut self, agent_id: i32) {
        self.workflow_planner.invalidate_agent(agent_id);
    }
}

/// Result of workflow creation operation
#[derive(Debug)]
pub struct WorkflowCreationResult {
    pub workflow_uuid: Option<String>,
    pub plan_result: PlanWorkflowResult,
    pub was_saved: bool,
    pub requires_approval: bool,
}

impl WorkflowCreationResult {
    /// Check if workflow creation was successful
    pub fn is_success(&self) -> bool {
        self.workflow_uuid.is_some() && self.plan_result.validation_result.is_valid
    }

    /// Get user-friendly status message
    pub fn status_message(&self) -> String {
        if !self.plan_result.validation_result.is_valid {
            format!("Workflow planning failed: {}",
                   self.plan_result.validation_result.errors.join(", "))
        } else if self.requires_approval {
            format!("Workflow planned successfully but requires approval: {}",
                   self.plan_result.summary())
        } else if self.was_saved {
            format!("Workflow created and saved: {}", self.plan_result.summary())
        } else {
            format!("Workflow planned but not saved: {}", self.plan_result.summary())
        }
    }
}

/// Agent statistics for monitoring
#[derive(Debug, Clone)]
pub struct AgentStats {
    pub agent_id: i32,
    pub total_workflows: i64,
    pub completed_workflows: i64,
    pub ephemeral_workflows: i64,
    pub cached_agent_count: usize,
}

impl AgentStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_workflows == 0 {
            0.0
        } else {
            self.completed_workflows as f64 / self.total_workflows as f64
        }
    }

    pub fn ephemeral_percentage(&self) -> f64 {
        if self.total_workflows == 0 {
            0.0
        } else {
            self.ephemeral_workflows as f64 / self.total_workflows as f64
        }
    }
}
