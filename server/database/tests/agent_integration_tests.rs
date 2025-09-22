use models::models::{
    Agent, AgentCapabilities, AgentPolicy, AgentManager,
    WorkflowValidator, ValidationResult, GarbageCollectionPolicy,
    PlanRequest, WorkflowPlanner
};
use models::{IdFields, TimestampFields};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::env;

/// Integration tests for the Agent composition system
///
/// These tests validate the complete flow of agent-driven workflow composition,
/// including planning, validation, execution, and garbage collection.

#[tokio::test]
#[ignore] // Requires database setup
async fn test_agent_composition_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    let pool = get_test_pool().await?;

    // Create test agent
    let agent = create_test_agent();
    let mut manager = AgentManager::new(pool.clone());

    // Test complete composition workflow
    let result = manager.compose_execute_and_monitor(
        1, // agent_id
        "Create a simple data analysis workflow",
        Some(json!({"data_source": "test.csv"})),
        Some(json!({"max_runtime_minutes": 10})),
        true,  // is_ephemeral
        false, // auto_execute
        false  // enforce_strict_validation
    ).await?;

    assert!(result.success);
    assert!(result.plan_validation.is_some());
    assert!(result.composition_session.is_some());

    // Verify ephemeral workflow was created
    let plan_validation = result.plan_validation.unwrap();
    assert!(plan_validation.validation_result.is_valid);

    Ok(())
}

#[tokio::test]
#[ignore] // Requires database setup
async fn test_agent_validation_system() -> Result<(), Box<dyn std::error::Error>> {
    let agent = create_test_agent();

    // Test valid workflow
    let valid_spec = json!({
        "name": "Data Processing Workflow",
        "workflow_type": "analysis",
        "steps": [
            {
                "step_type": "python",
                "config": {
                    "tool": "python",
                    "script": "import pandas as pd; df = pd.read_csv('data.csv')"
                }
            },
            {
                "step_type": "prompt",
                "config": {
                    "model": "gpt-4",
                    "prompt": "Analyze the data trends"
                }
            }
        ]
    });

    let validation = WorkflowValidator::validate_workflow_spec(&agent, &valid_spec, false)?;
    assert!(validation.is_valid);
    assert!(validation.errors.is_empty());

    // Test invalid workflow (dangerous operations)
    let dangerous_spec = json!({
        "name": "Dangerous Workflow",
        "workflow_type": "system",
        "steps": [
            {
                "step_type": "python",
                "config": {
                    "tool": "python",
                    "script": "import os; os.system('rm -rf /')"
                }
            }
        ]
    });

    let dangerous_validation = WorkflowValidator::validate_workflow_spec(&agent, &dangerous_spec, true)?;
    assert!(!dangerous_validation.is_valid);
    assert!(!dangerous_validation.errors.is_empty());
    assert!(dangerous_validation.errors.iter().any(|e| e.contains("dangerous pattern")));

    Ok(())
}

#[tokio::test]
#[ignore] // Requires database setup
async fn test_garbage_collection_system() -> Result<(), Box<dyn std::error::Error>> {
    let pool = get_test_pool().await?;

    // Create custom GC policy for testing
    let gc_policy = GarbageCollectionPolicy {
        ephemeral_retention_hours: 0, // Clean up immediately for testing
        failed_retention_hours: 1,
        max_completed_per_agent: Some(5),
        cleanup_orphaned_steps: true,
        cleanup_old_sessions: true,
        session_log_retention_days: 1,
        custom_rules: Value::Null,
    };

    let mut manager = AgentManager::with_gc_policy(pool.clone(), gc_policy);

    // Start automatic GC with short interval for testing
    manager.start_auto_gc(1).await?;

    // Create ephemeral workflows
    for i in 0..3 {
        let result = manager.compose_execute_and_monitor(
            1,
            &format!("Test ephemeral workflow {}", i),
            None,
            None,
            true,  // is_ephemeral
            false, // auto_execute
            false  // enforce_strict_validation
        ).await?;

        assert!(result.success);
    }

    // Run manual GC
    let gc_stats = manager.run_manual_gc().await?;

    // Verify cleanup occurred (in a real test, we'd check specific counts)
    println!("GC Stats: {:?}", gc_stats);
    assert!(gc_stats.cleanup_duration_ms > 0);

    manager.stop_auto_gc();

    Ok(())
}

#[test]
fn test_agent_capability_validation() {
    let agent = create_test_agent();

    // Test capability checks
    assert!(agent.can_use_tool("python"));
    assert!(agent.can_use_tool("webscrape"));
    assert!(!agent.can_use_tool("nonexistent_tool"));

    assert!(agent.can_use_model("gpt-4"));
    assert!(agent.can_use_model("claude-3"));
    assert!(!agent.can_use_model("nonexistent_model"));

    // Test ephemeral workflow creation capability
    assert!(agent.can_create_ephemeral_workflow());
}

#[test]
fn test_workflow_planning_system() {
    let agent = create_test_agent();

    let plan_request = PlanRequest {
        agent_id: 1,
        objective: "Analyze CSV data and generate insights".to_string(),
        context: Some(json!({"file_path": "data.csv", "columns": ["name", "age", "city"]})),
        constraints: Some(json!({"max_steps": 5, "timeout_minutes": 30})),
        is_ephemeral: false,
    };

    let plan_response = WorkflowPlanner::plan_workflow(&agent, &plan_request).unwrap();

    assert_eq!(plan_response.agent_id, 1);
    assert!(!plan_response.workflow_spec.name.is_empty());
    assert!(!plan_response.workflow_spec.steps.is_empty());
    assert!(plan_response.estimated_duration_minutes > 0);

    // Verify steps are appropriate for the objective
    let has_python_step = plan_response.workflow_spec.steps.iter()
        .any(|step| step.step_type == "python");
    let has_analysis_step = plan_response.workflow_spec.steps.iter()
        .any(|step| step.step_type == "prompt");

    assert!(has_python_step, "Should have Python step for CSV processing");
    assert!(has_analysis_step, "Should have prompt step for analysis");
}

#[test]
fn test_agent_policy_enforcement() {
    let mut agent = create_test_agent();

    // Test max steps constraint
    agent.capabilities.max_steps = Some(2);

    let workflow_with_too_many_steps = json!({
        "name": "Complex Workflow",
        "workflow_type": "analysis",
        "steps": [
            {"step_type": "python", "config": {"script": "step1"}},
            {"step_type": "python", "config": {"script": "step2"}},
            {"step_type": "python", "config": {"script": "step3"}},
            {"step_type": "python", "config": {"script": "step4"}}
        ]
    });

    let validation = WorkflowValidator::validate_workflow_spec(
        &agent,
        &workflow_with_too_many_steps,
        true
    ).unwrap();

    assert!(!validation.is_valid);
    assert!(validation.errors.iter().any(|e| e.contains("exceeds agent limit")));
}

#[test]
fn test_security_constraint_system() {
    let mut agent = create_test_agent();

    // Add security constraints
    agent.policy.security_constraints = json!({
        "blocked_tools": ["dangerous_tool", "system_access"],
        "require_approval": true
    });

    let workflow_with_blocked_tool = json!({
        "name": "Blocked Tool Workflow",
        "workflow_type": "system",
        "steps": [
            {
                "step_type": "python",
                "config": {
                    "tool": "dangerous_tool",
                    "script": "print('hello')"
                }
            }
        ]
    });

    let validation = WorkflowValidator::validate_workflow_spec(
        &agent,
        &workflow_with_blocked_tool,
        true
    ).unwrap();

    assert!(!validation.is_valid);
    assert!(validation.errors.iter().any(|e| e.contains("blocked by security policy")));
}

#[test]
fn test_validation_result_summary() {
    // Test validation result formatting
    let result = ValidationResult {
        is_valid: false,
        errors: vec!["Missing required field".to_string(), "Invalid step type".to_string()],
        warnings: vec!["Performance may be impacted".to_string()],
        requires_approval: true,
    };

    let summary = result.summary();
    assert!(summary.contains("2 errors"));
    assert!(summary.contains("1 warnings"));
    assert!(summary.contains("requires approval"));
    assert!(!result.can_execute());

    let valid_result = ValidationResult {
        is_valid: true,
        errors: vec![],
        warnings: vec![],
        requires_approval: false,
    };

    assert_eq!(valid_result.summary(), "valid");
    assert!(valid_result.can_execute());
}

// Helper functions

async fn get_test_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/portico_test".to_string());

    PgPool::connect(&database_url).await
}

fn create_test_agent() -> Agent {
    Agent {
        identifiers: IdFields {
            local_id: Some(1),
            global_uuid: "test-agent-uuid".to_string(),
        },
        timestamps: TimestampFields {
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
        },
        name: "Test Composition Agent".to_string(),
        description: Some("Agent for testing workflow composition capabilities".to_string()),
        capabilities: AgentCapabilities {
            tools: vec!["python".to_string(), "webscrape".to_string()],
            models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            max_steps: Some(10),
            can_create_ephemeral: true,
            metadata: json!({
                "version": "1.0",
                "test_mode": true
            }),
        },
        policy: AgentPolicy {
            max_workflows_per_hour: Some(100),
            allowed_patterns: vec!["analysis".to_string(), "processing".to_string()],
            security_constraints: json!({
                "require_approval_for_system_access": true,
                "blocked_domains": ["internal.company.com"]
            }),
            metadata: json!({
                "created_for": "integration_testing"
            }),
        },
    }
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validation_performance() {
        let agent = create_test_agent();
        let complex_workflow = json!({
            "name": "Performance Test Workflow",
            "workflow_type": "analysis",
            "steps": (0..50).map(|i| json!({
                "step_type": "python",
                "config": {
                    "tool": "python",
                    "script": format!("print('Step {}')", i)
                }
            })).collect::<Vec<_>>()
        });

        let start = Instant::now();
        let validation = WorkflowValidator::validate_workflow_spec(
            &agent,
            &complex_workflow,
            false
        ).unwrap();
        let duration = start.elapsed();

        // Validation should complete within reasonable time
        assert!(duration.as_millis() < 100, "Validation took too long: {:?}", duration);

        // Should warn about performance due to many steps
        assert!(!validation.warnings.is_empty());
    }

    #[test]
    fn test_planning_performance() {
        let agent = create_test_agent();
        let complex_request = PlanRequest {
            agent_id: 1,
            objective: "Perform comprehensive data analysis including data loading, cleaning, statistical analysis, visualization, and reporting with detailed insights".to_string(),
            context: Some(json!({
                "data_sources": ["file1.csv", "file2.json", "database_table"],
                "analysis_types": ["descriptive", "predictive", "clustering"],
                "output_formats": ["pdf", "html", "json"]
            })),
            constraints: Some(json!({"max_steps": 20})),
            is_ephemeral: false,
        };

        let start = Instant::now();
        let plan = WorkflowPlanner::plan_workflow(&agent, &complex_request).unwrap();
        let duration = start.elapsed();

        // Planning should complete within reasonable time
        assert!(duration.as_millis() < 200, "Planning took too long: {:?}", duration);

        // Should generate comprehensive workflow
        assert!(plan.workflow_spec.steps.len() >= 5);
        assert!(plan.estimated_duration_minutes > 10);
    }
}
