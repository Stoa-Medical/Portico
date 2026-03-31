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
            source: row.try_get("source").unwrap_or(None),
            idempotency_key: row.try_get("idempotency_key").unwrap_or(None),
            source_metadata: row.try_get("source_metadata").unwrap_or(None),
            leased_at: row.try_get("leased_at").unwrap_or(None),
            lease_expires_at: row.try_get("lease_expires_at").unwrap_or(None),
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

        sqlx::query(
            r#"
            INSERT INTO signals (
                global_uuid, user_requested_uuid, workflow_id, initiator_agent_id, rts_id,
                signal_type, initial_data, response_data, error_message,
                source, idempotency_key, source_metadata, leased_at, lease_expires_at,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, ($6::text)::signal_type, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(uuid_parsed)
        .bind(user_requested_uuid)
        .bind(self.workflow_id)
        .bind(self.initiator_agent_id)
        .bind(self.rts_id)
        .bind(signal_type_str)
        .bind(&self.initial_data)
        .bind(&self.response_data)
        .bind(self.error_message.as_deref())
        .bind(self.source.as_deref())
        .bind(self.idempotency_key.as_deref())
        .bind(&self.source_metadata)
        .bind(&self.leased_at)
        .bind(&self.lease_expires_at)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
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

        sqlx::query(
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
                source = $9,
                idempotency_key = $10,
                source_metadata = $11,
                leased_at = $12,
                lease_expires_at = $13,
                updated_at = $14
            WHERE id = $15
            "#,
        )
        .bind(user_requested_uuid)
        .bind(self.workflow_id)
        .bind(self.initiator_agent_id)
        .bind(self.rts_id)
        .bind(signal_type_str)
        .bind(&self.initial_data)
        .bind(&self.response_data)
        .bind(self.error_message.as_deref())
        .bind(self.source.as_deref())
        .bind(self.idempotency_key.as_deref())
        .bind(&self.source_metadata)
        .bind(&self.leased_at)
        .bind(&self.lease_expires_at)
        .bind(&self.timestamps.updated)
        .bind(id)
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

        sqlx::query("DELETE FROM signals WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to delete signal: {}", e))?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, global_uuid, user_requested_uuid, created_at, updated_at,
                workflow_id, initiator_agent_id, rts_id,
                signal_type,
                initial_data, response_data, error_message,
                source, idempotency_key, source_metadata, leased_at, lease_expires_at
            FROM signals
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch all signals: {}", e))?;

        let signals = rows
            .iter()
            .map(|row| {
                let signal_type_str: &str = row.try_get("signal_type")?;
                let signal_type: SignalType = signal_type_str
                    .parse()
                    .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid signal type: {}", e)))))?;

                Ok(Signal {
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
                    source: row.try_get("source").unwrap_or(None),
                    idempotency_key: row.try_get("idempotency_key").unwrap_or(None),
                    source_metadata: row.try_get("source_metadata").unwrap_or(None),
                    leased_at: row.try_get("leased_at").unwrap_or(None),
                    lease_expires_at: row.try_get("lease_expires_at").unwrap_or(None),
                })
            })
            .collect::<std::result::Result<Vec<_>, sqlx::Error>>()
            .map_err(|e| anyhow!("Failed to decode signal row: {}", e))?;

        Ok(signals)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query(
                r#"
                SELECT
                    id, global_uuid, user_requested_uuid, created_at, updated_at,
                    workflow_id, initiator_agent_id, rts_id,
                    signal_type,
                    initial_data, response_data, error_message,
                    source, idempotency_key, source_metadata, leased_at, lease_expires_at
                FROM signals
                WHERE id = $1
                "#,
            )
            .bind(local_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch signal by local ID: {}", e))?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query(
                r#"
                SELECT
                    id, global_uuid, user_requested_uuid, created_at, updated_at,
                    workflow_id, initiator_agent_id, rts_id,
                    signal_type,
                    initial_data, response_data, error_message,
                    source, idempotency_key, source_metadata, leased_at, lease_expires_at
                FROM signals
                WHERE global_uuid = $1
                "#,
            )
            .bind(uuid_parsed)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch signal by UUID: {}", e))?
        };

        match row_opt {
            None => Ok(None),
            Some(row) => {
                let signal_type_str: &str = row.try_get("signal_type")
                    .map_err(|e| anyhow!("Failed to decode signal_type: {}", e))?;
                let signal_type: SignalType = signal_type_str
                    .parse()
                    .map_err(|e| anyhow!("Invalid signal type: {}", e))?;

                Ok(Some(Signal {
                    identifiers: IdFields {
                        local_id: row.try_get("id")
                            .map_err(|e| anyhow!("Failed to decode id: {}", e))?,
                        global_uuid: row.try_get::<Uuid, _>("global_uuid")
                            .map_err(|e| anyhow!("Failed to decode global_uuid: {}", e))?
                            .to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.try_get("created_at")
                            .map_err(|e| anyhow!("Failed to decode created_at: {}", e))?,
                        updated: row.try_get("updated_at")
                            .map_err(|e| anyhow!("Failed to decode updated_at: {}", e))?,
                    },
                    user_requested_uuid: row.try_get::<Uuid, _>("user_requested_uuid")
                        .map_err(|e| anyhow!("Failed to decode user_requested_uuid: {}", e))?
                        .to_string(),
                    workflow_id: row.try_get("workflow_id")
                        .map_err(|e| anyhow!("Failed to decode workflow_id: {}", e))?,
                    initiator_agent_id: row.try_get("initiator_agent_id")
                        .map_err(|e| anyhow!("Failed to decode initiator_agent_id: {}", e))?,
                    rts_id: row.try_get("rts_id")
                        .map_err(|e| anyhow!("Failed to decode rts_id: {}", e))?,
                    signal_type,
                    initial_data: row.try_get("initial_data")
                        .map_err(|e| anyhow!("Failed to decode initial_data: {}", e))?,
                    response_data: row.try_get("response_data")
                        .map_err(|e| anyhow!("Failed to decode response_data: {}", e))?,
                    error_message: row.try_get("error_message")
                        .map_err(|e| anyhow!("Failed to decode error_message: {}", e))?,
                    source: row.try_get("source").unwrap_or(None),
                    idempotency_key: row.try_get("idempotency_key").unwrap_or(None),
                    source_metadata: row.try_get("source_metadata").unwrap_or(None),
                    leased_at: row.try_get("leased_at").unwrap_or(None),
                    lease_expires_at: row.try_get("lease_expires_at").unwrap_or(None),
                }))
            }
        }
    }
}
