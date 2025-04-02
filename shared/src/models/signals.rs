use crate::{Agent, RuntimeSession};
use crate::RunningStatus;
use crate::{IdFields, TimestampFields};
use serde_json::Value;
use anyhow::{anyhow, Result};
use sqlx::Row;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

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
                    global_uuid: row.try_get("agent_global_uuid")?,
                },
                timestamps: TimestampFields {
                    created: row.try_get("agent_created_timestamp")?,
                    updated: row.try_get("agent_last_updated_timestamp")?,
                },
                description: row.try_get("agent_description")?,
                agent_state: row.try_get("agent_state")?,
                accepted_completion_rate: row.try_get("agent_accepted_completion_rate")?,
                steps: Vec::new(), // Steps are loaded separately
                completion_count: Arc::new(AtomicU64::new(row.try_get::<i32, _>("agent_completion_count")? as u64)),
                run_count: Arc::new(AtomicU64::new(row.try_get::<i32, _>("agent_run_count")? as u64)),
            })
        } else {
            None
        };

        Ok(Self {
            identifiers: IdFields {
                local_id: row.try_get("id")?,
                global_uuid: row.try_get("global_uuid")?,
            },
            timestamps: TimestampFields {
                created: row.try_get("created_timestamp")?,
                updated: row.try_get("last_updated_timestamp")?,
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

                // TODO Save the results to the database
                Ok(res)
            },
            None => Err(anyhow!("Cannot process signal with no associated data")),
        }
    }
}
