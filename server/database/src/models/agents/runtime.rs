use super::planner::{PlanRequest, PlanResponse, WorkflowPlanner, WorkflowSpec};
use super::types::Agent;
use crate::models::{Signal, SignalType, Workflow};
use crate::{DatabaseItem, IdFields, TimestampFields};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// Agent runtime manages the composition and execution lifecycle
pub struct AgentRuntime {
    pool: PgPool,
}

/// Execution request for an agent-composed workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub agent_id: i32,
    pub workflow_uuid: String,
    pub signal_type: SignalType,
    pub initial_data: Option<Value>,
    pub user_requested_uuid: String,
}

/// Result of execution initiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
    pub signal_uuid: Option<String>,
    pub runtime_session_uuid: Option<String>,
}

/// Workflow composition and execution coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionSession {
    pub session_id: String,
    pub agent_id: i32,
    pub objective: String,
    pub planned_workflows: Vec<WorkflowSpec>,
    pub created_workflows: Vec<String>, // UUIDs of created workflows
    pub execution_status: CompositionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionStatus {
    Planning,
    WaitingApproval,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

impl AgentRuntime {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Full composition workflow: plan, validate, create, and optionally execute
    pub async fn compose_and_execute(
        &self,
        request: &PlanRequest,
        auto_execute: bool,
    ) -> Result<CompositionSession> {
        // Load the agent
        let agent = self.load_agent(request.agent_id).await?;

        // Validate the request
        WorkflowPlanner::validate_plan_request(&agent, request)?;

        // Plan the workflow
        let plan_response = WorkflowPlanner::plan_workflow(&agent, request)?;

        // Check for validation errors
        if !plan_response.validation_errors.is_empty() {
            return Err(anyhow!(
                "Workflow validation failed: {}",
                plan_response.validation_errors.join(", ")
            ));
        }

        // Create composition session
        let mut session = CompositionSession {
            session_id: Uuid::new_v4().to_string(),
            agent_id: request.agent_id,
            objective: request.objective.clone(),
            planned_workflows: vec![plan_response.workflow_spec],
            created_workflows: Vec::new(),
            execution_status: CompositionStatus::Planning,
            created_at: chrono::Utc::now(),
        };

        // If approval is required, set status and return
        if plan_response.requires_approval {
            session.execution_status = CompositionStatus::WaitingApproval;
            return Ok(session);
        }

        // Create the workflow(s)
        for workflow_spec in &session.planned_workflows {
            let workflow = WorkflowPlanner::spec_to_workflow(workflow_spec, Some(request.agent_id))?;

            // Save workflow to database
            workflow.try_db_create(&self.pool).await?;

            // Get the created workflow UUID
            let created_workflow = self.get_workflow_by_uuid(&workflow.identifiers.global_uuid).await?;
            session.created_workflows.push(created_workflow.identifiers.global_uuid.clone());

            // Create steps for the workflow
            self.create_steps_for_workflow(&created_workflow, workflow_spec).await?;
        }

        // Execute if requested and no approval needed
        if auto_execute {
            session.execution_status = CompositionStatus::Executing;

            for workflow_uuid in &session.created_workflows {
                let execution_request = ExecutionRequest {
                    agent_id: request.agent_id,
                    workflow_uuid: workflow_uuid.clone(),
                    signal_type: SignalType::Run,
                    initial_data: request.context.clone(),
                    user_requested_uuid: Uuid::new_v4().to_string(),
                };

                self.initiate_execution(&execution_request).await?;
            }

            session.execution_status = CompositionStatus::Completed;
        }

        Ok(session)
    }

    /// Initiate execution of a workflow by creating and processing a signal
    pub async fn initiate_execution(&self, request: &ExecutionRequest) -> Result<ExecutionResult> {
        // Load the workflow
        let workflow = self.get_workflow_by_uuid(&request.workflow_uuid).await?;

        // Create a signal to trigger execution
        let signal = Signal {
            identifiers: IdFields {
                local_id: None, // Will be set by database
                global_uuid: Uuid::new_v4().to_string(),
            },
            timestamps: TimestampFields {
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
            },
            workflow_id: workflow.identifiers.local_id,
            initiator_agent_id: Some(request.agent_id),
            rts_id: None, // Will be set when runtime session is created
            user_requested_uuid: request.user_requested_uuid.clone(),
            signal_type: request.signal_type.clone(),
            initial_data: request.initial_data.clone(),
            response_data: None,
            error_message: None,
        };

        // Save signal to database
        signal.try_db_create(&self.pool).await?;

        // At this point, the signal would be picked up by the bridge/engine
        // In a real implementation, we might trigger the bridge directly

        Ok(ExecutionResult {
            success: true,
            message: "Execution initiated successfully".to_string(),
            signal_uuid: Some(signal.identifiers.global_uuid),
            runtime_session_uuid: None, // Will be set by the engine
        })
    }

    /// Load an agent from the database
    async fn load_agent(&self, agent_id: i32) -> Result<Agent> {
        let id_fields = IdFields {
            local_id: Some(agent_id),
            global_uuid: String::new(), // Not needed for lookup by local_id
        };

        Agent::try_db_select_by_id(&self.pool, &id_fields)
            .await?
            .ok_or_else(|| anyhow!("Agent with ID {} not found", agent_id))
    }

    /// Get a workflow by its UUID
    async fn get_workflow_by_uuid(&self, uuid: &str) -> Result<Workflow> {
        let id_fields = IdFields {
            local_id: None,
            global_uuid: uuid.to_string(),
        };

        Workflow::try_db_select_by_id(&self.pool, &id_fields)
            .await?
            .ok_or_else(|| anyhow!("Workflow with UUID {} not found", uuid))
    }

    /// Create steps for a workflow based on the specification
    async fn create_steps_for_workflow(
        &self,
        _workflow: &Workflow,
        spec: &WorkflowSpec,
    ) -> Result<()> {
        use crate::models::Step;
        use crate::steps::StepType;
        use crate::DatabaseItem;

        for step_spec in spec.steps.iter() {
            // Convert step type string to enum
            let step_type = match step_spec.step_type.as_str() {
                "python" => StepType::Python,
                "llm" => StepType::LLM,
                "transform" => StepType::Transform,
                "validate" => StepType::Validate,
                "fhir" => StepType::FHIR,
                _ => return Err(anyhow!("Unknown step type: {}", step_spec.step_type)),
            };

            let step = Step {
                identifiers: IdFields {
                    local_id: None, // Will be set by database
                    global_uuid: Uuid::new_v4().to_string(),
                },
                timestamps: TimestampFields {
                    created: chrono::Utc::now(),
                    updated: chrono::Utc::now(),
                },
                name: step_spec.name.clone(),
                description: step_spec.description.clone(),
                step_type,
                config: Some(step_spec.config.clone()),
                step_order: None,
            };

            // Save step to database
            step.try_db_create(&self.pool).await?;
        }

        Ok(())
    }

    /// Check agent rate limits and constraints
    pub async fn check_agent_constraints(&self, agent_id: i32) -> Result<()> {
        let agent = self.load_agent(agent_id).await?;

        // Check hourly workflow creation limit
        if let Some(max_workflows) = agent.policy.max_workflows_per_hour {
            let count = self.count_workflows_created_last_hour(agent_id).await?;
            if count >= max_workflows as i64 {
                return Err(anyhow!(
                    "Agent has exceeded hourly workflow creation limit: {} >= {}",
                    count,
                    max_workflows
                ));
            }
        }

        Ok(())
    }

    /// Count workflows created by an agent in the last hour
    async fn count_workflows_created_last_hour(&self, agent_id: i32) -> Result<i64> {
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflows WHERE created_by_agent_id = $1 AND created_at > $2",
            agent_id,
            one_hour_ago
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    /// Cleanup ephemeral workflows that have completed execution
    pub async fn garbage_collect_ephemeral_workflows(&self) -> Result<u64> {
        // Find completed ephemeral workflows
        let completed_ephemeral = sqlx::query!(
            r#"
            SELECT w.id, w.global_uuid
            FROM workflows w
            JOIN runtime_sessions rs ON rs.workflow_id = w.id
            WHERE w.is_ephemeral = true
            AND rs.rts_status = 'completed'
            AND w.created_at < NOW() - INTERVAL '1 hour'
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut cleaned_count = 0u64;

        for record in completed_ephemeral {
            // Delete associated steps
            sqlx::query!("DELETE FROM steps WHERE workflow_id = $1", record.id)
                .execute(&self.pool)
                .await?;

            // Delete runtime sessions
            sqlx::query!("DELETE FROM runtime_sessions WHERE workflow_id = $1", record.id)
                .execute(&self.pool)
                .await?;

            // Delete signals
            sqlx::query!("DELETE FROM signals WHERE workflow_id = $1", record.id)
                .execute(&self.pool)
                .await?;

            // Delete the workflow itself
            sqlx::query!("DELETE FROM workflows WHERE id = $1", record.id)
                .execute(&self.pool)
                .await?;

            cleaned_count += 1;
        }

        Ok(cleaned_count)
    }

    /// Get composition statistics for an agent
    pub async fn get_agent_statistics(&self, agent_id: i32) -> Result<AgentStatistics> {
        // Workflows created in the last 24 hours
        let workflows_today = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflows WHERE created_by_agent_id = $1 AND created_at > NOW() - INTERVAL '24 hours'",
            agent_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Total workflows created
        let total_workflows = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflows WHERE created_by_agent_id = $1",
            agent_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        // Active workflows (not completed)
        let active_workflows = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM workflows w
            WHERE w.created_by_agent_id = $1
            AND NOT EXISTS (
                SELECT 1 FROM runtime_sessions rs
                WHERE rs.workflow_id = w.id AND rs.rts_status = 'completed'
            )
            "#,
            agent_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(AgentStatistics {
            agent_id,
            workflows_created_today: workflows_today,
            total_workflows_created: total_workflows,
            active_workflows,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Agent performance and usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatistics {
    pub agent_id: i32,
    pub workflows_created_today: i64,
    pub total_workflows_created: i64,
    pub active_workflows: i64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a test database setup
    // In a real implementation, we'd use sqlx-test or similar

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_agent_runtime_creation() {
        // Test would create a test database pool and runtime
        // This is a placeholder for the test structure
    }
}
