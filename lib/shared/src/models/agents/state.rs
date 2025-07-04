use super::types::{Agent, AgentState};
use anyhow::{anyhow, Result};
use std::str::FromStr;

impl AgentState {
    pub fn as_str(&self) -> &str {
        match self {
            AgentState::Inactive => "inactive",
            AgentState::Stable => "stable",
            AgentState::Unstable => "unstable",
        }
    }
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentState::Inactive => "inactive",
            AgentState::Stable => "stable",
            AgentState::Unstable => "unstable",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AgentState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inactive" => Ok(AgentState::Inactive),
            "stable" => Ok(AgentState::Stable),
            "unstable" => Ok(AgentState::Unstable),
            _ => Err(format!("Unknown agent state: {}", s)),
        }
    }
}

impl Agent {
    pub fn state(&self) -> AgentState {
        let guard = self.agent_state.lock().unwrap();
        guard.clone()
    }

    pub fn set_state(&self, new_state: AgentState) {
        let mut guard = self.agent_state.lock().unwrap();
        *guard = new_state;
    }

    pub fn start(&self) -> Result<()> {
        let current_state = self.state();
        match current_state {
            AgentState::Inactive => {
                // Set new state to Stable
                self.set_state(AgentState::Stable);
                Ok(())
            }
            _ => Err(anyhow!("Can only start from Inactive state")),
        }
    }

    pub fn stop(&self) -> Result<()> {
        // Set to inactive
        let current_state = self.state();
        match current_state {
            AgentState::Stable | AgentState::Unstable => {
                self.set_state(AgentState::Inactive);
                Ok(())
            }
            _ => Err(anyhow!("Can only stop from a running state")),
        }
    }
}
