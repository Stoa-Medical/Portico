use crate::{IdFields, RunningStatus, Step, TimestampFields};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub struct RuntimeSession {
    pub identifiers: IdFields<i64>,
    pub timestamps: TimestampFields,
    pub steps: Vec<Step>,
    pub status: RunningStatus,
    pub source_data: Value,
    pub last_step_idx: Option<i32>,
    pub last_successful_result: Option<Value>,
    pub step_execution_times: Vec<Duration>, // Stores duration for each step
    pub total_execution_time: Duration,      // Stores total runtime
    pub requested_by_agent_id: Option<i32>, // The local ID of the agent that requested this session
    pub step_results: Vec<Option<Value>>,   // Stores result for each step (None if failed)
}

impl RuntimeSession {
    pub fn new(source_data: Value, steps: Vec<Step>, requested_by_agent_id: Option<i32>) -> Self {
        Self {
            identifiers: IdFields::new(),
            timestamps: TimestampFields::new(),
            steps,
            status: RunningStatus::Waiting,
            source_data,
            last_step_idx: None,
            last_successful_result: None,
            step_execution_times: Vec::new(),
            total_execution_time: Duration::ZERO,
            requested_by_agent_id,
            step_results: Vec::new(),
        }
    }
}
