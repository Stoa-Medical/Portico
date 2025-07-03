use super::types::{RunPayload, Signal, SignalType, SyncPayload};
use anyhow::{anyhow, Result};
use serde_json::Value;

impl Signal {
    pub fn new(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<crate::models::agents::Agent>,
        signal_type: SignalType,
        initial_data: Option<Value>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: crate::TimestampFields::new(),
            user_requested_uuid,
            agent,
            linked_rts: None,
            signal_type,
            initial_data,
            result_data: None,
            error_message: None,
        }
    }

    pub fn new_run(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<crate::models::agents::Agent>,
        run_payload: RunPayload,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            agent,
            SignalType::Run,
            Some(serde_json::to_value(run_payload).unwrap_or(Value::Null)),
        )
    }

    pub fn new_sync(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<crate::models::agents::Agent>,
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
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        agent: Option<crate::models::agents::Agent>,
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

    pub fn parse_run_payload(&self) -> Result<RunPayload> {
        match &self.initial_data {
            Some(data) if self.signal_type == SignalType::Run => {
                serde_json::from_value(data.clone())
                    .map_err(|e| anyhow!("Invalid run payload: {}", e))
            }
            _ => Err(anyhow!("Not a run signal or missing data")),
        }
    }

    pub fn parse_sync_payload(&self) -> Result<SyncPayload> {
        match &self.initial_data {
            Some(data) if self.signal_type == SignalType::Sync => {
                serde_json::from_value(data.clone())
                    .map_err(|e| anyhow!("Invalid sync payload: {}", e))
            }
            _ => Err(anyhow!("Not a sync signal or missing data")),
        }
    }

    pub fn parse_fyi_data(&self) -> Result<Value> {
        match &self.initial_data {
            Some(data) if self.signal_type == SignalType::Fyi => Ok(data.clone()),
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

    async fn execute_signal(&self) -> Result<crate::models::runtime_sessions::RuntimeSession> {
        match &self.agent {
            Some(agent) => {
                let result = agent
                    .run(self.initial_data.clone().unwrap_or(Value::Null))
                    .await?;
                Ok(result)
            }
            None => {
                let error_msg = match self.signal_type {
                    SignalType::Run => "Cannot process run signal with no associated agent",
                    SignalType::Sync => "Cannot process sync signal with no associated agent",
                    SignalType::Fyi => "FYI signal requires an agent to process",
                };
                Err(anyhow!(error_msg))
            }
        }
    }
}
