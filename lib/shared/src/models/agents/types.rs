use crate::models::steps::Step;
use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// An Agent represents a component that listens for and reacts to Signals in the system.
/// Agents are responsible for monitoring specific Signal types and acting on them
/// NOTE: Agents are created in the UI, and Supabase is the source-of-truth for their state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub description: String,
    pub agent_state: Mutex<AgentState>,
    pub steps: Vec<Step>,
}

/// Different states for Agent to be in. State diagram:
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
#[sqlx(type_name = "agent_state", rename_all = "lowercase")]
pub enum AgentState {
    #[default]
    Inactive,
    Stable,
    Unstable,
}

impl Agent {
    pub fn new(
        identifiers: IdFields,
        timestamps: TimestampFields,
        description: String,
        steps: Vec<Step>,
    ) -> Self {
        // Start all agents in an inactive state
        Self {
            identifiers,
            timestamps,
            description,
            agent_state: Mutex::new(AgentState::Inactive),
            steps,
        }
    }
}
