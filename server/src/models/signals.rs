use crate::{IdFields, TimestampFields};
use crate::models::missions::Mission;
use crate::models::runtime_sessions::RuntimeSession;
use serde_json::Value;

pub struct Signal<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    mission: Option<&'a Mission<'a>>,
    signal_type: SignalType,
    signal_status: SignalStatus,
    initial_data: Option<Value>,
    response_data: Option<Value>,
    // Runtime fields
    runtime_session: Option<&'a RuntimeSession<'a>>,
}

// NOTE: Add others as we go
enum SignalType {
    MissionUserRequested,
    RuntimeSessionCompleted,
    AgentSaved,
    StepSaved
}

enum SignalStatus {
    InProgress,
    Completed,
    Cancelled
}