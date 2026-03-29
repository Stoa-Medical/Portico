use super::types::{Step, StepType};
use crate::{DatabaseItem, IdFields, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Step {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        let step_type_str: &str = row.try_get("step_type")?;

        let step_type = StepType::from_str(step_type_str)
            .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid step type: {}", e)))?;

        let config: Option<serde_json::Value> = row.try_get("config").unwrap_or(None);

        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get::<uuid::Uuid, _>("global_uuid")?.to_string(),
            },
            timestamps: TimestampFields {
                created: row.try_get("created_at")?,
                updated: row.try_get("updated_at")?,
            },
            name: row.try_get("name").unwrap_or(None),
            description: row.try_get("description")?,
            step_type,
            config,
            step_order: row.try_get("step_order").unwrap_or(None),
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

        sqlx::query(
            r#"
            INSERT INTO steps
                (global_uuid, name, description, step_type, config, step_order)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(uuid_parsed)
        .bind(&self.name)
        .bind(&self.description)
        .bind(self.step_type.as_str())
        .bind(&self.config)
        .bind(self.step_order)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;

        let result = sqlx::query(
            r#"
            UPDATE steps
            SET
                name = $1,
                description = $2,
                step_type = $3,
                config = $4,
                step_order = $5,
                updated_at = CURRENT_TIMESTAMP
            WHERE global_uuid = $6
            "#,
        )
        .bind(&self.name)
        .bind(&self.description)
        .bind(self.step_type.as_str())
        .bind(&self.config)
        .bind(self.step_order)
        .bind(uuid_parsed)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            if let Some(local_id) = self.identifiers.local_id {
                sqlx::query(
                    r#"
                    UPDATE steps
                    SET
                        name = $1,
                        description = $2,
                        step_type = $3,
                        config = $4,
                        step_order = $5,
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = $6
                    "#,
                )
                .bind(&self.name)
                .bind(&self.description)
                .bind(self.step_type.as_str())
                .bind(&self.config)
                .bind(self.step_order)
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
        let rows = sqlx::query(
            r#"
            SELECT
                id, global_uuid, name, description,
                step_type, config, step_order,
                created_at, updated_at
            FROM steps
            ORDER BY step_order ASC NULLS LAST, id ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        let steps = rows
            .iter()
            .filter_map(|row| {
                let step_type_str: &str = row.try_get("step_type").ok()?;
                let step_type = StepType::from_str(step_type_str).ok()?;

                Some(Step {
                    identifiers: IdFields {
                        local_id: row.try_get("id").ok(),
                        global_uuid: row
                            .try_get::<Uuid, _>("global_uuid")
                            .ok()?
                            .to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.try_get("created_at").ok()?,
                        updated: row.try_get("updated_at").ok()?,
                    },
                    name: row.try_get("name").unwrap_or(None),
                    description: row.try_get("description").ok()?,
                    step_type,
                    config: row.try_get("config").unwrap_or(None),
                    step_order: row.try_get("step_order").unwrap_or(None),
                })
            })
            .collect();

        Ok(steps)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query(
                r#"
                SELECT
                    id, global_uuid, name, description,
                    step_type, config, step_order,
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
            sqlx::query(
                r#"
                SELECT
                    id, global_uuid, name, description,
                    step_type, config, step_order,
                    created_at, updated_at
                FROM steps
                WHERE global_uuid = $1
                "#,
            )
            .bind(uuid_parsed)
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt.and_then(|row| {
            let step_type_str: &str = row.try_get("step_type").ok()?;
            let step_type = StepType::from_str(step_type_str).ok()?;

            Some(Step {
                identifiers: IdFields {
                    local_id: row.try_get("id").ok(),
                    global_uuid: row
                        .try_get::<Uuid, _>("global_uuid")
                        .ok()?
                        .to_string(),
                },
                timestamps: TimestampFields {
                    created: row.try_get("created_at").ok()?,
                    updated: row.try_get("updated_at").ok()?,
                },
                name: row.try_get("name").unwrap_or(None),
                description: row.try_get("description").ok()?,
                step_type,
                config: row.try_get("config").unwrap_or(None),
                step_order: row.try_get("step_order").unwrap_or(None),
            })
        }))
    }
}
