use super::types::Signal;
use crate::models::agents::Agent;
use crate::models::agents::AgentState;
use crate::models::SignalType;
use crate::{DatabaseItem, IdFields, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::Mutex;
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

        // Get the agent if one exists
        let agent = if row.try_get::<Option<i64>, _>("agent_id")?.is_some() {
            Some(Agent {
                identifiers: IdFields {
                    local_id: row.try_get("agent_id")?,
                    global_uuid: row.try_get::<Uuid, _>("agent_global_uuid")?.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.try_get("agent_created_at")?,
                    updated: row.try_get("agent_updated_at")?,
                },
                description: row.try_get("agent_description")?,
                agent_state: Mutex::new(row.try_get("agent_state")?),
                steps: Vec::new(), // Steps are loaded separately
            })
        } else {
            None
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
            signal_type,
            linked_rts: None, // This will be populated after if needed
            agent,
            initial_data: row.try_get("initial_data")?,
            result_data: row.try_get("response_data")?,
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

        // First ensure the linked RuntimeSession is saved if it exists
        if let Some(rts) = &self.linked_rts {
            rts.try_db_create(pool).await?;
        }

        let uuid_parsed = Uuid::parse_str(&self.identifiers.global_uuid)?;
        let user_requested_uuid = Uuid::parse_str(&self.user_requested_uuid)?;
        let signal_type_str = self.signal_type.as_str();

        sqlx::query!(
            r#"
            INSERT INTO signals (
                global_uuid, user_requested_uuid, agent_id, rts_id,
                signal_type, initial_data, response_data, error_message
            ) VALUES ($1, $2, $3, $4, ($5::text)::signal_type, $6, $7, $8)
            "#,
            uuid_parsed,
            user_requested_uuid,
            self.agent.as_ref().and_then(|a| a.identifiers.local_id),
            self.linked_rts
                .as_ref()
                .and_then(|rts| rts.identifiers.local_id),
            signal_type_str,
            &self.initial_data as _,
            &self.result_data as _,
            &self.error_message.as_deref().unwrap_or_default()
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

        // Update the linked RuntimeSession if it exists
        if let Some(rts) = &self.linked_rts {
            rts.try_db_update(pool).await?;
        }

        let signal_type_str = self.signal_type.as_str();
        let user_requested_uuid = Uuid::parse_str(&self.user_requested_uuid)?;

        sqlx::query!(
            r#"
            UPDATE signals SET
                user_requested_uuid = $1,
                agent_id = $2,
                rts_id = $3,
                signal_type = ($4::text)::signal_type,
                initial_data = $5,
                response_data = $6,
                error_message = $7,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $8
            "#,
            user_requested_uuid,
            self.agent.as_ref().and_then(|a| a.identifiers.local_id),
            self.linked_rts
                .as_ref()
                .and_then(|rts| rts.identifiers.local_id),
            signal_type_str,
            &self.initial_data as _,
            &self.result_data as _,
            &self.error_message.as_deref().unwrap_or_default(),
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
        // Define struct compatible with query_as! output
        struct SignalRow {
            id: i64,
            global_uuid: uuid::Uuid,
            user_requested_uuid: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            agent_id: Option<i32>,
            agent_global_uuid: Option<uuid::Uuid>,
            agent_created_at: Option<chrono::DateTime<chrono::Utc>>,
            agent_updated_at: Option<chrono::DateTime<chrono::Utc>>,
            agent_description: Option<String>,
            agent_state: Option<AgentState>,
            #[allow(dead_code)]
            /// This field is required to match the SQL query structure but is handled
            /// separately through RuntimeSession loading after row mapping
            rts_id: Option<i64>,
            signal_type: SignalType,
            initial_data: Option<serde_json::Value>,
            response_data: Option<serde_json::Value>,
            error_message: Option<String>,
        }

        let rows = sqlx::query_as!(
            SignalRow,
            r#"
            SELECT
                s.id, s.global_uuid, s.user_requested_uuid,
                s.created_at, s.updated_at,
                s.signal_type as "signal_type: _",
                s.initial_data as "initial_data: serde_json::Value",
                s.response_data as "response_data: serde_json::Value",
                s.error_message,
                s.rts_id,
                a.id as agent_id,
                a.global_uuid as agent_global_uuid,
                a.created_at as agent_created_at,
                a.updated_at as agent_updated_at,
                a.description as agent_description,
                a.agent_state as "agent_state: AgentState"
            FROM signals s
            LEFT JOIN agents a ON s.agent_id = a.id
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut signals = Vec::with_capacity(rows.len());
        for row in rows {
            let signal = Signal {
                identifiers: IdFields {
                    local_id: Some(row.id),
                    global_uuid: row.global_uuid.to_string(),
                },
                timestamps: TimestampFields {
                    created: row.created_at,
                    updated: row.updated_at,
                },
                user_requested_uuid: row.user_requested_uuid,
                agent: if row.agent_id.is_some() {
                    Some(Agent {
                        identifiers: IdFields {
                            local_id: row.agent_id,
                            global_uuid: row
                                .agent_global_uuid
                                .map(|uuid| uuid.to_string())
                                .unwrap_or_default(),
                        },
                        timestamps: TimestampFields {
                            created: row.agent_created_at.unwrap_or_default(),
                            updated: row.agent_updated_at.unwrap_or_default(),
                        },
                        description: row.agent_description.unwrap_or_default(),
                        agent_state: Mutex::new(row.agent_state.unwrap_or_default()),
                        steps: Vec::new(), // Steps are loaded separately
                    })
                } else {
                    None
                },
                linked_rts: None, // Will be populated after if needed
                signal_type: row.signal_type,
                initial_data: row.initial_data,
                result_data: row.response_data,
                error_message: row.error_message,
            };

            signals.push(signal);
        }

        Ok(signals)
    }

    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>> {
        let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
        let row = sqlx::query!(
            r#"
            SELECT
                s.id, s.global_uuid, s.user_requested_uuid,
                s.created_at, s.updated_at,
                s.signal_type as "signal_type!: crate::models::signals::SignalType",
                s.initial_data as "initial_data: Value",
                s.response_data as "response_data: Value",
                s.error_message,
                a.id as "agent_id?",
                a.global_uuid as "agent_global_uuid?",
                a.created_at as "agent_created_at?",
                a.updated_at as "agent_updated_at?",
                a.description as "agent_description?",
                a.agent_state as "agent_state?: crate::models::agents::AgentState"
            FROM signals s
            LEFT JOIN agents a ON s.agent_id = a.id
            WHERE s.global_uuid = $1
            "#,
            uuid_parsed
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| Signal {
            identifiers: IdFields {
                local_id: Some(row.id),
                global_uuid: row.global_uuid.to_string(),
            },
            timestamps: TimestampFields {
                created: row.created_at,
                updated: row.updated_at,
            },
            user_requested_uuid: row.user_requested_uuid.to_string(),
            agent: if row.agent_id.is_some() {
                Some(Agent {
                    identifiers: IdFields {
                        local_id: row.agent_id,
                        global_uuid: row.agent_global_uuid.unwrap().to_string(),
                    },
                    timestamps: TimestampFields {
                        created: row.agent_created_at.unwrap(),
                        updated: row.agent_updated_at.unwrap(),
                    },
                    description: row.agent_description.unwrap_or_default(),
                    agent_state: Mutex::new(row.agent_state.unwrap_or_default()),
                    steps: Vec::new(), // Steps are loaded separately
                })
            } else {
                None
            },
            linked_rts: None, // Will be populated after if needed
            signal_type: row.signal_type,
            initial_data: row.initial_data,
            result_data: row.response_data,
            error_message: row.error_message,
        }))
    }
}
