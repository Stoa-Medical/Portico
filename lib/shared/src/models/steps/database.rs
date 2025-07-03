use super::types::{Step, StepType};
use crate::{DatabaseItem, IdFields, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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

        let steps = rows
            .into_iter()
            .map(|row| {
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

        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query_as::<_, StepRow>(
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
            .await?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query_as::<_, StepRow>(
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
            .await?
        };

        Ok(row_opt.map(|row| {
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
        }))
    }
}
