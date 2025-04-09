use crate::{IdFields, TimestampFields, call_llm, exec_python, DatabaseItem};
use serde_json::Value;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, PgPool, postgres::PgArgumentBuffer};
use chrono::NaiveDateTime;
use async_trait::async_trait;

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

    pub fn as_str(&self) -> &'static str {
        match self {
            StepType::Python => "python",
            StepType::Prompt => "prompt",
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

impl<'q> sqlx::Encode<'q, Postgres> for StepType {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.as_str();
        buf.extend_from_slice(s.as_bytes());
        Ok(sqlx::encode::IsNull::No)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
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
            // agent_owner_uuid: row.try_get("agent_id")?,
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
            step_type,
            step_content,
            name,
            description,
            run_count: Arc::new(AtomicU64::new(run_count)),
            success_count: Arc::new(AtomicU64::new(success_count)),
        }
    }

    /// Generates a Python function template for this step
    pub fn generate_python_template(&self) -> String {
        let func_name = format!("step_{}", self.identifiers.global_uuid.replace("-", "_"));
        // Example of generated Python function:
        // ```python
        // def step_123e4567_e89b_12d3_a456_426614174000(source: dict) -> dict:
        //     """
        //     Process the input data
        //
        //     Args:
        //         source: Input data dictionary from previous step
        //
        //     Returns:
        //         dict: Output data to pass to next step
        //     """
        //     # Step implementation
        //     result = source  # Default pass-through
        //
        //     # Custom code here
        //
        //     return result
        //
        // result = step_123e4567_e89b_12d3_a456_426614174000(source)
        // ```
        let docstring = format!(
            "\"\"\"\n    {}\n    \n    Args:\n        source: Input data dictionary from previous step\n        \n    Returns:\n        dict: Output data to pass to next step\n    \"\"\"",
            self.description.as_deref().unwrap_or("No description provided")
        );

        format!(
            r#"def {}(source: dict) -> dict:
    {}
    # Step implementation
    result = source  # Default pass-through

    {}

    return result

# Execute the step function
result = {}(source)"#,
            func_name,
            docstring,
            self.step_content.replace("\n", "\n    "),
            func_name
        )
    }

    /// Runs the step with fresh context
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
            StepType::Python => match exec_python(source_data, &self.generate_python_template()) {
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

#[async_trait]
impl DatabaseItem for Step {
    fn id(&self) -> &IdFields {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO steps (
                global_uuid, name, description, step_type, step_content,
                success_count, run_count, created_timestamp, last_updated_timestamp
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(&self.identifiers.global_uuid)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.step_type)
        .bind(&self.step_content)
        .bind(self.success_count.load(Ordering::Relaxed) as i32)
        .bind(self.run_count.load(Ordering::Relaxed) as i32)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE steps
            SET name = $1,
                description = $2,
                step_type = $3,
                step_content = $4,
                success_count = $5,
                run_count = $6,
                last_updated_timestamp = $7
            WHERE global_uuid = $8
            "#
        )
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.step_type)
        .bind(&self.step_content)
        .bind(self.success_count.load(Ordering::Relaxed) as i32)
        .bind(self.run_count.load(Ordering::Relaxed) as i32)
        .bind(&self.timestamps.updated)
        .bind(&self.identifiers.global_uuid)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM steps WHERE global_uuid = $1")
            .bind(&self.identifiers.global_uuid)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query_as::<_, Step>(
            r#"
            SELECT * FROM steps
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields) -> Result<Option<Self>> {
        let row = if let Some(local_id) = id.local_id {
            sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE id = $1")
                .bind(local_id)
                .fetch_optional(pool)
                .await?
        } else {
            sqlx::query_as::<_, Step>("SELECT * FROM steps WHERE global_uuid = $1")
                .bind(&id.global_uuid)
                .fetch_optional(pool)
                .await?
        };

        Ok(row)
    }
}
