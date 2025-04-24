use crate::models::agents::Agent;
use crate::models::agents::AgentState;
use crate::models::runtime_sessions::RuntimeSession;
use crate::{DatabaseItem, IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SignalType {
    Command,
    Sync,
    Fyi,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::Command => "command",
            SignalType::Sync => "sync",
            SignalType::Fyi => "fyi",
        }
    }
}

impl FromStr for SignalType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "command" => Ok(SignalType::Command),
            "sync" => Ok(SignalType::Sync),
            "fyi" => Ok(SignalType::Fyi),
            _ => Err(format!("Invalid signal type: {}", s)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for SignalType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("signal_type")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SignalType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.as_str()? {
            "command" => Ok(SignalType::Command),
            "sync" => Ok(SignalType::Sync),
            "fyi" => Ok(SignalType::Fyi),
            _ => Err("Invalid signal type".into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SignalType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.as_str();
        buf.extend_from_slice(s.as_bytes());
        Ok(sqlx::encode::IsNull::No)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandPayload {
    pub command: String,
    pub payload: CommandDataPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDataPayload {
    pub id: String,
    pub properties: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub scope: String,
    pub mode: String,
    pub targets: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Signal {
    pub identifiers: IdFields<i64>,
    pub timestamps: TimestampFields,
    pub user_requested_uuid: String,
    pub agent: Option<Agent>,
    pub linked_rts: Option<RuntimeSession>,
    pub signal_type: SignalType,
    pub initial_data: Option<Value>,
    pub result_data: Option<Value>,
    pub error_message: Option<String>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Signal {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        // Get the signal type
        let signal_type = if let Ok(signal_type_str) = row.try_get::<&str, _>("signal_type") {
            SignalType::from_str(signal_type_str).unwrap_or(SignalType::Fyi)
        } else {
            SignalType::Fyi
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

impl Signal {
    pub fn new(
        identifiers: IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<Agent>,
        signal_type: SignalType,
        initial_data: Option<Value>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            user_requested_uuid,
            agent,
            linked_rts: None,
            signal_type,
            initial_data,
            result_data: None,
            error_message: None,
        }
    }

    pub fn new_command(
        identifiers: IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<Agent>,
        command_payload: CommandPayload,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            agent,
            SignalType::Command,
            Some(serde_json::to_value(command_payload).unwrap_or(Value::Null)),
        )
    }

    pub fn new_sync(
        identifiers: IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<Agent>,
        sync_payload: SyncPayload,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            agent,
            SignalType::Sync,
            Some(serde_json::to_value(sync_payload).unwrap_or(Value::Null)),
        )
    }

    pub fn new_fyi(
        identifiers: IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<Agent>,
        data: Value,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            agent,
            SignalType::Fyi,
            Some(data),
        )
    }

    pub fn parse_command_payload(&self) -> Result<CommandPayload> {
        match &self.initial_data {
            Some(data) if self.signal_type == SignalType::Command => {
                serde_json::from_value(data.clone()).map_err(|e| anyhow!("Invalid command payload: {}", e))
            }
            _ => Err(anyhow!("Not a command signal or missing data")),
        }
    }

    pub fn parse_sync_payload(&self) -> Result<SyncPayload> {
        match &self.initial_data {
            Some(data) if self.signal_type == SignalType::Sync => {
                serde_json::from_value(data.clone()).map_err(|e| anyhow!("Invalid sync payload: {}", e))
            }
            _ => Err(anyhow!("Not a sync signal or missing data")),
        }
    }

    pub fn parse_fyi_data(&self) -> Result<Value> {
        match &self.initial_data {
            Some(data) if self.signal_type == SignalType::Fyi => {
                Ok(data.clone())
            }
            _ => Err(anyhow!("Not an FYI signal or missing data")),
        }
    }

    pub async fn process(&mut self) -> Result<()> {
        match self.execute_signal().await {
            Ok(runtime_session) => {
                self.linked_rts = Some(runtime_session);
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
                Err(e)
            }
        }
    }

    async fn execute_signal(&self) -> Result<RuntimeSession> {
        match &self.agent {
            Some(agent) => {
                let result = agent.run(self.initial_data.clone().unwrap_or(Value::Null)).await?;
                Ok(result)
            }
            None => {
                let error_msg = match self.signal_type {
                    SignalType::Command => "Cannot process command signal with no associated agent",
                    SignalType::Sync => "Cannot process sync signal with no associated agent",
                    SignalType::Fyi => "FYI signal requires an agent to process",
                };
                Err(anyhow!(error_msg))
            }
        }
    }
}

impl JsonLike for Signal {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "user_requested_uuid": self.user_requested_uuid,
            "agent": self.agent.as_ref().map(|a| a.to_json()),
            "linked_rts_id": self.linked_rts.as_ref().and_then(|rts| rts.identifiers.local_id),
            "signal_type": self.signal_type.as_str(),
            "initial_data": self.initial_data,
            "result_data": self.result_data,
            "error_message": self.error_message
        })
    }

    fn update_from_json(&mut self, obj: Value) -> Result<Vec<String>> {
        let mut updated_fields = Vec::new();

        if let Some(obj_map) = obj.as_object() {
            for (key, value) in obj_map {
                match key.as_str() {
                    "user_requested_uuid" => {
                        if let Some(new_uuid) = value.as_str() {
                            if self.user_requested_uuid != new_uuid {
                                self.user_requested_uuid = new_uuid.to_string();
                                updated_fields.push(key.to_string());
                            }
                        }
                    }
                    "agent" => {
                        match &mut self.agent {
                            Some(agent) => {
                                // If we have an agent, try to update it
                                if !value.is_null() {
                                    if let Ok(fields) = agent.update_from_json(value.clone()) {
                                        if !fields.is_empty() {
                                            updated_fields
                                                .push(format!("agent.{}", fields.join(", agent.")));
                                        }
                                    }
                                } else {
                                    // If JSON value is null, remove the agent
                                    self.agent = None;
                                    updated_fields.push(key.to_string());
                                }
                            }
                            None => {
                                // If we don't have an agent, try to create one
                                if !value.is_null() {
                                    if let Ok(new_agent) = Agent::from_json(value.clone()) {
                                        self.agent = Some(new_agent);
                                        updated_fields.push(key.to_string());
                                    }
                                }
                            }
                        }
                    }
                    "linked_rts_id" => {
                        // We can't directly update RuntimeSession from just an ID here
                        // This would require a database lookup, so we'll just note that
                        // it was requested but will need to be loaded separately
                        if value.is_null() {
                            if self.linked_rts.is_some() {
                                self.linked_rts = None;
                                updated_fields.push(key.to_string());
                            }
                        }
                        // Note: Setting linked_rts from just an ID is not possible here
                        // without additional DB queries - would need to be handled separately
                    }
                    "signal_type" => {
                        if let Some(new_type) = value.as_str() {
                            match SignalType::from_str(new_type) {
                                Ok(new_signal_type) => {
                                    if self.signal_type != new_signal_type {
                                        self.signal_type = new_signal_type;
                                        updated_fields.push(key.to_string());
                                    }
                                }
                                Err(e) => {
                                    return Err(anyhow!("Invalid signal type '{}': {}", new_type, e))
                                }
                            }
                        }
                    }
                    "initial_data" => {
                        // For JSON data fields, just compare stringified versions to detect changes
                        let current_json = self
                            .initial_data
                            .as_ref()
                            .map(|v| serde_json::to_string(v).unwrap_or_default())
                            .unwrap_or_default();
                        let new_json = serde_json::to_string(value).unwrap_or_default();

                        if value.is_null() {
                            if self.initial_data.is_some() {
                                self.initial_data = None;
                                updated_fields.push(key.to_string());
                            }
                        } else if current_json != new_json {
                            self.initial_data = Some(value.clone());
                            updated_fields.push(key.to_string());
                        }
                    }
                    "result_data" => {
                        // Similar approach for result_data
                        let current_json = self
                            .result_data
                            .as_ref()
                            .map(|v| serde_json::to_string(v).unwrap_or_default())
                            .unwrap_or_default();
                        let new_json = serde_json::to_string(value).unwrap_or_default();

                        if value.is_null() {
                            if self.result_data.is_some() {
                                self.result_data = None;
                                updated_fields.push(key.to_string());
                            }
                        } else if current_json != new_json {
                            self.result_data = Some(value.clone());
                            updated_fields.push(key.to_string());
                        }
                    }
                    "error_message" => {
                        if value.is_null() {
                            if self.error_message.is_some() {
                                self.error_message = None;
                                updated_fields.push(key.to_string());
                            }
                        } else if let Some(new_msg) = value.as_str() {
                            let current = self.error_message.as_deref().unwrap_or("");
                            if current != new_msg {
                                self.error_message = Some(new_msg.to_string());
                                updated_fields.push(key.to_string());
                            }
                        }
                    }
                    // Skip fields that shouldn't be updated directly
                    "id" | "global_uuid" | "created_at" | "updated_at" => {
                        // These fields are skipped intentionally
                    }
                    // Unknown fields
                    _ => {
                        // Optionally: log or warn about unknown fields
                    }
                }
            }
        }

        self.timestamps.update();
        updated_fields.push("updated_at".to_string());

        Ok(updated_fields)
    }

    fn from_json(obj: Value) -> Result<Self> {
        // Required fields
        let global_uuid = obj
            .get("global_uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid global_uuid"))?
            .to_string();

        let user_requested_uuid = obj
            .get("user_requested_uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid user_requested_uuid"))?
            .to_string();

        let signal_type_str = obj
            .get("signal_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing or invalid signal_type"))?;

        let signal_type = SignalType::from_str(signal_type_str)
            .map_err(|e| anyhow!("Invalid signal type: {}", e))?;

        // Optional fields
        let local_id = obj.get("id").and_then(|v| v.as_i64()).map(|id| id as i64);

        let agent = if let Some(agent_obj) = obj.get("agent") {
            if agent_obj.is_null() {
                None
            } else {
                Some(Agent::from_json(agent_obj.clone())?)
            }
        } else {
            None
        };

        let initial_data = obj.get("initial_data").cloned();
        let result_data = obj.get("result_data").cloned();
        let error_message = obj
            .get("error_message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Note: linked_rts can't be fully constructed from JSON alone
        // as it would require database queries

        // Build the Signal
        Ok(Signal {
            identifiers: IdFields {
                local_id,
                global_uuid,
            },
            timestamps: TimestampFields::new(),
            user_requested_uuid,
            agent,
            linked_rts: None, // This would need to be loaded separately
            signal_type,
            initial_data,
            result_data,
            error_message,
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
            self.linked_rts.as_ref().and_then(|rts| rts.identifiers.local_id),
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
            self.linked_rts.as_ref().and_then(|rts| rts.identifiers.local_id),
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
                            global_uuid: row.agent_global_uuid.map(|uuid| uuid.to_string()).unwrap_or_default(),
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

    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields<Self::IdType>) -> Result<Option<Self>> {
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

        let row_opt = if let Some(local_id) = id.local_id {
            sqlx::query_as!(
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
                WHERE s.id = $1
                "#,
                local_id
            )
            .fetch_optional(pool)
            .await?
        } else {
            let uuid_parsed = Uuid::parse_str(&id.global_uuid)?;
            sqlx::query_as!(
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
                WHERE s.global_uuid = $1
                "#,
                uuid_parsed
            )
            .fetch_optional(pool)
            .await?
        };

        Ok(row_opt.map(|row| {
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
                            global_uuid: row.agent_global_uuid.map(|uuid| uuid.to_string()).unwrap_or_default(),
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

            // Note: We can't do async operations in this closure
            // Instead, we'll return the signal as is and let the caller handle RuntimeSession loading if needed
            signal
        }))
    }
}
