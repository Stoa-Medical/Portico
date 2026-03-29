use super::types::Integration;
use crate::{DatabaseItem, IdFields, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[async_trait]
impl DatabaseItem for Integration {
    type IdType = i32;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;

        sqlx::query(
            r#"
            INSERT INTO integrations (
                global_uuid, name, connection_type, direction,
                config, agent_id, status, last_seen_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(uuid_parsed)
        .bind(&self.name)
        .bind(&self.connection_type)
        .bind(&self.direction)
        .bind(&self.config)
        .bind(self.agent_id)
        .bind(&self.status)
        .bind(&self.last_seen_at)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;

        sqlx::query(
            r#"
            UPDATE integrations
            SET name = $1,
                connection_type = $2,
                direction = $3,
                config = $4,
                agent_id = $5,
                status = $6,
                last_seen_at = $7,
                updated_at = CURRENT_TIMESTAMP
            WHERE global_uuid = $8
            "#,
        )
        .bind(&self.name)
        .bind(&self.connection_type)
        .bind(&self.direction)
        .bind(&self.config)
        .bind(self.agent_id)
        .bind(&self.status)
        .bind(&self.last_seen_at)
        .bind(uuid_parsed)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let res = sqlx::query("DELETE FROM integrations WHERE global_uuid = $1")
            .bind(uuid_parsed)
            .execute(pool)
            .await?;

        if res.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete Integration"))
        }
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, global_uuid, name, connection_type, direction,
                config, agent_id, status, last_seen_at,
                created_at, updated_at
            FROM integrations
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let items = rows
            .iter()
            .filter_map(|row| {
                Some(Integration {
                    identifiers: IdFields {
                        local_id: row.try_get("id").ok(),
                        global_uuid: row.try_get::<Uuid, _>("global_uuid").ok()?.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.try_get("created_at").ok()?,
                        updated: row.try_get("updated_at").ok()?,
                    },
                    name: row.try_get("name").ok()?,
                    connection_type: row.try_get("connection_type").ok()?,
                    direction: row.try_get("direction").ok()?,
                    config: row.try_get("config").unwrap_or(None),
                    agent_id: row.try_get("agent_id").unwrap_or(None),
                    status: row.try_get("status").ok()?,
                    last_seen_at: row.try_get("last_seen_at").unwrap_or(None),
                })
            })
            .collect();

        Ok(items)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query(
                r#"
                SELECT
                    id, global_uuid, name, connection_type, direction,
                    config, agent_id, status, last_seen_at,
                    created_at, updated_at
                FROM integrations
                WHERE id = $1
                "#,
            )
            .bind(local_id)
            .fetch_optional(pool)
            .await?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query(
                r#"
                SELECT
                    id, global_uuid, name, connection_type, direction,
                    config, agent_id, status, last_seen_at,
                    created_at, updated_at
                FROM integrations
                WHERE global_uuid = $1
                "#,
            )
            .bind(uuid_parsed)
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt.and_then(|row| {
            Some(Integration {
                identifiers: IdFields {
                    local_id: row.try_get("id").ok(),
                    global_uuid: row.try_get::<Uuid, _>("global_uuid").ok()?.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.try_get("created_at").ok()?,
                    updated: row.try_get("updated_at").ok()?,
                },
                name: row.try_get("name").ok()?,
                connection_type: row.try_get("connection_type").ok()?,
                direction: row.try_get("direction").ok()?,
                config: row.try_get("config").unwrap_or(None),
                agent_id: row.try_get("agent_id").unwrap_or(None),
                status: row.try_get("status").ok()?,
                last_seen_at: row.try_get("last_seen_at").unwrap_or(None),
            })
        }))
    }
}
