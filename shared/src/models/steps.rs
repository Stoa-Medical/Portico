use crate::{IdFields, TimestampFields, call_llm, exec_python};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde_json::Value;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{Postgres, Row};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StepType {
    Python,
    Prompt,
}

impl sqlx::Type<Postgres> for StepType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("step_type")
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for StepType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.as_str()? {
            "Python" => Ok(StepType::Python),
            "Prompt" => Ok(StepType::Prompt),
            _ => Err("Invalid step type".into()),
        }
    }
}

#[derive(Clone)]
pub struct Step {
    identifiers: IdFields,
    timestamps: TimestampFields,
    agent_owner_uuid: Uuid,
    name: String,
    description: Option<String>,
    step_type: StepType,
    step_content: String,
    success_count: Arc<AtomicU64>,
    run_count: Arc<AtomicU64>,
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

}
