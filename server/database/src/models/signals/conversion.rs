use super::types::{Signal, SignalType};
use crate::{IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::str::FromStr;

impl JsonLike for Signal {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
            "user_requested_uuid": self.user_requested_uuid,
            "workflow_id": self.workflow_id,
            "initiator_agent_id": self.initiator_agent_id,
            "rts_id": self.rts_id,
            "signal_type": self.signal_type.as_str(),
            "initial_data": self.initial_data,
            "response_data": self.response_data,
            "error_message": self.error_message,
            "source": self.source,
            "idempotency_key": self.idempotency_key,
            "source_metadata": self.source_metadata,
            "leased_at": self.leased_at,
            "lease_expires_at": self.lease_expires_at
        })
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

        let workflow_id = obj.get("workflow_id").and_then(|v| v.as_i64().map(|i| i as i32));
        let initiator_agent_id = obj.get("initiator_agent_id").and_then(|v| v.as_i64().map(|i| i as i32));
        let rts_id = obj.get("rts_id").and_then(|v| v.as_i64());

        let initial_data = obj.get("initial_data").cloned();
        let response_data = obj.get("response_data").cloned();
        let error_message = obj
            .get("error_message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Build the Signal
        Ok(Signal {
            identifiers: IdFields {
                local_id,
                global_uuid,
            },
            timestamps: TimestampFields::new(),
            user_requested_uuid,
            workflow_id,
            initiator_agent_id,
            rts_id,
            signal_type,
            initial_data,
            response_data,
            error_message,
            source: obj.get("source").and_then(|v| v.as_str()).map(|s| s.to_string()),
            idempotency_key: obj.get("idempotency_key").and_then(|v| v.as_str()).map(|s| s.to_string()),
            source_metadata: obj.get("source_metadata").cloned(),
            leased_at: obj.get("leased_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            lease_expires_at: obj.get("lease_expires_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }
}
