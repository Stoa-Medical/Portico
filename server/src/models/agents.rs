/// Class for defining Agents
///   An Agent is a listener that can receive + react to data
///   An Agent can take Steps to achieve its goal (based on heuristics)
///     Steps are python code, and the scaffolding is defined in `steps.rs`

use super::steps::Step;
use crate::{CanAct, CanReact, DataSource};

use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;

/// Agents are units of action
struct Agent {
    /// The UUID identifier
    id: Uuid,
    /// What does this agent do (as described by a human)?
    description: String,
    /// The state of the agent
    state: AgentState,
    /// The steps the agent should take
    steps: Vec<Step>,
    /// The allowed error rate before flagging as AgentState::Unstable
    accepted_err_rate: f32,
    /// Outgoing destionations (if applicable)
    outgoing_dest_list: Option<Vec<DataSource>>,
}


enum AgentState {
    Waiting,
    Active,
    Unstable,
    Stopping,
    Inactive
}


impl Agent {
    fn new(description: String, accepted_err_rate: f32, steps: Vec<Step>, outgoing_dest_list: Option<Vec<DataSource>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            description,
            state: AgentState::Waiting,
            steps,
            accepted_err_rate,
            outgoing_dest_list
        }
    }
    
}
 
