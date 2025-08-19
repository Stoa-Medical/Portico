use super::types::Workflow;
use crate::models::workflows::WorkflowState;
use crate::models::runtime_sessions::RuntimeSession;
use crate::PythonRuntime;
use anyhow::{anyhow, Result};
use serde_json::Value;

impl Workflow {
    /// Create a Python runtime for this workflow
    /// Note: Steps need to be provided separately since workflow only stores step_ids
    pub fn create_python_runtime_with_steps(&self, steps: &[crate::models::steps::Step]) -> Result<PythonRuntime> {
        let mut runtime = PythonRuntime::new(&self.identifiers.global_uuid)?;

        // Add all Python steps
        for step in steps {
            if step.is_python_step() {
                runtime.add_step(step)?;
            }
        }

        Ok(runtime)
    }

    /// Process data with this workflow using provided steps
    /// Note: Steps need to be provided separately since workflow only stores step_ids
    pub async fn run_with_steps(&self, source: Value, steps: Vec<crate::models::steps::Step>) -> Result<RuntimeSession> {
        // Check if state is Inactive. If so, return error
        if *self.state() == WorkflowState::Inactive {
            return Err(anyhow!("Cannot run workflow in Inactive state"));
        }

        // Create a Python runtime for this workflow
        let runtime = self.create_python_runtime_with_steps(&steps)?;

        // Create a new RuntimeSession with the provided steps and local_id
        let mut session =
            RuntimeSession::new(source, steps, self.identifiers.local_id);

        // Start the RuntimeSession with the Python runtime
        let result = session.start_with_runtime(&runtime).await;

        // If there was an error, propagate it
        if let Err(e) = result {
            return Err(e);
        }

        // Return final session
        Ok(session)
    }
}
