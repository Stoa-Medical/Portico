use crate::{DatabaseItem, IdFields, JsonLike, TimestampFields};
use serde_json::Value;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgArgumentBuffer, PgPool, Postgres, Row};
use uuid::Uuid;

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
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::from_str(value.as_str()?)
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

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Step {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get("global_uuid")?,
            },
            timestamps: TimestampFields {
                created: row.try_get("created_at")?,
                updated: row.try_get("updated_at")?,
            },
            // agent_owner_uuid: row.try_get("agent_id")?,
            description: row.try_get("description")?,
            step_type: row.try_get("step_type")?,
            step_content: row.try_get("step_content")?,
        })
    }
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

    /// Check if this step is a Python step
    pub fn is_python_step(&self) -> bool {
        matches!(self.step_type, StepType::Python)
    }

    /// Generates a Python function with the standardized signature for a step
    pub fn to_python_function(&self) -> String {
        let func_name = self.python_function_name();
        let docstring = format!(
            "\"\"\"\n    {}\n    \n    Args:\n        source: Input data dictionary from previous step\n        \n    Returns:\n        dict: Output data to pass to next step\n    \"\"\"",
            self.description.as_deref().unwrap_or("No description provided")
        );

        // Create a simplified Python function with proper indentation
        format!(
            r#"def {}(source):
    {}
    # Step implementation
    result = source  # Default pass-through

{}

    return result"#,
            func_name,
            docstring,
            // Indent all lines with 4 spaces for proper Python indentation
            self.step_content
                .lines()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Returns the standard Python function name for this step
    pub fn python_function_name(&self) -> String {
        format!("step_{}", self.identifiers.global_uuid.replace("-", "_"))
    }

    /// Deprecated: Use to_python_function() instead
    #[deprecated(since = "0.2.0", note = "Use to_python_function() instead")]
    pub fn generate_python_template(&self) -> String {
        self.to_python_function()
    }

    /// Runs the step with fresh context
    /// This method is deprecated and will be removed in the future
    /// Use PythonRuntime for executing Python steps
    #[deprecated(since = "0.2.0", note = "Use PythonRuntime for executing Python steps")]
    pub async fn run(&self, source_data: Value, step_idx: usize) -> Result<Value> {
        match &self.step_type {
            StepType::Prompt => match crate::call_llm(&self.step_content, source_data).await {
                Ok(res_str) => Ok(Value::String(res_str)),
                Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
            },
            StepType::Python => {
                #[allow(deprecated)]
                match crate::exec_python(source_data, &self.to_python_function()) {
                    Ok(result) => Ok(result),
                    Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
                }
            }
        }
    }

    pub fn from_json_array(steps_json: &Value) -> Vec<Self> {
        if let Some(steps_array) = steps_json.as_array() {
            steps_array
                .iter()
                .filter_map(|step_json| Step::from_json(step_json.clone()).ok())
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl JsonLike for Step {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "description": self.description,
            "step_type": self.step_type.as_str(),
            "step_content": self.step_content,
        })
    }

    fn from_json(obj: Value) -> Result<Self> {
        if let Some(obj) = obj.as_object() {
            Ok(Self {
                identifiers: IdFields {
                    local_id: obj.get("id").and_then(|v| v.as_i64()).map(|v| v as i32),
                    global_uuid: obj
                        .get("global_uuid")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                },
                timestamps: TimestampFields {
                    created: chrono::DateTime::parse_from_str(
                        &obj.get("created_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default(),
                        "%Y-%m-%d %H:%M:%S %z",
                    )
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                    updated: chrono::DateTime::parse_from_str(
                        &obj.get("updated_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default(),
                        "%Y-%m-%d %H:%M:%S %z",
                    )
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                },
                description: obj.get("description").and_then(|v| {
                    if v.is_null() {
                        None
                    } else {
                        Some(v.as_str().unwrap_or_default().to_string())
                    }
                }),
                step_type: StepType::from_str(
                    obj.get("step_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("python"),
                )
                .unwrap_or(StepType::Python),
                step_content: obj
                    .get("step_content")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            })
        } else {
            Err(anyhow!("Step::from_json - not a JSON object"))
        }
    }

    fn update_from_json(&mut self, obj: Value) -> Result<Vec<String>> {
        let mut updated_fields = Vec::new();

        if let Some(obj) = obj.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    "description" => {
                        if value.is_null() {
                            if self.description.is_some() {
                                self.description = None;
                                updated_fields.push(key.to_string());
                            }
                        } else if let Some(s) = value.as_str() {
                            let current = self.description.as_deref().unwrap_or("");
                            if current != s {
                                self.description = Some(s.to_string());
                                updated_fields.push(key.to_string());
                            }
                        }
                    }
                    "step_type" => {
                        if let Some(s) = value.as_str() {
                            if let Ok(new_type) = StepType::from_str(s) {
                                if self.step_type.as_str() != new_type.as_str() {
                                    self.step_type = new_type;
                                    updated_fields.push(key.to_string());
                                }
                            }
                        }
                    }
                    "step_content" => {
                        if let Some(s) = value.as_str() {
                            if self.step_content != s {
                                self.step_content = s.to_string();
                                updated_fields.push(key.to_string());
                            }
                        }
                    }
                    // Skip fields that shouldn't be updated directly
                    "id" | "global_uuid" | "created_at" | "updated_at" => {
                        // These are either ID fields or timestamp fields
                        // Skip updating them via this method
                    }
                    // If we don't recognize the field, just skip
                    _ => {}
                }
            }
        }

        Ok(updated_fields)
    }
}

#[async_trait]
impl DatabaseItem for Step {
    fn id(&self) -> &IdFields {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        let agent_id = if let Some(id) = self.identifiers.local_id {
            id
        } else {
            return Err(anyhow!("Cannot create a Step without a local_id"));
        };

        let mut tx = pool.begin().await?;

        // Get the next sequence_number for this step
        let next_seq: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(MAX(sequence_number), -1) + 1
            FROM steps
            WHERE agent_id = $1
            "#,
        )
        .bind(agent_id)
        .fetch_one(&mut *tx)
        .await?;

        // Create the step
        let res = sqlx::query(
            r#"
            INSERT INTO steps (
                global_uuid, agent_id, sequence_number, name, description,
                step_type, step_content, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
        .bind(agent_id)
        .bind(next_seq)
        .bind(&self.description)
        .bind(&self.step_type)
        .bind(&self.step_content)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        if res.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!("Failed to create Step"))
        }
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let res = sqlx::query(
            r#"
            UPDATE steps
            SET description = $1,
                step_type = $2,
                step_content = $3,
                updated_at = $4
            WHERE global_uuid = $5
            "#,
        )
        .bind(&self.description)
        .bind(&self.step_type)
        .bind(&self.step_content)
        .bind(&self.timestamps.updated)
        .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
        .execute(pool)
        .await?;

        if res.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!("Failed to update Step"))
        }
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        // Simply delete this Step by UUID
        let res = sqlx::query("DELETE FROM steps WHERE global_uuid = $1")
            .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
            .execute(pool)
            .await?;

        if res.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete Step"))
        }
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let steps = sqlx::query_as::<_, Self>("SELECT * FROM steps ORDER BY id")
            .fetch_all(pool)
            .await?;

        Ok(steps)
    }

    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields) -> Result<Option<Self>> {
        // Try to find by global UUID
        if !id.global_uuid.is_empty() {
            let step = sqlx::query_as::<_, Self>("SELECT * FROM steps WHERE global_uuid = $1")
                .bind(Uuid::parse_str(&id.global_uuid)?)
                .fetch_optional(pool)
                .await?;

            if step.is_some() {
                return Ok(step);
            }
        }

        // Fall back to local ID if available and uuid not found
        if let Some(local_id) = id.local_id {
            let step = sqlx::query_as::<_, Self>("SELECT * FROM steps WHERE id = $1")
                .bind(local_id)
                .fetch_optional(pool)
                .await?;

            return Ok(step);
        }

        // If we get here, neither UUID nor local ID matched
        Ok(None)
    }
}
