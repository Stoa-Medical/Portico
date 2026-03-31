use super::types::{RunPayload, Signal, SignalType, SyncPayload};
use serde_json::Value;

impl Signal {
    pub fn new(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        workflow_id: Option<i32>,
        initiator_agent_id: Option<i32>,
        signal_type: SignalType,
        initial_data: Option<Value>,
    ) -> Self {
        Self {
            identifiers,
            timestamps: crate::TimestampFields::new(),
            user_requested_uuid,
            workflow_id,
            initiator_agent_id,
            rts_id: None,
            signal_type,
            initial_data,
            response_data: None,
            error_message: None,
            source: None,
            idempotency_key: None,
            source_metadata: None,
            leased_at: None,
            lease_expires_at: None,
        }
    }

    pub fn new_run(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        workflow_id: Option<i32>,
        initiator_agent_id: Option<i32>,
        run_payload: RunPayload,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            workflow_id,
            initiator_agent_id,
            SignalType::Run,
            Some(serde_json::to_value(run_payload).unwrap_or_default()),
        )
    }

    pub fn new_sync(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        workflow_id: Option<i32>,
        initiator_agent_id: Option<i32>,
        sync_payload: SyncPayload,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            workflow_id,
            initiator_agent_id,
            SignalType::Sync,
            Some(serde_json::to_value(sync_payload).unwrap_or_default()),
        )
    }

    pub fn new_fyi(
        identifiers: crate::IdFields<i64>,
        user_requested_uuid: String,
        workflow_id: Option<i32>,
        initiator_agent_id: Option<i32>,
        fyi_data: Value,
    ) -> Self {
        Self::new(
            identifiers,
            user_requested_uuid,
            workflow_id,
            initiator_agent_id,
            SignalType::Fyi,
            Some(fyi_data),
        )
    }

    /// Get the workflow ID associated with this signal
    pub fn get_workflow_id(&self) -> Option<i32> {
        self.workflow_id
    }

    /// Get the initiator agent ID for this signal
    pub fn get_initiator_agent_id(&self) -> Option<i32> {
        self.initiator_agent_id
    }

    /// Check if this signal has been processed (has response data or error)
    pub fn is_processed(&self) -> bool {
        self.response_data.is_some() || self.error_message.is_some()
    }

    /// Set response data for the signal
    pub fn set_response(&mut self, response_data: Value) {
        self.response_data = Some(response_data);
        self.timestamps.updated = chrono::Utc::now();
    }

    /// Set error message for the signal
    pub fn set_error(&mut self, error_message: String) {
        self.error_message = Some(error_message);
        self.timestamps.updated = chrono::Utc::now();
    }

    /// Link this signal to a runtime session
    pub fn link_runtime_session(&mut self, rts_id: i64) {
        self.rts_id = Some(rts_id);
        self.timestamps.updated = chrono::Utc::now();
    }
}
