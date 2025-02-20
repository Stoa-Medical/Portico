use crate::{IdFields, TimestampFields};
use crate::models::agents::Agent;
use crate::RunningStatus;
use serde_json::Value;
use crate::models::runtime_sessions::RuntimeSession;
pub struct Mission<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    user_requested_uuid: String,
    requested_agent: &'a Agent<'a>,
    mission_status: RunningStatus,
    description: String,
    initial_data: Value,
    // Runtime fields
    runtime_session: Option<&'a RuntimeSession<'a>>,
}

