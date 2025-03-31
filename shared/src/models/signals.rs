use crate::{Agent, RuntimeSession};
use crate::RunningStatus;
use crate::{IdFields, TimestampFields};
use serde_json::Value;
use anyhow::{anyhow, Result};

#[derive(sqlx::FromRow)]
pub struct Signal {
    #[sqlx(flatten)]
    identifiers: IdFields,
    #[sqlx(flatten)]
    timestamps: TimestampFields,
    user_requested_uuid: String,
    agent: Agent,
    status: RunningStatus, // TODO: Should this just link to a `RuntimeSession`?
    description: String,
    initial_data: Option<Value>,
    result_data: Option<Value>,
    error_message: Option<String>,
}

impl Signal {
    pub fn new(
        identifiers: IdFields,
        user_requested_uuid: String,
        agent: Agent,
        description: String,
        initial_data: Option<Value>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            user_requested_uuid,
            agent,
            status: RunningStatus::Waiting,
            description,
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
                let res = self.agent.run(data.clone()).await;
                self.status = RunningStatus::Completed;

                // TODO Save the results to the database
                res
            },
            None => Err(anyhow!("Cannot process signal with no associated data")),
        }
    }
}
