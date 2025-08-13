use super::types::{Agent, AgentCapabilities, AgentPolicy};
use anyhow::{anyhow, Result};
use serde_json::Value;

/// Comprehensive workflow validation for agent-composed workflows
pub struct WorkflowValidator;

impl WorkflowValidator {
    /// Validate a complete workflow specification against all policies and constraints
    pub fn validate_workflow_spec(
        agent: &Agent,
        workflow_spec: &Value,
        enforce_strict: bool,
    ) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Basic structure validation
        if let Err(e) = Self::validate_structure(workflow_spec) {
            errors.push(e.to_string());
        }

        // Agent capability validation
        if let Err(e) = Self::validate_capabilities(agent, workflow_spec) {
            errors.push(e.to_string());
        }

        // Policy compliance validation
        if let Err(e) = Self::validate_policies(agent, workflow_spec, enforce_strict) {
            if enforce_strict {
                errors.push(e.to_string());
            } else {
                warnings.push(e.to_string());
            }
        }

        // Security validation
        if let Err(e) = Self::validate_security(agent, workflow_spec) {
            errors.push(e.to_string());
        }

        // Performance validation
        if let Err(e) = Self::validate_performance(workflow_spec) {
            warnings.push(e.to_string());
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            requires_approval: Self::requires_approval(agent, workflow_spec, &errors, &warnings),
        })
    }

    /// Validate basic workflow structure
    fn validate_structure(workflow_spec: &Value) -> Result<()> {
        let spec = workflow_spec.as_object()
            .ok_or_else(|| anyhow!("Workflow spec must be an object"))?;

        // Required fields
        if !spec.contains_key("name") {
            return Err(anyhow!("Workflow must have a name"));
        }

        if !spec.contains_key("steps") {
            return Err(anyhow!("Workflow must have steps"));
        }

        let steps = spec.get("steps")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("Steps must be an array"))?;

        if steps.is_empty() {
            return Err(anyhow!("Workflow must have at least one step"));
        }

        // Validate each step structure
        for (i, step) in steps.iter().enumerate() {
            Self::validate_step_structure(step, i)?;
        }

        Ok(())
    }

    /// Validate individual step structure
    fn validate_step_structure(step: &Value, index: usize) -> Result<()> {
        let step_obj = step.as_object()
            .ok_or_else(|| anyhow!("Step {} must be an object", index))?;

        if !step_obj.contains_key("step_type") {
            return Err(anyhow!("Step {} must have a step_type", index));
        }

        if !step_obj.contains_key("config") {
            return Err(anyhow!("Step {} must have a config", index));
        }

        let step_type = step_obj.get("step_type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Step {} step_type must be a string", index))?;

        match step_type {
            "python" | "prompt" | "webscrape" => {},
            _ => return Err(anyhow!("Step {} has unknown step_type: {}", index, step_type)),
        }

        Ok(())
    }

    /// Validate agent capabilities against workflow requirements
    fn validate_capabilities(agent: &Agent, workflow_spec: &Value) -> Result<()> {
        let steps = workflow_spec.get("steps")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("Invalid workflow structure"))?;

        for (i, step) in steps.iter().enumerate() {
            Self::validate_step_capabilities(agent, step, i)?;
        }

        Ok(())
    }

    /// Validate agent capabilities for a specific step
    fn validate_step_capabilities(agent: &Agent, step: &Value, index: usize) -> Result<()> {
        let step_obj = step.as_object()
            .ok_or_else(|| anyhow!("Invalid step structure"))?;

        let step_type = step_obj.get("step_type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing step_type"))?;

        let config = step_obj.get("config")
            .ok_or_else(|| anyhow!("Missing config"))?;

        match step_type {
            "python" => {
                if !agent.can_use_tool("python") {
                    return Err(anyhow!("Step {}: Agent cannot use Python tool", index));
                }
            },
            "webscrape" => {
                if !agent.can_use_tool("webscrape") {
                    return Err(anyhow!("Step {}: Agent cannot use webscrape tool", index));
                }
            },
            "prompt" => {
                if let Some(model) = config.get("model").and_then(|m| m.as_str()) {
                    if !agent.can_use_model(model) {
                        return Err(anyhow!("Step {}: Agent cannot use model: {}", index, model));
                    }
                }
            },
            _ => return Err(anyhow!("Step {}: Unknown step type: {}", index, step_type)),
        }

        Ok(())
    }

    /// Validate policy compliance
    fn validate_policies(agent: &Agent, workflow_spec: &Value, _enforce_strict: bool) -> Result<()> {
        let steps = workflow_spec.get("steps")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("Invalid workflow structure"))?;

        // Check step count limits
        if let Some(max_steps) = agent.capabilities.max_steps {
            if steps.len() > max_steps as usize {
                return Err(anyhow!(
                    "Workflow has {} steps, exceeds agent limit of {}",
                    steps.len(),
                    max_steps
                ));
            }
        }

        // Check allowed patterns
        if !agent.policy.allowed_patterns.is_empty() {
            let workflow_type = workflow_spec.get("workflow_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            let name = workflow_spec.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");

            let matches_pattern = agent.policy.allowed_patterns.iter()
                .any(|pattern| workflow_type.contains(pattern) || name.contains(pattern));

            if !matches_pattern {
                return Err(anyhow!(
                    "Workflow type '{}' does not match allowed patterns: {:?}",
                    workflow_type,
                    agent.policy.allowed_patterns
                ));
            }
        }

        Ok(())
    }

    /// Validate security constraints
    fn validate_security(agent: &Agent, workflow_spec: &Value) -> Result<()> {
        // Check for potentially dangerous operations
        let steps = workflow_spec.get("steps")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("Invalid workflow structure"))?;

        for (i, step) in steps.iter().enumerate() {
            Self::validate_step_security(step, i)?;
        }

        // Apply agent-specific security constraints
        if !agent.policy.security_constraints.is_null() {
            // In a real implementation, this would apply complex security rules
            // For now, we'll do basic validation
            let constraints = &agent.policy.security_constraints;

            if let Some(blocked_tools) = constraints.get("blocked_tools").and_then(|t| t.as_array()) {
                for step in steps {
                    if let Some(config) = step.get("config") {
                        if let Some(tool) = config.get("tool").and_then(|t| t.as_str()) {
                            for blocked in blocked_tools {
                                if let Some(blocked_tool) = blocked.as_str() {
                                    if tool == blocked_tool {
                                        return Err(anyhow!("Tool '{}' is blocked by security policy", tool));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate step-level security
    fn validate_step_security(step: &Value, index: usize) -> Result<()> {
        let step_obj = step.as_object()
            .ok_or_else(|| anyhow!("Invalid step structure"))?;

        let step_type = step_obj.get("step_type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing step_type"))?;

        let config = step_obj.get("config")
            .ok_or_else(|| anyhow!("Missing config"))?;

        match step_type {
            "python" => {
                // Check for potentially dangerous Python code patterns
                if let Some(script) = config.get("script").and_then(|s| s.as_str()) {
                    let dangerous_patterns = ["import os", "subprocess", "eval(", "exec(", "__import__"];
                    for pattern in &dangerous_patterns {
                        if script.contains(pattern) {
                            return Err(anyhow!(
                                "Step {}: Python script contains potentially dangerous pattern: {}",
                                index, pattern
                            ));
                        }
                    }
                }
            },
            "webscrape" => {
                // Validate URL patterns
                if let Some(url) = config.get("url").and_then(|u| u.as_str()) {
                    if url.starts_with("file://") || url.contains("localhost") || url.contains("127.0.0.1") {
                        return Err(anyhow!(
                            "Step {}: Webscraping local/file URLs is not allowed",
                            index
                        ));
                    }
                }
            },
            _ => {}, // Other step types pass security validation
        }

        Ok(())
    }

    /// Validate performance characteristics
    fn validate_performance(workflow_spec: &Value) -> Result<()> {
        let steps = workflow_spec.get("steps")
            .and_then(|s| s.as_array())
            .ok_or_else(|| anyhow!("Invalid workflow structure"))?;

        // Check for excessive complexity
        if steps.len() > 20 {
            return Err(anyhow!(
                "Workflow has {} steps, which may impact performance",
                steps.len()
            ));
        }

        // Check for potential infinite loops or recursive patterns
        // This is a simplified check - a real implementation would be more sophisticated
        let step_types: Vec<String> = steps.iter()
            .filter_map(|step| step.get("step_type"))
            .filter_map(|t| t.as_str())
            .map(|s| s.to_string())
            .collect();

        // Count consecutive similar steps
        let mut consecutive_count = 1;
        let mut max_consecutive = 1;

        for i in 1..step_types.len() {
            if step_types[i] == step_types[i-1] {
                consecutive_count += 1;
                max_consecutive = max_consecutive.max(consecutive_count);
            } else {
                consecutive_count = 1;
            }
        }

        if max_consecutive > 5 {
            return Err(anyhow!(
                "Workflow has {} consecutive steps of the same type, which may indicate inefficient design",
                max_consecutive
            ));
        }

        Ok(())
    }

    /// Determine if workflow requires approval
    fn requires_approval(
        agent: &Agent,
        workflow_spec: &Value,
        errors: &[String],
        warnings: &[String],
    ) -> bool {
        // Always require approval if there are validation errors
        if !errors.is_empty() {
            return true;
        }

        // Require approval for complex workflows
        if let Some(steps) = workflow_spec.get("steps").and_then(|s| s.as_array()) {
            if steps.len() > 10 {
                return true;
            }
        }

        // Require approval if there are security warnings
        if !warnings.is_empty() && warnings.iter().any(|w| w.contains("security") || w.contains("dangerous")) {
            return true;
        }

        // Check agent policy for approval requirements
        if !agent.policy.security_constraints.is_null() {
            if let Some(require_approval) = agent.policy.security_constraints.get("require_approval") {
                if require_approval.as_bool().unwrap_or(false) {
                    return true;
                }
            }
        }

        false
    }
}

/// Result of workflow validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub requires_approval: bool,
}

impl ValidationResult {
    /// Check if the workflow can be executed
    pub fn can_execute(&self) -> bool {
        self.is_valid && !self.requires_approval
    }

    /// Get a summary of validation issues
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.errors.is_empty() {
            parts.push(format!("{} errors", self.errors.len()));
        }

        if !self.warnings.is_empty() {
            parts.push(format!("{} warnings", self.warnings.len()));
        }

        if self.requires_approval {
            parts.push("requires approval".to_string());
        }

        if parts.is_empty() {
            "valid".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdFields, TimestampFields};
    use serde_json::json;

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
            description: Some("Test agent for validation".to_string()),
            capabilities: super::types::AgentCapabilities {
                tools: vec!["python".to_string(), "webscrape".to_string()],
                models: vec!["gpt-4".to_string()],
                max_steps: Some(10),
                can_create_ephemeral: true,
                metadata: Value::Null,
            },
            policy: super::types::AgentPolicy {
                max_workflows_per_hour: Some(100),
                allowed_patterns: vec!["test".to_string()],
                security_constraints: json!({
                    "blocked_tools": ["dangerous_tool"]
                }),
                metadata: Value::Null,
            },
        }
    }

    #[test]
    fn test_valid_workflow() {
        let agent = create_test_agent();
        let workflow_spec = json!({
            "name": "Test Workflow",
            "workflow_type": "test",
            "steps": [
                {
                    "step_type": "python",
                    "config": {
                        "tool": "python",
                        "script": "print('hello')"
                    }
                }
            ]
        });

        let result = WorkflowValidator::validate_workflow_spec(&agent, &workflow_spec, false).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_workflow_missing_capability() {
        let agent = create_test_agent();
        let workflow_spec = json!({
            "name": "Test Workflow",
            "workflow_type": "test",
            "steps": [
                {
                    "step_type": "python",
                    "config": {
                        "tool": "dangerous_tool"
                    }
                }
            ]
        });

        let result = WorkflowValidator::validate_workflow_spec(&agent, &workflow_spec, true).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_security_validation() {
        let agent = create_test_agent();
        let workflow_spec = json!({
            "name": "Test Workflow",
            "workflow_type": "test",
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

        let result = WorkflowValidator::validate_workflow_spec(&agent, &workflow_spec, false).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("dangerous pattern")));
    }
}
