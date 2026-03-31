use super::types::DataMapping;
use crate::{DatabaseItem, IdFields, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[async_trait]
impl DatabaseItem for DataMapping {
    type IdType = i32;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;

        sqlx::query(
            r#"
            INSERT INTO data_mappings (
                global_uuid, name, source_schema, target_schema,
                field_mappings, ai_generated, agent_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(uuid_parsed)
        .bind(&self.name)
        .bind(&self.source_schema)
        .bind(&self.target_schema)
        .bind(&self.field_mappings)
        .bind(self.ai_generated)
        .bind(self.agent_id)
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
            UPDATE data_mappings
            SET name = $1,
                source_schema = $2,
                target_schema = $3,
                field_mappings = $4,
                ai_generated = $5,
                agent_id = $6,
                updated_at = CURRENT_TIMESTAMP
            WHERE global_uuid = $7
            "#,
        )
        .bind(&self.name)
        .bind(&self.source_schema)
        .bind(&self.target_schema)
        .bind(&self.field_mappings)
        .bind(self.ai_generated)
        .bind(self.agent_id)
        .bind(uuid_parsed)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let res = sqlx::query("DELETE FROM data_mappings WHERE global_uuid = $1")
            .bind(uuid_parsed)
            .execute(pool)
            .await?;

        if res.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!("Failed to delete DataMapping"))
        }
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, global_uuid, name, source_schema, target_schema,
                field_mappings, ai_generated, agent_id, created_at, updated_at
            FROM data_mappings
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let items = rows
            .iter()
            .filter_map(|row| {
                Some(DataMapping {
                    identifiers: IdFields {
                        local_id: row.try_get("id").ok(),
                        global_uuid: row.try_get::<Uuid, _>("global_uuid").ok()?.to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.try_get("created_at").ok()?,
                        updated: row.try_get("updated_at").ok()?,
                    },
                    name: row.try_get("name").ok()?,
                    source_schema: row.try_get("source_schema").unwrap_or(None),
                    target_schema: row.try_get("target_schema").unwrap_or(None),
                    field_mappings: row.try_get("field_mappings").unwrap_or(None),
                    ai_generated: row.try_get("ai_generated").unwrap_or(false),
                    agent_id: row.try_get("agent_id").unwrap_or(None),
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
                    id, global_uuid, name, source_schema, target_schema,
                    field_mappings, ai_generated, agent_id, created_at, updated_at
                FROM data_mappings
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
                    id, global_uuid, name, source_schema, target_schema,
                    field_mappings, ai_generated, agent_id, created_at, updated_at
                FROM data_mappings
                WHERE global_uuid = $1
                "#,
            )
            .bind(uuid_parsed)
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt.and_then(|row| {
            Some(DataMapping {
                identifiers: IdFields {
                    local_id: row.try_get("id").ok(),
                    global_uuid: row.try_get::<Uuid, _>("global_uuid").ok()?.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.try_get("created_at").ok()?,
                    updated: row.try_get("updated_at").ok()?,
                },
                name: row.try_get("name").ok()?,
                source_schema: row.try_get("source_schema").unwrap_or(None),
                target_schema: row.try_get("target_schema").unwrap_or(None),
                field_mappings: row.try_get("field_mappings").unwrap_or(None),
                ai_generated: row.try_get("ai_generated").unwrap_or(false),
                agent_id: row.try_get("agent_id").unwrap_or(None),
            })
        }))
    }
}
