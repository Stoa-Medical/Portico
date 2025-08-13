use crate::models::steps::Step;
use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// A Workflow represents a sequence of configurable steps that can be executed.
/// Workflows define the processing logic and can be dynamically created by Agents.
/// NOTE: Workflows are created in the UI, and Supabase is the source-of-truth for their state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub description: String,
    pub workflow_state: Mutex<WorkflowState>,
    pub steps: Vec<Step>,
}

/// Different states for Workflow to be in. State diagram:
/// ```plain
///          (start)    ┌──────────┐
///  Inactive ───────► Stable ──┐  │
///      ▲              ▲       │  │
///      │              │ (err) │  │
///      │          Unstable ◄──┘  │
///      │   (stop)     │          │
///      └──────────────┘◄─────────┘
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default, sqlx::Type)]
#[sqlx(type_name = "workflow_state", rename_all = "lowercase")]
pub enum WorkflowState {
    #[default]
    Inactive,
    Stable,
    Unstable,
}

impl Workflow {
    pub fn new(
        identifiers: IdFields,
        timestamps: TimestampFields,
        description: String,
        steps: Vec<Step>,
    ) -> Self {
        // Start all workflows in an inactive state
        Self {
            identifiers,
            timestamps,
            description,
            workflow_state: Mutex::new(WorkflowState::Inactive),
            steps,
        }
    }
}
