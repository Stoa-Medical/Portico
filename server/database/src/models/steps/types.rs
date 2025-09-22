use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgArgumentBuffer, Postgres};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StepType {
    Python,
    Prompt(String),
    WebScrape,
}

impl StepType {
    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match s {
            "python" => Ok(StepType::Python),
            "prompt" => Ok(StepType::Prompt(
                crate::JsonModeLLMs::MetaLlama33_70b.to_string(),
            )),
            "webscrape" => Ok(StepType::WebScrape),
            _ => Err("Invalid step type".into()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StepType::Python => "python",
            StepType::Prompt(_) => "prompt",
            StepType::WebScrape => "webscrape",
        }
    }

    pub fn get_llm_model(&self) -> Option<String> {
        match self {
            StepType::Prompt(model) => Some(model.clone()),
            _ => None,
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
        match value.as_str()? {
            "python" => Ok(StepType::Python),
            "prompt" => Ok(StepType::Prompt(
                crate::JsonModeLLMs::MetaLlama33_70b.to_string(),
            )),
            "webscrape" => Ok(StepType::WebScrape),
            s => Err(format!("Invalid step type: {}", s).into()),
        }
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
    pub description: Option<String>,
    pub step_type: StepType,
    pub step_content: String,
}

impl Step {
    pub fn new(
        identifiers: IdFields,
        step_type: StepType,
        step_content: String,
        description: Option<String>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            step_type,
            step_content,
            description,
        }
    }

    pub fn new_prompt(
        identifiers: IdFields,
        step_content: String,
        description: Option<String>,
        llm_model: Option<String>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            step_type: StepType::Prompt(
                llm_model.unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
            ),
            step_content,
            description,
        }
    }

    pub fn new_webscrape(identifiers: IdFields, url: String, description: Option<String>) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            step_type: StepType::WebScrape,
            step_content: url,
            description,
        }
    }

    pub fn is_python_step(&self) -> bool {
        matches!(self.step_type, StepType::Python)
    }

    pub fn is_prompt_step(&self) -> bool {
        matches!(self.step_type, StepType::Prompt(_))
    }

    pub fn is_webscrape_step(&self) -> bool {
        matches!(self.step_type, StepType::WebScrape)
    }

    pub fn get_llm_model(&self) -> Option<String> {
        self.step_type.get_llm_model()
    }
}
