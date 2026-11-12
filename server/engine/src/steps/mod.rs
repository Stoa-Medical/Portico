pub mod fhir;
pub mod llm;
pub mod python;
pub mod transform;
pub mod validate;

use anyhow::{anyhow, Result};
use serde_json::Value;

/// Mirrors the `step_type` enum in `server/database/schema.sql`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepType {
    Python,
    Llm,
    Transform,
    Validate,
    Fhir,
}

impl StepType {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "python" => Ok(Self::Python),
            "llm" => Ok(Self::Llm),
            "transform" => Ok(Self::Transform),
            "validate" => Ok(Self::Validate),
            "fhir" => Ok(Self::Fhir),
            other => Err(anyhow!("unknown step_type '{other}'")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Llm => "llm",
            Self::Transform => "transform",
            Self::Validate => "validate",
            Self::Fhir => "fhir",
        }
    }
}

pub async fn execute_step(step_type: StepType, config: &Value, input: Value) -> Result<Value> {
    match step_type {
        StepType::Python => python::execute(config, input).await,
        StepType::Llm => llm::execute(config, input).await,
        StepType::Transform => transform::execute(config, input).await,
        StepType::Validate => validate::execute(config, input).await,
        StepType::Fhir => fhir::execute(config, input).await,
    }
}
