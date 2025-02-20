use crate::{IdFields, TimestampFields};
use crate::models::agents::Agent;
use crate::models::steps::Step;
use crate::RunningStatus;
use anyhow::Result;
use serde_json::Value;

pub struct RuntimeSession<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    requested_by_agent: &'a Agent<'a>,
    status: RunningStatus,
    initial_data: Value,
    latest_step_idx: i32,
    latest_result: Result<Value>,
    // Runtime fields
    steps: Vec<Step<'a>>,
}