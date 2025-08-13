use super::types::{Agent, AgentCapabilities, AgentPolicy};
use crate::models::workflows::Workflow;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

/// Workflow specification for agent-composed workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: Option<String>,
    pub steps: Vec<StepSpec>,
    pub is_ephemeral: bool,
    pub version: String,
}

/// Step specification within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSpec {
    pub name: Option<String>,
    pub description: Option<String>,
    pub step_type: String,
    pub config: Value,
}

/// Request for planning a new workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRequest {
    pub agent_id: i32,
    pub objective: String,
    pub context: Option<Value>,
    pub constraints: Option<Value>,
    pub is_ephemeral: bool,
}

/// Response from workflow planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResponse {
    pub workflow_spec: WorkflowSpec,
    pub validation_errors: Vec<String>,
    pub estimated_steps: usize,
    pub requires_approval: bool,
}

/// Core workflow planning and composition engine
pub struct WorkflowPlanner;

impl WorkflowPlanner {
    /// Plan a new workflow based on agent capabilities and request
    pub fn plan_workflow(agent: &Agent, request: &PlanRequest) -> Result<PlanResponse> {
        let mut validation_errors = Vec::new();

        // Validate basic request
        if request.objective.trim().is_empty() {
            validation_errors.push("Objective cannot be empty".to_string());
        }

        // Check ephemeral workflow permission
        if request.is_ephemeral && !agent.capabilities.can_create_ephemeral {
            validation_errors.push("Agent cannot create ephemeral workflows".to_string());
        }

        // Generate basic workflow spec
        let workflow_spec = Self::generate_workflow_spec(agent, request)?;

        // Validate workflow against agent policies
        if let Err(e) = agent.validate_workflow_spec(&serde_json::to_value(&workflow_spec)?) {
            validation_errors.push(e);
        }

        // Check workflow complexity against policies
        let estimated_steps = workflow_spec.steps.len();
        if let Some(max_steps) = agent.capabilities.max_steps {
            if estimated_steps > max_steps as usize {
                validation_errors.push(format!(
                    "Workflow exceeds maximum steps: {} > {}",
                    estimated_steps, max_steps
                ));
            }
        }

        // Determine if approval is required
        let requires_approval = Self::requires_approval(agent, &workflow_spec);

        Ok(PlanResponse {
            workflow_spec,
            validation_errors,
            estimated_steps,
            requires_approval,
        })
    }

    /// Convert a workflow specification into a concrete Workflow model
    pub fn spec_to_workflow(
        spec: &WorkflowSpec,
        created_by_agent_id: Option<i32>
    ) -> Result<Workflow> {
        let global_uuid = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // Create step IDs placeholder (will be filled when steps are created)
        let step_ids: Vec<i32> = Vec::new();

        let workflow = Workflow {
            identifiers: crate::IdFields {
                local_id: None, // Will be set by database
                global_uuid,
            },
            timestamps: crate::TimestampFields {
                created: now,
                updated: now,
            },
            name: Some(spec.name.clone()),
            workflow_type: spec.workflow_type.clone(),
            description: spec.description.clone(),
            workflow_state: crate::models::workflows::WorkflowState::Inactive,
            workflow_name: Some(spec.name.clone()),
            step_ids: Some(step_ids),
            created_by_agent_id,
            is_ephemeral: spec.is_ephemeral,
            version: spec.version.clone(),
        };

        Ok(workflow)
    }

    /// Generate workflow specification based on agent capabilities and request
    fn generate_workflow_spec(agent: &Agent, request: &PlanRequest) -> Result<WorkflowSpec> {
        // Parse the objective to determine required steps
        let steps = Self::parse_objective_to_steps(agent, &request.objective, request.context.as_ref())?;

        Ok(WorkflowSpec {
            name: format!("Agent {} - {}", agent.name, Self::summarize_objective(&request.objective)),
            description: Some(format!("Generated workflow for: {}", request.objective)),
            workflow_type: Some("agent_composed".to_string()),
            steps,
            is_ephemeral: request.is_ephemeral,
            version: "v1".to_string(),
        })
    }

    /// Parse objective into concrete steps based on agent capabilities
    fn parse_objective_to_steps(
        agent: &Agent,
        objective: &str,
        context: Option<&Value>
    ) -> Result<Vec<StepSpec>> {
        let mut steps = Vec::new();

        // Simple objective parsing - in a real implementation, this would be more sophisticated
        let objective_lower = objective.to_lowercase();

        // If objective mentions web scraping
        if objective_lower.contains("scrape") || objective_lower.contains("web") {
            if agent.can_use_tool("webscrape") {
                steps.push(StepSpec {
                    name: Some("Web Scraping".to_string()),
                    description: Some("Extract data from web sources".to_string()),
                    step_type: "webscrape".to_string(),
                    config: json!({
                        "tool": "webscrape",
                        "url": context.and_then(|c| c.get("url")).unwrap_or(&Value::Null)
                    }),
                });
            } else {
                return Err(anyhow!("Objective requires web scraping but agent lacks this capability"));
            }
        }

        // If objective mentions analysis or processing
        if objective_lower.contains("analyze") || objective_lower.contains("process") {
            if !agent.capabilities.models.is_empty() {
                let model = agent.capabilities.models.first().unwrap();
                steps.push(StepSpec {
                    name: Some("Analysis".to_string()),
                    description: Some("Analyze and process data".to_string()),
                    step_type: "prompt".to_string(),
                    config: json!({
                        "model": model,
                        "prompt": format!("Analyze the following objective: {}", objective)
                    }),
                });
            }
        }

        // If objective mentions code execution
        if objective_lower.contains("calculate") || objective_lower.contains("compute") {
            if agent.can_use_tool("python") {
                steps.push(StepSpec {
                    name: Some("Computation".to_string()),
                    description: Some("Perform calculations".to_string()),
                    step_type: "python".to_string(),
                    config: json!({
                        "tool": "python",
                        "script": "# Generated computation script"
                    }),
                });
            }
        }

        // Default fallback - create a basic prompt step
        if steps.is_empty() && !agent.capabilities.models.is_empty() {
            let model = agent.capabilities.models.first().unwrap();
            steps.push(StepSpec {
                name: Some("General Task".to_string()),
                description: Some("Handle the requested objective".to_string()),
                step_type: "prompt".to_string(),
                config: json!({
                    "model": model,
                    "prompt": objective
                }),
            });
        }

        if steps.is_empty() {
            return Err(anyhow!("Could not generate any steps for the given objective"));
        }

        Ok(steps)
    }

    /// Determine if a workflow requires approval based on agent policy
    fn requires_approval(agent: &Agent, spec: &WorkflowSpec) -> bool {
        // Check if workflow matches allowed patterns
        if !agent.policy.allowed_patterns.is_empty() {
            let spec_type = spec.workflow_type.as_deref().unwrap_or("");
            let matches_pattern = agent.policy.allowed_patterns.iter()
                .any(|pattern| spec_type.contains(pattern) || spec.name.contains(pattern));

            if !matches_pattern {
                return true; // Requires approval if no patterns match
            }
        }

        // Check security constraints
        if !agent.policy.security_constraints.is_null() {
            // In a real implementation, this would check specific security rules
            // For now, assume any security constraints require approval
            return true;
        }

        // Default to no approval required for simple workflows
        spec.steps.len() > 5 // Require approval for complex workflows
    }

    /// Create a short summary of the objective for naming
    fn summarize_objective(objective: &str) -> String {
        let words: Vec<&str> = objective.split_whitespace().take(4).collect();
        let summary = words.join(" ");
        if summary.len() > 50 {
            format!("{}...", &summary[..47])
        } else {
            summary
        }
    }

    /// Validate that a plan request is valid for the given agent
    pub fn validate_plan_request(agent: &Agent, request: &PlanRequest) -> Result<()> {
        // Check rate limits
        if let Some(max_workflows) = agent.policy.max_workflows_per_hour {
            // In a real implementation, this would check against actual usage
            // For now, just validate the constraint exists
            if max_workflows == 0 {
                return Err(anyhow!("Agent is not allowed to create workflows"));
            }
        }

        // Validate objective content
        if request.objective.len() > 10000 {
            return Err(anyhow!("Objective is too long (max 10,000 characters)"));
        }

        // Check ephemeral permission
        if request.is_ephemeral && !agent.capabilities.can_create_ephemeral {
            return Err(anyhow!("Agent cannot create ephemeral workflows"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdFields, TimestampFields};

    fn create_test_agent() -> Agent {
        Agent {
            identifiers: IdFields {
                local_id: Some(1),
                global_uuid: "test-uuid".to_string(),
            },
            timestamps: TimestampFields {
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
            },
            name: "Test Agent".to_string(),
            description: Some("Test agent for planning".to_string()),
            capabilities: AgentCapabilities {
                tools: vec!["python".to_string(), "webscrape".to_string()],
                models: vec!["gpt-4".to_string()],
                max_steps: Some(10),
                can_create_ephemeral: true,
                metadata: Value::Null,
            },
            policy: AgentPolicy {
                max_workflows_per_hour: Some(100),
                allowed_patterns: vec!["agent_composed".to_string()],
                security_constraints: Value::Null,
                metadata: Value::Null,
            },
        }
    }

    #[test]
    fn test_plan_basic_workflow() {
        let agent = create_test_agent();
        let request = PlanRequest {
            agent_id: 1,
            objective: "Analyze some data".to_string(),
            context: None,
            constraints: None,
            is_ephemeral: false,
        };

        let result = WorkflowPlanner::plan_workflow(&agent, &request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.workflow_spec.steps.is_empty());
        assert_eq!(response.validation_errors.len(), 0);
    }

    #[test]
    fn test_ephemeral_workflow_validation() {
        let mut agent = create_test_agent();
        agent.capabilities.can_create_ephemeral = false;

        let request = PlanRequest {
            agent_id: 1,
            objective: "Quick analysis".to_string(),
            context: None,
            constraints: None,
            is_ephemeral: true,
        };

        let result = WorkflowPlanner::plan_workflow(&agent, &request);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.validation_errors.is_empty());
        assert!(response.validation_errors[0].contains("ephemeral"));
    }

    #[test]
    fn test_spec_to_workflow_conversion() {
        let spec = WorkflowSpec {
            name: "Test Workflow".to_string(),
            description: Some("Test description".to_string()),
            workflow_type: Some("test".to_string()),
            steps: vec![],
            is_ephemeral: false,
            version: "v1".to_string(),
        };

        let result = WorkflowPlanner::spec_to_workflow(&spec, Some(1));
        assert!(result.is_ok());

        let workflow = result.unwrap();
        assert_eq!(workflow.name, Some("Test Workflow".to_string()));
        assert_eq!(workflow.created_by_agent_id, Some(1));
        assert!(!workflow.is_ephemeral);
    }
}
