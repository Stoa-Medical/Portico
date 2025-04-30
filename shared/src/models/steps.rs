use crate::{DatabaseItem, IdFields, JsonLike, PythonRuntime, TimestampFields};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgArgumentBuffer, PgPool, Postgres, Row};
use uuid::Uuid;

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
        // Only decode the type string, not the LLM model (which will be handled separately)
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

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Step {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let step_type_str: &str = row.try_get("step_type")?;

        // Try to get llm_model, but don't fail if the column doesn't exist
        let llm_model: Option<String> = match row.try_get("llm_model") {
            Ok(model) => model,
            Err(_) => None, // Column might not exist yet
        };

        let step_type = match step_type_str {
            "python" => StepType::Python,
            "prompt" => StepType::Prompt(
                llm_model.unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
            ),
            "webscrape" => StepType::WebScrape,
            _ => return Err(sqlx::Error::ColumnNotFound("Invalid step type".into())),
        };

        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get::<uuid::Uuid, _>("global_uuid")?.to_string(),
            },
            timestamps: TimestampFields {
                created: row.try_get("created_at")?,
                updated: row.try_get("updated_at")?,
            },
            description: row.try_get("description")?,
            step_type,
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

    // Helper method to create a new Prompt step with an LLM model
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

    /// Helper method to create a new WebScrape step
    pub fn new_webscrape(identifiers: IdFields, url: String, description: Option<String>) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            step_type: StepType::WebScrape,
            step_content: url,
            description,
        }
    }

    /// Check if this step is a Python step
    pub fn is_python_step(&self) -> bool {
        matches!(self.step_type, StepType::Python)
    }

    /// Check if this step is a Prompt step
    pub fn is_prompt_step(&self) -> bool {
        matches!(self.step_type, StepType::Prompt(_))
    }

    /// Check if this step is a WebScrape step
    pub fn is_webscrape_step(&self) -> bool {
        matches!(self.step_type, StepType::WebScrape)
    }

    /// Get the LLM model if this is a Prompt step
    pub fn get_llm_model(&self) -> Option<String> {
        self.step_type.get_llm_model()
    }

    /// Generates a Python function with the standardized signature for execution in a PythonRuntime
    ///
    /// This method generates the Python function code that will be loaded into the PythonRuntime
    /// for execution. The function follows a standardized signature expected by the runtime.
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

    /// Runs the step with fresh context
    /// For Python steps, a runtime must be provided.
    /// For Prompt steps, the runtime is optional.
    pub async fn run(
        &self,
        source_data: Value,
        step_idx: usize,
        runtime: Option<&PythonRuntime>,
    ) -> Result<Value> {
        match &self.step_type {
            StepType::Prompt(llm_model) => {
                match crate::call_llm(&self.step_content, source_data, Some(llm_model.clone()))
                    .await
                {
                    Ok(res_str) => Ok(Value::String(res_str)),
                    Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
                }
            }
            StepType::Python => {
                // For Python steps, require a runtime
                if let Some(rt) = runtime {
                    rt.execute_step(&self.identifiers.global_uuid, source_data)
                } else {
                    Err(anyhow!(
                        "Python step {} requires a runtime to execute",
                        step_idx
                    ))
                }
            }
            StepType::WebScrape => {
                // For WebScrape steps, the step_content should contain the URL to scrape
                let url = self.step_content.trim();
                if url.is_empty() {
                    return Err(anyhow!("WebScrape step {} has empty URL", step_idx));
                }

                // Call the web scraping function
                match crate::scrape_webpage(url).await {
                    Ok(result) => Ok(result),
                    Err(err) => Err(anyhow!("WebScrape step {} failed: {}", step_idx, err)),
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
        let mut json = json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "description": self.description,
            "step_type": self.step_type.as_str(),
            "step_content": self.step_content,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        // Add llm_model field only for Prompt steps
        if let StepType::Prompt(model) = &self.step_type {
            json["llm_model"] = json!(model);
        }

        json
    }

    fn from_json(obj: Value) -> Result<Self> {
        let step_type_str = obj["step_type"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing step_type"))?;

        let step_content = obj["step_content"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing step_content"))?;

        // Handle optional fields
        let description = obj["description"].as_str().map(|s| s.to_string());
        let llm_model = obj["llm_model"].as_str().map(|s| s.to_string());

        // Create the appropriate StepType based on the type string and llm_model
        let step_type = match step_type_str {
            "python" => StepType::Python,
            "prompt" => StepType::Prompt(
                llm_model.unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
            ),
            _ => return Err(anyhow!("Invalid step type: {}", step_type_str)),
        };

        // Handle ID fields
        let local_id = obj["id"].as_i64().map(|id| id as i32);
        let global_uuid = if let Some(uuid_str) = obj["global_uuid"].as_str() {
            uuid_str.to_string()
        } else {
            Uuid::new_v4().to_string()
        };

        // Handle timestamp fields
        let created = if let Some(ts) = obj["created_at"].as_str() {
            chrono::DateTime::parse_from_rfc3339(ts)
                .map_err(|e| anyhow!("Invalid created_at timestamp: {}", e))?
                .with_timezone(&chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        let updated = if let Some(ts) = obj["updated_at"].as_str() {
            chrono::DateTime::parse_from_rfc3339(ts)
                .map_err(|e| anyhow!("Invalid updated_at timestamp: {}", e))?
                .with_timezone(&chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        Ok(Self {
            identifiers: IdFields {
                local_id,
                global_uuid,
            },
            timestamps: TimestampFields { created, updated },
            description,
            step_type,
            step_content: step_content.to_string(),
        })
    }
}

#[async_trait]
impl DatabaseItem for Step {
    type IdType = i32;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;

        // Extract llm_model from step_type if it's a Prompt step
        let llm_model = match &self.step_type {
            StepType::Prompt(model) => Some(model.clone()),
            _ => None,
        };

        // Insert with the llm_model column
        sqlx::query(
            r#"
            INSERT INTO steps
                (global_uuid, description, step_type, step_content, llm_model)
            VALUES
                ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(uuid_parsed)
        .bind(&self.description)
        .bind(self.step_type.as_str())
        .bind(&self.step_content)
        .bind(llm_model)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;

        // Extract llm_model from step_type if it's a Prompt step
        let llm_model = match &self.step_type {
            StepType::Prompt(model) => Some(model.clone()),
            _ => None,
        };

        // Try to update by global UUID first
        let result = sqlx::query(
            r#"
            UPDATE steps
            SET
                description = $1,
                step_type = $2,
                step_content = $3,
                llm_model = $4,
                updated_at = CURRENT_TIMESTAMP
            WHERE global_uuid = $5
            "#,
        )
        .bind(&self.description)
        .bind(self.step_type.as_str())
        .bind(&self.step_content)
        .bind(&llm_model)
        .bind(uuid_parsed)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            // If no rows were updated by UUID, try by local ID if available
            if let Some(local_id) = self.identifiers.local_id {
                sqlx::query(
                    r#"
                    UPDATE steps
                    SET
                        description = $1,
                        step_type = $2,
                        step_content = $3,
                        llm_model = $4,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = $5
                    "#,
                )
                .bind(&self.description)
                .bind(self.step_type.as_str())
                .bind(&self.step_content)
                .bind(&llm_model)
                .bind(local_id)
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let res = sqlx::query("DELETE FROM steps WHERE global_uuid = $1")
            .bind(uuid_parsed)
            .execute(pool)
            .await?;

        if res.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete Step"))
        }
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        // Define struct compatible with query_as output
        #[derive(sqlx::FromRow)]
        struct StepRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            step_type: String,
            step_content: String,
            llm_model: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = sqlx::query_as::<_, StepRow>(
            r#"
            SELECT
                id, global_uuid, description,
                step_type, step_content, llm_model,
                created_at, updated_at
            FROM steps
            ORDER BY id
            "#,
        )
        .fetch_all(pool)
        .await?;

        // Convert rows to Step objects
        let steps = rows
            .into_iter()
            .map(|row| {
                // Create the appropriate StepType based on the type string and llm_model
                let step_type = match row.step_type.as_str() {
                    "python" => StepType::Python,
                    "prompt" => StepType::Prompt(
                        row.llm_model
                            .unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
                    ),
                    "webscrape" => StepType::WebScrape,
                    _ => StepType::Python, // Default fallback
                };

                Step {
                    identifiers: IdFields {
                        local_id: Some(row.id),
                        global_uuid: row.global_uuid.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.created_at,
                        updated: row.updated_at,
                    },
                    description: row.description,
                    step_type,
                    step_content: row.step_content,
                }
            })
            .collect();

        Ok(steps)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        // Define struct compatible with query_as output
        #[derive(sqlx::FromRow)]
        struct StepRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            step_type: String,
            step_content: String,
            llm_model: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        // Try to find by global UUID first
        if !id.global_uuid.is_empty() {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            let row = sqlx::query_as::<_, StepRow>(
                r#"
                SELECT
                    id, global_uuid, description,
                    step_type, step_content, llm_model,
                    created_at, updated_at
                FROM steps
                WHERE global_uuid = $1
                "#,
            )
            .bind(uuid_parsed)
            .fetch_optional(pool)
            .await?;

            if let Some(row) = row {
                // Create the appropriate StepType based on the type string and llm_model
                let step_type = match row.step_type.as_str() {
                    "python" => StepType::Python,
                    "prompt" => StepType::Prompt(
                        row.llm_model
                            .unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
                    ),
                    "webscrape" => StepType::WebScrape,
                    _ => StepType::Python, // Default fallback
                };

                return Ok(Some(Step {
                    identifiers: IdFields {
                        local_id: Some(row.id),
                        global_uuid: row.global_uuid.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.created_at,
                        updated: row.updated_at,
                    },
                    description: row.description,
                    step_type,
                    step_content: row.step_content,
                }));
            }
        }

        // Fall back to local ID if available and UUID not found
        if let Some(local_id) = id.local_id {
            let row = sqlx::query_as::<_, StepRow>(
                r#"
                SELECT
                    id, global_uuid, description,
                    step_type, step_content, llm_model,
                    created_at, updated_at
                FROM steps
                WHERE id = $1
                "#,
            )
            .bind(local_id)
            .fetch_optional(pool)
            .await?;

            return Ok(row.map(|row| {
                // Create the appropriate StepType based on the type string and llm_model
                let step_type = match row.step_type.as_str() {
                    "python" => StepType::Python,
                    "prompt" => StepType::Prompt(
                        row.llm_model
                            .unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
                    ),
                    "webscrape" => StepType::WebScrape,
                    _ => StepType::Python, // Default fallback
                };

                Step {
                    identifiers: IdFields {
                        local_id: Some(row.id),
                        global_uuid: row.global_uuid.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.created_at,
                        updated: row.updated_at,
                    },
                    description: row.description,
                    step_type,
                    step_content: row.step_content,
                }
            }));
        }

        // If we get here, neither UUID nor local ID matched
        Ok(None)
    }
}
