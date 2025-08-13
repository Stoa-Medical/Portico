use super::types::{Workflow, WorkflowState};
use anyhow::{anyhow, Result};
use std::str::FromStr;

impl WorkflowState {
    pub fn as_str(&self) -> &str {
        match self {
            WorkflowState::Inactive => "inactive",
            WorkflowState::Stable => "stable",
            WorkflowState::Unstable => "unstable",
        }
    }
}

impl std::fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            WorkflowState::Inactive => "inactive",
            WorkflowState::Stable => "stable",
            WorkflowState::Unstable => "unstable",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for WorkflowState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inactive" => Ok(WorkflowState::Inactive),
            "stable" => Ok(WorkflowState::Stable),
            "unstable" => Ok(WorkflowState::Unstable),
            _ => Err(format!("Unknown workflow state: {}", s)),
        }
    }
}

impl Workflow {
    pub fn state(&self) -> WorkflowState {
        let guard = self.workflow_state.lock().unwrap();
        guard.clone()
    }

    pub fn set_state(&self, new_state: WorkflowState) {
        let mut guard = self.workflow_state.lock().unwrap();
        *guard = new_state;
    }

    pub fn start(&self) -> Result<()> {
        let current_state = self.state();
        match current_state {
            WorkflowState::Inactive => {
                // Set new state to Stable
                self.set_state(WorkflowState::Stable);
                Ok(())
            }
            _ => Err(anyhow!("Can only start from Inactive state")),
        }
    }

    pub fn stop(&self) -> Result<()> {
        // Set to inactive
        let current_state = self.state();
        match current_state {
            WorkflowState::Stable | WorkflowState::Unstable => {
                self.set_state(WorkflowState::Inactive);
                Ok(())
            }
            _ => Err(anyhow!("Can only stop from a running state")),
        }
    }
}
