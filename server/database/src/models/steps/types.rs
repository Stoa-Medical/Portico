use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgArgumentBuffer, Postgres};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StepType {
    Python,
    LLM,
    Transform,
    Validate,
    FHIR,
}

impl StepType {
    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match s {
            "python" => Ok(StepType::Python),
            "llm" => Ok(StepType::LLM),
            "transform" => Ok(StepType::Transform),
            "validate" => Ok(StepType::Validate),
            "fhir" => Ok(StepType::FHIR),
            _ => Err(format!("Invalid step type: {}", s).into()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StepType::Python => "python",
            StepType::LLM => "llm",
            StepType::Transform => "transform",
            StepType::Validate => "validate",
            StepType::FHIR => "fhir",
        }
    }
}

impl sqlx::Type<Postgres> for StepType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("step_type")
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for StepType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        StepType::from_str(value.as_str()?)
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for StepType {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.as_str();
        buf.extend_from_slice(s.as_bytes());
        Ok(sqlx::encode::IsNull::No)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub name: Option<String>,
    pub description: Option<String>,
    pub step_type: StepType,
    pub config: Option<Value>,
    pub step_order: Option<i32>,
}

impl Step {
    pub fn new(
        identifiers: IdFields,
        step_type: StepType,
        config: Option<Value>,
        description: Option<String>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            name: None,
            step_type,
            config,
            description,
            step_order: None,
        }
    }
}
