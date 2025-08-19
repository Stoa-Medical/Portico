use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};

/// A Workflow represents a sequence of configurable steps that can be executed.
/// Workflows define the processing logic and can be dynamically created by Agents.
/// NOTE: Workflows are created in the UI, and Supabase is the source-of-truth for their state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub name: Option<String>,
    pub workflow_type: Option<String>,
    pub description: Option<String>,
    pub workflow_state: WorkflowState,
    pub workflow_name: Option<String>,
    pub step_ids: Option<Vec<i32>>,
    pub created_by_agent_id: Option<i32>,
    pub version: String,
    pub is_ephemeral: bool,
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
        name: Option<String>,
        description: Option<String>,
    ) -> Self {
        // Start all workflows in an inactive state
        Self {
            identifiers,
            timestamps,
            workflow_name: name.clone(),
            name,
            workflow_type: None,
            description,
            workflow_state: WorkflowState::Inactive,
            step_ids: None,
            created_by_agent_id: None,
            version: "v1".to_string(),
            is_ephemeral: false,
        }
    }

    /// Get the current state of the workflow
    pub fn state(&self) -> &WorkflowState {
        &self.workflow_state
    }

    /// Check if the workflow is ephemeral (should be garbage collected)
    pub fn is_ephemeral(&self) -> bool {
        self.is_ephemeral
    }

    /// Get the agent that created this workflow
    pub fn creator_agent_id(&self) -> Option<i32> {
        self.created_by_agent_id
    }
}
