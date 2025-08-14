use super::types::{Workflow, WorkflowState};
use crate::models::steps::Step;
use crate::{DatabaseItem, IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Workflow {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let id: i32 = row.try_get("id")?;
        let global_uuid: uuid::Uuid = row.try_get("global_uuid")?;
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
        let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at")?;
        let description: Option<String> = row.try_get("description")?;
        let workflow_state: WorkflowState = row.try_get("workflow_state")?;

        // Get step IDs if available
        let step_ids: Option<Vec<i32>> = row.try_get("step_ids").ok();

        Ok(Self {
            identifiers: IdFields {
                local_id: Some(id),
                global_uuid: global_uuid.to_string(),
            },
            timestamps: TimestampFields {
                created: created_at,
                updated: updated_at,
            },
            name: None, // Will be populated separately if needed
            workflow_type: None,
            description,
            workflow_state,
            workflow_name: None,
            step_ids,
            created_by_agent_id: None,
            version: "v1".to_string(),
            is_ephemeral: false,
        })
    }
}

impl JsonLike for Workflow {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "description": self.description,
            "workflow_state": self.state(),
            "step_ids": self.step_ids,
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
                description: obj
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                workflow_state: obj.get("workflow_state")
                    .and_then(|v| v.as_str())
                    .and_then(|s| WorkflowState::from_str(s).ok())
                    .unwrap_or_default(),
                name: obj.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                workflow_type: obj.get("workflow_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                workflow_name: obj.get("workflow_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                step_ids: obj.get("step_ids")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_i64().map(|i| i as i32))
                            .collect()
                    }),
                created_by_agent_id: obj.get("created_by_agent_id")
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i32),
                version: obj.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("v1")
                    .to_string(),
                is_ephemeral: obj.get("is_ephemeral")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            })
        } else {
            Err(anyhow!("Expected JSON object"))
        }
    }
}

#[async_trait]
impl DatabaseItem for Workflow {
    type IdType = i32;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // Check if a workflow with the same UUID already exists
        if crate::check_exists_by_uuid(pool, "workflows", &self.identifiers.global_uuid).await? {
            return Ok(()); // Workflow already exists, no need to create it again
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let workflow_state = self.state(); // Get the current state

        // Use query_scalar! for inserting the workflow and returning the ID
        let workflow_id = sqlx::query_scalar!(
            r#"
            INSERT INTO workflows (
                global_uuid, description, workflow_state, created_at, updated_at
            )
            VALUES ($1, $2, $3::workflow_state, $4, $5)
            RETURNING id
            "#,
            uuid_parsed,
            &self.description,
            workflow_state as WorkflowState,
            &self.timestamps.created,
            &self.timestamps.updated
        )
        .fetch_one(pool)
        .await?;

        // Steps are now created separately via step_ids field
        // Note: workflow_id is stored but not returned since self is immutable

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let workflow_state = self.state();

        sqlx::query!(
            r#"
            UPDATE workflows
            SET description = $1,
                workflow_state = $2::workflow_state,
                updated_at = $3
            WHERE global_uuid = $4
            "#,
            &self.description,
            workflow_state as WorkflowState,
            &self.timestamps.updated,
            uuid_parsed
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        if let Some(id) = self.identifiers.local_id {
            sqlx::query!("DELETE FROM steps WHERE workflow_id = $1", id)
                .execute(pool)
                .await?;
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        sqlx::query!("DELETE FROM workflows WHERE global_uuid = $1", uuid_parsed)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        struct WorkflowRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            workflow_state: WorkflowState,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = sqlx::query_as!(
            WorkflowRow,
            r#"
            SELECT
                w.id, w.global_uuid, w.description,
                w.workflow_state as "workflow_state: _",
                w.created_at, w.updated_at
            FROM workflows w
            "#
        )
        .fetch_all(pool)
        .await?;

        let workflows = rows
            .into_iter()
            .map(|row| {
                Workflow {
                    identifiers: IdFields {
                        local_id: Some(row.id),
                        global_uuid: row.global_uuid.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.created_at,
                        updated: row.updated_at,
                    },
                    name: None,
                    workflow_type: None,
                    description: row.description,
                    workflow_state: row.workflow_state,
                    workflow_name: None,
                    step_ids: None,
                    created_by_agent_id: None,
                    version: "v1".to_string(),
                    is_ephemeral: false,
                }
            })
            .collect();

        Ok(workflows)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        struct WorkflowRow {
            id: i32,
            global_uuid: uuid::Uuid,
            description: Option<String>,
            workflow_state: WorkflowState,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query_as!(
                WorkflowRow,
                r#"
                SELECT
                    w.id, w.global_uuid, w.description,
                    w.workflow_state as "workflow_state: _",
                    w.created_at, w.updated_at
                FROM workflows w
                WHERE w.id = $1
                "#,
                local_id
            )
            .fetch_optional(pool)
            .await?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query_as!(
                WorkflowRow,
                r#"
                SELECT
                    w.id, w.global_uuid, w.description,
                    w.workflow_state as "workflow_state: _",
                    w.created_at, w.updated_at
                FROM workflows w
                WHERE w.global_uuid = $1
                "#,
                uuid_parsed
            )
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt.map(|row| {
            Workflow {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                name: None,
                workflow_type: None,
                description: row.description,
                workflow_state: row.workflow_state,
                workflow_name: None,
                step_ids: None,
                created_by_agent_id: None,
                version: "v1".to_string(),
                is_ephemeral: false,
            }
        }))
    }
}
