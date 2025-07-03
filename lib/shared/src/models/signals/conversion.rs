use super::types::{Signal, SignalType};
use crate::models::agents::Agent;
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
            "agent": self.agent.as_ref().map(|a| a.to_json()),
            "linked_rts_id": self.linked_rts.as_ref().and_then(|rts| rts.identifiers.local_id),
            "signal_type": self.signal_type.as_str(),
            "initial_data": self.initial_data,
            "result_data": self.result_data,
            "error_message": self.error_message
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
