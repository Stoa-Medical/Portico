use super::types::{Agent, AgentCapabilities, AgentPolicy};
use crate::{DatabaseItem, IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Agent {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let id: i32 = row.try_get("id")?;
        let global_uuid: uuid::Uuid = row.try_get("global_uuid")?;
        let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
        let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at")?;
        let name: String = row.try_get("name")?;
        let description: Option<String> = row.try_get("description")?;

        // Parse capabilities JSON
        let capabilities_json: Option<serde_json::Value> = row.try_get("capabilities_json")?;
        let capabilities = if let Some(caps_json) = capabilities_json {
            serde_json::from_value(caps_json).unwrap_or_default()
        } else {
            AgentCapabilities::default()
        };

        // Parse policy JSON
        let policy_json: Option<serde_json::Value> = row.try_get("policy_json")?;
        let policy = if let Some(pol_json) = policy_json {
            serde_json::from_value(pol_json).unwrap_or_default()
        } else {
            AgentPolicy::default()
        };

        Ok(Self {
            identifiers: IdFields {
                local_id: Some(id),
                global_uuid: global_uuid.to_string(),
            },
            timestamps: TimestampFields {
                created: created_at,
                updated: updated_at,
            },
            name,
            description,
            capabilities,
            policy,
        })
    }
}

impl JsonLike for Agent {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "name": self.name,
            "description": self.description,
            "capabilities": serde_json::to_value(&self.capabilities).unwrap_or_default(),
            "policy": serde_json::to_value(&self.policy).unwrap_or_default(),
        })
    }

    fn from_json(obj: Value) -> Result<Self> {
        if let Some(obj) = obj.as_object() {
            let capabilities = obj
                .get("capabilities")
                .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
                .unwrap_or_default();

            let policy = obj
                .get("policy")
                .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
                .unwrap_or_default();

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
                name: obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                description: obj
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                capabilities,
                policy,
            })
        } else {
            Err(anyhow!("Expected JSON object"))
        }
    }
}

#[async_trait]
impl DatabaseItem for Agent {
    type IdType = i32;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // Check if an agent with the same UUID already exists
        if crate::check_exists_by_uuid(pool, "agents", &self.identifiers.global_uuid).await? {
            return Ok(()); // Agent already exists, no need to create it again
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let capabilities_json = serde_json::to_value(&self.capabilities)?;
        let policy_json = serde_json::to_value(&self.policy)?;

        sqlx::query(
            r#"
            INSERT INTO agents (
                global_uuid, name, description, capabilities_json, policy_json, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(uuid_parsed)
        .bind(&self.name)
        .bind(self.description.as_deref())
        .bind(capabilities_json)
        .bind(policy_json)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let capabilities_json = serde_json::to_value(&self.capabilities)?;
        let policy_json = serde_json::to_value(&self.policy)?;

        sqlx::query(
            r#"
            UPDATE agents
            SET name = $1,
                description = $2,
                capabilities_json = $3,
                policy_json = $4,
                updated_at = $5
            WHERE global_uuid = $6
            "#,
        )
        .bind(&self.name)
        .bind(self.description.as_deref())
        .bind(capabilities_json)
        .bind(policy_json)
        .bind(&self.timestamps.updated)
        .bind(uuid_parsed)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        sqlx::query("DELETE FROM agents WHERE global_uuid = $1")
            .bind(uuid_parsed)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, global_uuid, name, description, capabilities_json, policy_json, created_at, updated_at
            FROM agents
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query_as::<_, Agent>(
                r#"
                SELECT id, global_uuid, name, description, capabilities_json, policy_json, created_at, updated_at
                FROM agents
                WHERE id = $1
                "#,
            )
            .bind(local_id)
            .fetch_optional(pool)
            .await?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query_as::<_, Agent>(
                r#"
                SELECT id, global_uuid, name, description, capabilities_json, policy_json, created_at, updated_at
                FROM agents
                WHERE global_uuid = $1
                "#,
            )
            .bind(uuid_parsed)
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt)
    }
}
