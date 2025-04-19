use crate::models::agents::Agent;
use crate::models::runtime_sessions::RuntimeSession;
use crate::RunningStatus;
use crate::{DatabaseItem, IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug)]
pub struct Signal {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub user_requested_uuid: String,
    pub agent: Option<Agent>,
    pub status: RunningStatus, // TODO: Should this just link to a `RuntimeSession`?
    pub signal_type: String,
    pub initial_data: Option<Value>,
    pub result_data: Option<Value>,
    pub error_message: Option<String>,
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Signal {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        // For the agent field, we'll create a new Agent directly from the prefixed columns
        let agent = if row.try_get::<Option<i32>, _>("agent_id")?.is_some() {
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
            user_requested_uuid: row.try_get("user_requested_uuid")?,
            signal_type: row.try_get("signal_type")?,
            status: row.try_get("signal_status")?,
            agent,
            initial_data: row.try_get("initial_data")?,
            result_data: row.try_get("response_data")?,
            error_message: row.try_get("error_message")?,
        })
    }
}

impl Signal {
    pub fn new(
        identifiers: IdFields,
        user_requested_uuid: String,
        agent: Option<Agent>,
        signal_type: String,
        initial_data: Option<Value>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            user_requested_uuid,
            agent,
            status: RunningStatus::Waiting,
            signal_type,
            initial_data,
            result_data: None,
            error_message: None,
        }
    }

    pub async fn process(&mut self) -> Result<RuntimeSession> {
        // Check if there is data
        match &self.initial_data {
            Some(data) => {
                // Execute the requested Agent
                self.status = RunningStatus::Running;

                // Run the `Agent`
                let res = match &mut self.agent {
                    Some(agent) => agent.run(data.clone()).await,
                    None => Err(anyhow!("Cannot process signal with no associated agent")),
                }?;
                self.status = RunningStatus::Completed;

                Ok(res)
            }
            None => Err(anyhow!("Cannot process signal with no associated data")),
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
            "status": self.status,
            "signal_type": self.signal_type,
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
                    "status" => {
                        if let Some(status_str) = value.as_str() {
                            match RunningStatus::from_str(status_str) {
                                Ok(new_status) => {
                                    if self.status != new_status {
                                        self.status = new_status;
                                        updated_fields.push(key.to_string());
                                    }
                                }
                                Err(e) => {
                                    return Err(anyhow!("Invalid status '{}': {}", status_str, e))
                                }
                            }
                        }
                    }
                    "signal_type" => {
                        if let Some(new_type) = value.as_str() {
                            if self.signal_type != new_type {
                                self.signal_type = new_type.to_string();
                                updated_fields.push(key.to_string());
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

            // If any fields were updated, update the timestamp
            if !updated_fields.is_empty() {
                self.timestamps.update();
                updated_fields.push("updated_at".to_string());
            }

            Ok(updated_fields)
        } else {
            Err(anyhow!("Expected JSON object"))
        }
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
                user_requested_uuid: obj
                    .get("user_requested_uuid")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                agent: obj
                    .get("agent")
                    .and_then(|v| Agent::from_json(v.clone()).ok()),
                status: obj
                    .get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| RunningStatus::from_str(s).ok())
                    .unwrap_or_default(),
                signal_type: obj
                    .get("signal_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                initial_data: obj.get("initial_data").cloned(),
                result_data: obj.get("result_data").cloned(),
                error_message: obj
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        } else {
            Err(anyhow!("Expected JSON object"))
        }
    }
}

#[async_trait]
impl DatabaseItem for Signal {
    fn id(&self) -> &IdFields {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        // Check if a signal with the same UUID already exists
        if crate::check_exists_by_uuid(pool, "signals", &self.identifiers.global_uuid).await? {
            return Ok(());  // Signal already exists, no need to create it again
        }

        let agent_id = self.agent.as_ref().and_then(|a| a.identifiers.local_id);
        let agent_uuid = self
            .agent
            .as_ref()
            .map(|a| a.identifiers.global_uuid.clone());

        sqlx::query(
            r#"
            INSERT INTO signals (
                global_uuid, user_requested_uuid, agent_id, agent_uuid,
                signal_type, signal_status, initial_data, response_data,
                error_message, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
        .bind(&self.user_requested_uuid)
        .bind(agent_id)
        .bind(agent_uuid.as_deref().map(Uuid::parse_str).transpose()?)
        .bind(&self.signal_type)
        .bind(&self.status)
        .bind(&self.initial_data)
        .bind(&self.result_data)
        .bind(&self.error_message)
        .bind(&self.timestamps.created)
        .bind(&self.timestamps.updated)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_update(&self, pool: &PgPool) -> Result<()> {
        let agent_id = self.agent.as_ref().and_then(|a| a.identifiers.local_id);
        let agent_uuid = self
            .agent
            .as_ref()
            .map(|a| a.identifiers.global_uuid.clone());

        sqlx::query(
            r#"
            UPDATE signals
            SET user_requested_uuid = $1,
                agent_id = $2,
                agent_uuid = $3,
                signal_type = $4,
                signal_status = $5,
                initial_data = $6,
                response_data = $7,
                error_message = $8,
                updated_at = $9
            WHERE global_uuid = $10
            "#,
        )
        .bind(&self.user_requested_uuid)
        .bind(agent_id)
        .bind(agent_uuid.as_deref().map(Uuid::parse_str).transpose()?)
        .bind(&self.signal_type)
        .bind(&self.status)
        .bind(&self.initial_data)
        .bind(&self.result_data)
        .bind(&self.error_message)
        .bind(&self.timestamps.updated)
        .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM signals WHERE global_uuid = $1")
            .bind(Uuid::parse_str(&self.identifiers.global_uuid)?)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        // Fetch signals with agent info using a JOIN
        let query = crate::signal_with_agent_sql("");

        let rows = sqlx::query_as::<_, Signal>(&query)
            .fetch_all(pool)
            .await?;

        // For each signal that has an agent, we need to load its steps
        let mut signals = Vec::with_capacity(rows.len());

        for mut signal in rows {
            if let Some(agent) = &mut signal.agent {
                // Use the helper function for loading steps
                if let Some(agent_id) = agent.identifiers.local_id {
                    if let Ok(Some(steps_json)) = crate::load_agent_steps(pool, agent_id).await {
                        agent.steps = crate::Step::from_json_array(&steps_json);
                    }
                }
            }
            signals.push(signal);
        }

        Ok(signals)
    }

    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields) -> Result<Option<Self>> {
        let query = crate::signal_with_agent_sql("WHERE s.global_uuid = $1");

        let result = if let Some(local_id) = id.local_id {
            sqlx::query_as::<_, Signal>(&query)
                .bind(local_id)
                .fetch_optional(pool)
                .await?
        } else {
            sqlx::query_as::<_, Signal>(&query)
                .bind(Uuid::parse_str(&id.global_uuid)?)
                .fetch_optional(pool)
                .await?
        };

        if let Some(mut signal) = result {
            // If the signal has an agent, load its steps
            if let Some(agent) = &mut signal.agent {
                if let Some(agent_id) = agent.identifiers.local_id {
                    if let Ok(Some(steps_json)) = crate::load_agent_steps(pool, agent_id).await {
                        agent.steps = crate::Step::from_json_array(&steps_json);
                    }
                }
            }
            return Ok(Some(signal));
        }

        Ok(None)
    }
}
