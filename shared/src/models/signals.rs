use crate::{Agent, RuntimeSession};
use crate::RunningStatus;
use crate::{IdFields, TimestampFields, DatabaseItem};
use serde_json::Value;
use anyhow::{anyhow, Result};
use sqlx::{Row, PgPool};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use async_trait::async_trait;

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

#[async_trait]
impl DatabaseItem for Signal {
    fn id(&self) -> &IdFields {
        &self.identifiers
    }

    async fn try_db_create(&self, pool: &PgPool) -> Result<()> {
        let agent_id = self.agent.as_ref().and_then(|a| a.identifiers.local_id);
        let agent_uuid = self.agent.as_ref().map(|a| a.identifiers.global_uuid.clone());

        sqlx::query(
            r#"
            INSERT INTO signals (
                global_uuid, user_requested_uuid, agent_id, agent_uuid,
                signal_type, signal_status, initial_data, response_data,
                error_message, created_timestamp, last_updated_timestamp
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(&self.identifiers.global_uuid)
        .bind(&self.user_requested_uuid)
        .bind(agent_id)
        .bind(agent_uuid)
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
        let agent_uuid = self.agent.as_ref().map(|a| a.identifiers.global_uuid.clone());

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
                last_updated_timestamp = $9
            WHERE global_uuid = $10
            "#
        )
        .bind(&self.user_requested_uuid)
        .bind(agent_id)
        .bind(agent_uuid)
        .bind(&self.signal_type)
        .bind(&self.status)
        .bind(&self.initial_data)
        .bind(&self.result_data)
        .bind(&self.error_message)
        .bind(&self.timestamps.updated)
        .bind(&self.identifiers.global_uuid)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn try_db_delete(&self, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM signals WHERE global_uuid = $1")
            .bind(&self.identifiers.global_uuid)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>> {
        // Fetch signals with agent info using a JOIN
        let rows = sqlx::query_as::<_, Signal>(
            r#"
            SELECT
                s.*,
                a.id as agent_id,
                a.global_uuid as agent_global_uuid,
                a.description as agent_description,
                a.agent_state as agent_state,
                a.accepted_completion_rate as agent_accepted_completion_rate,
                a.completion_count as agent_completion_count,
                a.run_count as agent_run_count,
                a.created_timestamp as agent_created_timestamp,
                a.last_updated_timestamp as agent_last_updated_timestamp
            FROM signals s
            LEFT JOIN agents a ON s.agent_id = a.id
            "#
        )
        .fetch_all(pool)
        .await?;

        // For each signal that has an agent, we need to load its steps
        let mut signals = Vec::with_capacity(rows.len());

        for mut signal in rows {
            if let Some(agent) = &mut signal.agent {
                // Use the existing query pattern to load steps for this agent
                if let Some(agent_id) = agent.identifiers.local_id {
                    let steps_query = sqlx::query(
                        r#"
                        SELECT json_agg(json_build_object(
                            'id', s.id,
                            'global_uuid', s.global_uuid,
                            'created_timestamp', s.created_timestamp,
                            'last_updated_timestamp', s.last_updated_timestamp,
                            'name', s.name,
                            'description', s.description,
                            'step_type', s.step_type,
                            'step_content', s.step_content,
                            'success_count', s.success_count,
                            'run_count', s.run_count
                        )) as steps
                        FROM steps s
                        WHERE s.agent_id = $1
                        ORDER BY s.sequence_number
                        "#
                    )
                    .bind(agent_id)
                    .fetch_one(pool)
                    .await?;

                    let steps_json: Option<Value> = steps_query.get("steps");

                    if let Some(json) = steps_json {
                        agent.steps = crate::Step::from_json_array(&json);
                    }
                }
            }
            signals.push(signal);
        }

        Ok(signals)
    }

    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields) -> Result<Option<Self>> {
        let query = if let Some(local_id) = id.local_id {
            sqlx::query_as::<_, Signal>(
                r#"
                SELECT
                    s.*,
                    a.id as agent_id,
                    a.global_uuid as agent_global_uuid,
                    a.description as agent_description,
                    a.agent_state as agent_state,
                    a.accepted_completion_rate as agent_accepted_completion_rate,
                    a.completion_count as agent_completion_count,
                    a.run_count as agent_run_count,
                    a.created_timestamp as agent_created_timestamp,
                    a.last_updated_timestamp as agent_last_updated_timestamp
                FROM signals s
                LEFT JOIN agents a ON s.agent_id = a.id
                WHERE s.id = $1
                "#
            )
            .bind(local_id)
        } else {
            sqlx::query_as::<_, Signal>(
                r#"
                SELECT
                    s.*,
                    a.id as agent_id,
                    a.global_uuid as agent_global_uuid,
                    a.description as agent_description,
                    a.agent_state as agent_state,
                    a.accepted_completion_rate as agent_accepted_completion_rate,
                    a.completion_count as agent_completion_count,
                    a.run_count as agent_run_count,
                    a.created_timestamp as agent_created_timestamp,
                    a.last_updated_timestamp as agent_last_updated_timestamp
                FROM signals s
                LEFT JOIN agents a ON s.agent_id = a.id
                WHERE s.global_uuid = $1
                "#
            )
            .bind(&id.global_uuid)
        };

        let result = query.fetch_optional(pool).await?;

        if let Some(mut signal) = result {
            // If the signal has an agent, load its steps
            if let Some(agent) = &mut signal.agent {
                if let Some(agent_id) = agent.identifiers.local_id {
                    let steps_query = sqlx::query(
                        r#"
                        SELECT json_agg(json_build_object(
                            'id', s.id,
                            'global_uuid', s.global_uuid,
                            'created_timestamp', s.created_timestamp,
                            'last_updated_timestamp', s.last_updated_timestamp,
                            'name', s.name,
                            'description', s.description,
                            'step_type', s.step_type,
                            'step_content', s.step_content,
                            'success_count', s.success_count,
                            'run_count', s.run_count
                        )) as steps
                        FROM steps s
                        WHERE s.agent_id = $1
                        ORDER BY s.sequence_number
                        "#
                    )
                    .bind(agent_id)
                    .fetch_one(pool)
                    .await?;

                    let steps_json: Option<Value> = steps_query.get("steps");

                    if let Some(json) = steps_json {
                        agent.steps = crate::Step::from_json_array(&json);
                    }
                }
            }
            return Ok(Some(signal));
        }

        Ok(None)
    }
}
