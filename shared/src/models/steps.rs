use crate::{IdFields, TimestampFields, call_llm, exec_python};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde_json::Value;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{Postgres, Row};
use chrono::NaiveDateTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StepType {
    Python,
    Prompt,
}

impl StepType {
    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match s {
            "python" => Ok(StepType::Python),
            "prompt" => Ok(StepType::Prompt),
            _ => Err("Invalid step type".into()),
        }
    }
}

impl sqlx::Type<Postgres> for StepType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("step_type")
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for StepType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::from_str(value.as_str()?)
    }
}

#[derive(Clone)]
pub struct Step {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub agent_owner_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub step_type: StepType,
    pub step_content: String,
    pub success_count: Arc<AtomicU64>,
    pub run_count: Arc<AtomicU64>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Step {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get("global_uuid")?,
            },
            timestamps: TimestampFields {
                created: row.try_get("created_timestamp")?,
                updated: row.try_get("last_updated_timestamp")?,
            },
            agent_owner_uuid: row.try_get("agent_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            step_type: row.try_get("step_type")?,
            step_content: row.try_get("step_content")?,
            success_count: Arc::new(AtomicU64::new(row.try_get::<i32, _>("success_count")? as u64)),
            run_count: Arc::new(AtomicU64::new(row.try_get::<i32, _>("run_count")? as u64)),
        })
    }
}

impl Step {
    pub fn new(
        identifiers: IdFields,
        agent_owner_uuid: Uuid,
        step_type: StepType,
        step_content: String,
        name: String,
        description: Option<String>,
        success_count: u64,
        run_count: u64,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            agent_owner_uuid,
            step_type,
            step_content,
            name,
            description,
            run_count: Arc::new(AtomicU64::new(run_count)),
            success_count: Arc::new(AtomicU64::new(success_count)),
        }
    }

    /// Runs the step with fresh context
    /// NOTE: The input from the last Step is stored in a python variable called `source`
    pub async fn run(&self, source_data: Value, step_idx: usize) -> Result<Value> {
        // Increment FIRST, before any potential errors
        self.run_count.fetch_add(1, Ordering::Relaxed);

        match &self.step_type {
            StepType::Prompt => match call_llm(&self.step_content, source_data).await {
                Ok(res_str) => {
                    self.success_count.fetch_add(1, Ordering::Relaxed);
                    Ok(Value::String(res_str))
                }
                Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
            },
            StepType::Python => match exec_python(source_data, &self.step_content) {
                Ok(result) => {
                    self.success_count.fetch_add(1, Ordering::Relaxed);
                    Ok(result)
                }
                Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
            },
        }
    }

    pub fn get_run_count(&self) -> u64 {
        self.run_count.load(Ordering::Relaxed)
    }

    pub fn get_success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }

    pub fn from_json(step_obj: &serde_json::Map<String, Value>) -> Option<Self> {
        Some(Self {
            identifiers: IdFields {
                local_id: step_obj.get("id").and_then(|v| v.as_i64()),
                global_uuid: step_obj.get("global_uuid").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            },
            timestamps: TimestampFields {
                created: NaiveDateTime::parse_from_str(
                    &step_obj.get("created_timestamp").and_then(|v| v.as_str()).unwrap_or_default(),
                    "%Y-%m-%d %H:%M:%S"
                ).unwrap_or_default(),
                updated: NaiveDateTime::parse_from_str(
                    &step_obj.get("last_updated_timestamp").and_then(|v| v.as_str()).unwrap_or_default(),
                    "%Y-%m-%d %H:%M:%S"
                ).unwrap_or_default(),
            },
            agent_owner_uuid: Uuid::parse_str(
                &step_obj.get("agent_id").and_then(|v| v.as_str()).unwrap_or_default()
            ).unwrap_or_default(),
            name: step_obj.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            description: step_obj.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            step_type: StepType::from_str(
                step_obj.get("step_type").and_then(|v| v.as_str()).unwrap_or_default()
            ).unwrap_or(StepType::Python),
            step_content: step_obj.get("step_content").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            success_count: Arc::new(AtomicU64::new(step_obj.get("success_count").and_then(|v| v.as_i64()).unwrap_or(0) as u64)),
            run_count: Arc::new(AtomicU64::new(step_obj.get("run_count").and_then(|v| v.as_i64()).unwrap_or(0) as u64)),
        })
    }

    pub fn from_json_array(steps_json: &Value) -> Vec<Self> {
        if let Some(steps_array) = steps_json.as_array() {
            steps_array.iter().filter_map(|step_json| {
                step_json.as_object().and_then(Step::from_json)
            }).collect()
        } else {
            Vec::new()
        }
    }
}
