use super::{
    gc_policy::{GarbageCollectionPolicy, GarbageCollector, GcScheduler},
    runtime::AgentRuntime,
    types::Agent,
    validator::{WorkflowValidator, ValidationResult},
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

/// Comprehensive agent management system that integrates all agent functionality
pub struct AgentManager {
    runtime: AgentRuntime,
    gc_collector: GarbageCollector,
    gc_scheduler: Option<GcScheduler>,
    agent_cache: HashMap<i32, Agent>,
    validation_cache: HashMap<String, ValidationResult>,
}

impl AgentManager {
    /// Create a new agent manager with default policies
    pub fn new(pool: PgPool) -> Self {
        let runtime = AgentRuntime::new(pool.clone());
        let gc_policy = GarbageCollectionPolicy::default();
        let gc_collector = GarbageCollector::new(pool, gc_policy);

        Self {
            runtime,
            gc_collector,
            gc_scheduler: None,
            agent_cache: HashMap::new(),
            validation_cache: HashMap::new(),
        }
    }

    /// Create agent manager with custom GC policy
    pub fn with_gc_policy(pool: PgPool, gc_policy: GarbageCollectionPolicy) -> Self {
        let runtime = AgentRuntime::new(pool.clone());
        let gc_collector = GarbageCollector::new(pool, gc_policy);

        Self {
            runtime,
            gc_collector,
            gc_scheduler: None,
            agent_cache: HashMap::new(),
            validation_cache: HashMap::new(),
        }
    }

    /// Start automatic garbage collection with specified interval
    pub async fn start_auto_gc(&mut self, interval_hours: u64) -> Result<()> {
        if self.gc_scheduler.is_some() {
            return Err(anyhow!("GC scheduler is already running"));
        }

        let scheduler = GcScheduler::new(
            GarbageCollector::new(self.gc_collector.get_policy().clone(), self.runtime.pool.clone()),
            interval_hours,
        );

        // Start the scheduler in a background task
        let scheduler_clone = scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = scheduler_clone.start_scheduled_cleanup().await {
                eprintln!("[AGENT MANAGER] GC scheduler error: {}", e);
            }
        });

        self.gc_scheduler = Some(scheduler);
        Ok(())
    }

    /// Stop automatic garbage collection
    pub fn stop_auto_gc(&mut self) {
        self.gc_scheduler = None;
    }

    /// Plan and validate a workflow with full validation
    pub async fn plan_and_validate_workflow(
        &mut self,
        agent_id: i32,
        objective: &str,
        context: Option<Value>,
        constraints: Option<Value>,
        is_ephemeral: bool,
        enforce_strict_validation: bool,
    ) -> Result<PlanValidationResult> {
        // Load agent (with caching)
        let agent = self.get_agent_cached(agent_id).await?;

        // Create plan request
        let plan_request = super::planner::PlanRequest {
            agent_id,
            objective: objective.to_string(),
            context,
            constraints,
            is_ephemeral,
        };

        // Plan the workflow
        let plan_response = super::planner::WorkflowPlanner::plan_workflow(&agent, &plan_request)?;

        // Validate the planned workflow
        let workflow_spec_json = serde_json::to_value(&plan_response.workflow_spec)?;
        let validation_result = WorkflowValidator::validate_workflow_spec(
            &agent,
            &workflow_spec_json,
            enforce_strict_validation,
        )?;

        // Check agent constraints
        self.runtime.check_agent_constraints(agent_id).await?;

        Ok(PlanValidationResult {
            plan_response,
            validation_result,
            agent_constraints_passed: true,
        })
    }

    /// Execute a complete composition workflow with validation and cleanup
    pub async fn compose_execute_and_monitor(
        &mut self,
        agent_id: i32,
        objective: &str,
        context: Option<Value>,
        constraints: Option<Value>,
        is_ephemeral: bool,
        auto_execute: bool,
        enforce_strict_validation: bool,
    ) -> Result<CompositionResult> {
        // Plan and validate
        let plan_validation = self
            .plan_and_validate_workflow(
                agent_id,
                objective,
                context.clone(),
                constraints.clone(),
                is_ephemeral,
                enforce_strict_validation,
            )
            .await?;

        // Check if validation passed
        if !plan_validation.validation_result.is_valid {
            return Ok(CompositionResult {
                success: false,
                message: format!(
                    "Validation failed: {}",
                    plan_validation.validation_result.errors.join(", ")
                ),
                plan_validation: Some(plan_validation),
                composition_session: None,
                gc_stats: None,
            });
        }

        // Check if approval is required
        if plan_validation.validation_result.requires_approval && auto_execute {
            return Ok(CompositionResult {
                success: false,
                message: "Workflow requires approval but auto_execute was requested".to_string(),
                plan_validation: Some(plan_validation),
                composition_session: None,
                gc_stats: None,
            });
        }

        // Create plan request for execution
        let plan_request = super::planner::PlanRequest {
            agent_id,
            objective: objective.to_string(),
            context,
            constraints,
            is_ephemeral,
        };

        // Execute composition
        let composition_session = self
            .runtime
            .compose_and_execute(&plan_request, auto_execute)
            .await?;

        // Run cleanup if needed
        let gc_stats = if is_ephemeral {
            Some(self.gc_collector.run_full_cleanup().await?)
        } else {
            None
        };

        // Clear cache for this agent (capabilities might have changed)
        self.agent_cache.remove(&agent_id);

        Ok(CompositionResult {
            success: true,
            message: "Composition completed successfully".to_string(),
            plan_validation: Some(plan_validation),
            composition_session: Some(composition_session),
            gc_stats,
        })
    }

    /// Get agent with caching
    async fn get_agent_cached(&mut self, agent_id: i32) -> Result<Agent> {
        if let Some(agent) = self.agent_cache.get(&agent_id) {
            return Ok(agent.clone());
        }

        let agent = self.runtime.load_agent(agent_id).await?;
        self.agent_cache.insert(agent_id, agent.clone());
        Ok(agent)
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) {
        self.agent_cache.clear();
        self.validation_cache.clear();
    }

    /// Get agent statistics with additional metrics
    pub async fn get_comprehensive_agent_stats(&self, agent_id: i32) -> Result<ComprehensiveAgentStats> {
        let basic_stats = self.runtime.get_agent_statistics(agent_id).await?;
        let cleanup_candidates = self.gc_collector.get_cleanup_candidates().await?;

        // Calculate additional metrics
        let validation_cache_hits = self.validation_cache.len();
        let agent_cache_hits = self.agent_cache.len();

        Ok(ComprehensiveAgentStats {
            basic_stats,
            cleanup_candidates,
            validation_cache_hits,
            agent_cache_hits,
            gc_policy: self.gc_collector.get_policy().clone(),
        })
    }

    /// Update garbage collection policy
    pub fn update_gc_policy(&mut self, new_policy: GarbageCollectionPolicy) {
        self.gc_collector.update_policy(new_policy);
    }

    /// Run manual garbage collection
    pub async fn run_manual_gc(&self) -> Result<super::gc_policy::GcStatistics> {
        self.gc_collector.run_full_cleanup().await
    }

    /// Validate agent configuration
    pub fn validate_agent_config(&self, agent: &Agent) -> Result<AgentConfigValidation> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate capabilities
        if agent.capabilities.tools.is_empty() && agent.capabilities.models.is_empty() {
            errors.push("Agent must have at least one tool or model capability".to_string());
        }

        // Check for dangerous tool combinations
        if agent.capabilities.tools.contains(&"python".to_string())
            && agent.capabilities.tools.contains(&"webscrape".to_string()) {
            warnings.push("Agent has both Python and webscrape capabilities - ensure security policies are configured".to_string());
        }

        // Validate policy constraints
        if let Some(max_steps) = agent.capabilities.max_steps {
            if max_steps == 0 {
                errors.push("max_steps cannot be 0".to_string());
            } else if max_steps > 100 {
                warnings.push("max_steps is very high - may impact performance".to_string());
            }
        }

        if let Some(max_workflows) = agent.policy.max_workflows_per_hour {
            if max_workflows > 1000 {
                warnings.push("max_workflows_per_hour is very high - may impact system performance".to_string());
            }
        }

        Ok(AgentConfigValidation {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
}

/// Result of planning and validation
#[derive(Debug, Clone)]
pub struct PlanValidationResult {
    pub plan_response: super::planner::PlanResponse,
    pub validation_result: ValidationResult,
    pub agent_constraints_passed: bool,
}

/// Result of complete composition operation
#[derive(Debug)]
pub struct CompositionResult {
    pub success: bool,
    pub message: String,
    pub plan_validation: Option<PlanValidationResult>,
    pub composition_session: Option<super::runtime::CompositionSession>,
    pub gc_stats: Option<super::gc_policy::GcStatistics>,
}

/// Comprehensive agent statistics
#[derive(Debug, Clone)]
pub struct ComprehensiveAgentStats {
    pub basic_stats: super::runtime::AgentStatistics,
    pub cleanup_candidates: super::gc_policy::CleanupCandidates,
    pub validation_cache_hits: usize,
    pub agent_cache_hits: usize,
    pub gc_policy: GarbageCollectionPolicy,
}

/// Agent configuration validation result
#[derive(Debug, Clone)]
pub struct AgentConfigValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdFields, TimestampFields};

    // Note: These tests would require a test database setup
    // In a real implementation, we'd use sqlx-test or similar

    #[tokio::test]
    #[ignore] // Requires database setup
    async fn test_agent_manager_creation() {
        // Test would create a test database pool and manager
        // This is a placeholder for the test structure
    }

    #[test]
    fn test_agent_config_validation() {
        let agent = Agent {
            identifiers: IdFields {
                local_id: Some(1),
                global_uuid: "test-uuid".to_string(),
            },
            timestamps: TimestampFields {
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
            },
            name: "Test Agent".to_string(),
            description: Some("Test agent".to_string()),
            capabilities: super::types::AgentCapabilities {
                tools: vec!["python".to_string()],
                models: vec!["gpt-4".to_string()],
                max_steps: Some(10),
                can_create_ephemeral: true,
                metadata: Value::Null,
            },
            policy: super::types::AgentPolicy {
                max_workflows_per_hour: Some(100),
                allowed_patterns: vec!["test".to_string()],
                security_constraints: Value::Null,
                metadata: Value::Null,
            },
        };

        let pool = PgPool::connect("").await.unwrap(); // This would be a real connection in tests
        let manager = AgentManager::new(pool);
        let validation = manager.validate_agent_config(&agent).unwrap();

        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }
}
