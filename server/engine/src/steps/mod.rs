pub mod python;
pub mod llm;
pub mod transform;
pub mod validate;
pub mod fhir;

use anyhow::Result;
use portico_database::StepType;
use serde_json::Value;

/// Dispatch a step to the appropriate executor based on its type
pub async fn execute_step(
    step_type: &StepType,
    config: &Value,
    input: Value,
) -> Result<Value> {
    match step_type {
        StepType::Python => python::execute(config, input).await,
        StepType::LLM => llm::execute(config, input).await,
        StepType::Transform => transform::execute(config, input).await,
        StepType::Validate => validate::execute(config, input).await,
        StepType::FHIR => fhir::execute(config, input).await,
    }
}
