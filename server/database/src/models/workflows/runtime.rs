use super::types::Workflow;
use crate::models::workflows::WorkflowState;
use crate::models::runtime_sessions::RuntimeSession;
use anyhow::{anyhow, Result};
use serde_json::Value;

impl Workflow {
    /// Process data with this workflow using provided steps.
    /// Note: Step execution has been moved to the engine's step dispatch system.
    /// This method creates a RuntimeSession but does not execute steps directly.
    pub async fn run_with_steps(&self, source: Value, steps: Vec<crate::models::steps::Step>) -> Result<RuntimeSession> {
        // Check if state is Inactive. If so, return error
        if *self.state() == WorkflowState::Inactive {
            return Err(anyhow!("Cannot run workflow in Inactive state"));
        }

        // Create a new RuntimeSession with the provided steps and local_id
        let session = RuntimeSession::new(source, steps, self.identifiers.local_id);

        // Step execution is now handled by the engine's step dispatch system.
        // Return the session for the engine to execute.
        Ok(session)
    }
}
