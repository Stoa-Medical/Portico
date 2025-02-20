use crate::{IdFields, TimestampFields};
use crate::models::agents::Agent;

pub struct Step<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    agent_owner: &'a Agent<'a>,
    step_type: StepType,
    step_content: String,
    run_count: i32,
    success_count: i32,
    // Runtime fields
}

enum StepType {
    Python,
    Prompt
}
