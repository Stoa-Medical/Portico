use super::types::Signal;
use crate::{DatabaseItem, IdFields, TimestampFields};
use crate::models::signals::SignalType;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Signal {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        // Get the signal type
        let signal_type = if let Ok(signal_type_str) = row.try_get::<&str, _>("signal_type") {
            signal_type_str
                .parse()
                .unwrap_or(crate::models::signals::SignalType::Fyi)
        } else {
            crate::models::signals::SignalType::Fyi
        };

        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get::<Uuid, _>("global_uuid")?.to_string(),
            },
            timestamps: TimestampFields {
                created: row.try_get("created_at")?,
                updated: row.try_get("updated_at")?,
            },
            user_requested_uuid: row.try_get::<Uuid, _>("user_requested_uuid")?.to_string(),
            workflow_id: row.try_get("workflow_id")?,
            initiator_agent_id: row.try_get("initiator_agent_id")?,
            rts_id: row.try_get("rts_id")?,
            signal_type,
            initial_data: row.try_get("initial_data")?,
            response_data: row.try_get("response_data")?,
            error_message: row.try_get("error_message")?,
        })
    }
}

#[async_trait]
impl DatabaseItem for Signal {
    type IdType = i64;

    fn id(&self) -> &IdFields<Self::IdType> {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // First, check if a record with this UUID already exists
        if crate::check_exists_by_uuid(pool, "signals", &self.identifiers.global_uuid).await? {
            return Err(anyhow!(
                "Signal with UUID {} already exists",
                self.identifiers.global_uuid
            ));
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let user_requested_uuid = Uuid::parse_str(&self.user_requested_uuid)?;
        let signal_type_str = self.signal_type.as_str();

        sqlx::query!(
            r#"
            INSERT INTO signals (
                global_uuid, user_requested_uuid, workflow_id, initiator_agent_id, rts_id,
                signal_type, initial_data, response_data, error_message, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, ($6::text)::signal_type, $7, $8, $9, $10, $11)
            "#,
            uuid_parsed,
            user_requested_uuid,
            self.workflow_id,
            self.initiator_agent_id,
            self.rts_id,
            signal_type_str,
            &self.initial_data as _,
            &self.response_data as _,
            self.error_message.as_deref(),
            &self.timestamps.created,
            &self.timestamps.updated
        )
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create signal: {}", e))?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let id = self
            .identifiers
            .local_id
            .ok_or_else(|| anyhow!("Cannot update signal without a local ID"))?;

        let signal_type_str = self.signal_type.as_str();
        let user_requested_uuid = Uuid::parse_str(&self.user_requested_uuid)?;

        sqlx::query!(
            r#"
            UPDATE signals SET
                user_requested_uuid = $1,
                workflow_id = $2,
                initiator_agent_id = $3,
                rts_id = $4,
                signal_type = ($5::text)::signal_type,
                initial_data = $6,
                response_data = $7,
                error_message = $8,
                updated_at = $9
            WHERE id = $10
            "#,
            user_requested_uuid,
            self.workflow_id,
            self.initiator_agent_id,
            self.rts_id,
            signal_type_str,
            &self.initial_data as _,
            &self.response_data as _,
            self.error_message.as_deref(),
            &self.timestamps.updated,
            id
        )
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update signal: {}", e))?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        let id = self
            .identifiers
            .local_id
            .ok_or_else(|| anyhow!("Cannot delete signal without a local ID"))?;

        sqlx::query!("DELETE FROM signals WHERE id = $1", id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to delete signal: {}", e))?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, global_uuid, user_requested_uuid, created_at, updated_at,
                workflow_id, initiator_agent_id, rts_id,
                signal_type as "signal_type!: SignalType",
                initial_data, response_data, error_message
            FROM signals
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch all signals: {}", e))?;

        let signals = rows
            .into_iter()
            .map(|row| Signal {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                user_requested_uuid: row.user_requested_uuid.to_string(),
                workflow_id: row.workflow_id,
                initiator_agent_id: row.initiator_agent_id,
                rts_id: row.rts_id,
                signal_type: row.signal_type,
                initial_data: row.initial_data,
                response_data: row.response_data,
                error_message: row.error_message,
            })
            .collect();

        Ok(signals)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        if let Some(local_id) = id.local_id {
            let row_opt = sqlx::query!(
                r#"
                SELECT
                    id, global_uuid, user_requested_uuid, created_at, updated_at,
                    workflow_id, initiator_agent_id, rts_id,
                    signal_type as "signal_type!: SignalType",
                    initial_data, response_data, error_message
                FROM signals
                WHERE id = $1
                "#,
                local_id
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch signal by local ID: {}", e))?;

            Ok(row_opt.map(|row| Signal {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                user_requested_uuid: row.user_requested_uuid.to_string(),
                workflow_id: row.workflow_id,
                initiator_agent_id: row.initiator_agent_id,
                rts_id: row.rts_id,
                signal_type: row.signal_type,
                initial_data: row.initial_data,
                response_data: row.response_data,
                error_message: row.error_message,
            }))
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            let row_opt = sqlx::query!(
                r#"
                SELECT
                    id, global_uuid, user_requested_uuid, created_at, updated_at,
                    workflow_id, initiator_agent_id, rts_id,
                    signal_type as "signal_type!: SignalType",
                    initial_data, response_data, error_message
                FROM signals
                WHERE global_uuid = $1
                "#,
                uuid_parsed
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch signal by UUID: {}", e))?;

            Ok(row_opt.map(|row| Signal {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                user_requested_uuid: row.user_requested_uuid.to_string(),
                workflow_id: row.workflow_id,
                initiator_agent_id: row.initiator_agent_id,
                rts_id: row.rts_id,
                signal_type: row.signal_type,
                initial_data: row.initial_data,
                response_data: row.response_data,
                error_message: row.error_message,
            }))
        }
    }
}
