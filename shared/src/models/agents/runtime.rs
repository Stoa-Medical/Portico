use super::types::Agent;
use crate::models::agents::AgentState;
use crate::models::runtime_sessions::RuntimeSession;
use crate::PythonRuntime;
use anyhow::{anyhow, Result};
use serde_json::Value;

impl Agent {
    /// Create a Python runtime for this agent
    pub fn create_python_runtime(&self) -> Result<PythonRuntime> {
        let mut runtime = PythonRuntime::new(&self.identifiers.global_uuid)?;

        // Add all Python steps
        for step in &self.steps {
            if step.is_python_step() {
                runtime.add_step(step)?;
            }
        }

        Ok(runtime)
    }

    /// Process data with this agent using an immutable reference
    pub async fn run(&self, source: Value) -> Result<RuntimeSession> {
        // Check if state is Inactive. If so, return error
        if self.state() == AgentState::Inactive {
            return Err(anyhow!("Cannot run agent in Inactive state"));
        }

        // Create a Python runtime for this agent
        let runtime = self.create_python_runtime()?;

        // Create a new RuntimeSession with the agent's steps and local_id
        let mut session =
            RuntimeSession::new(source, self.steps.clone(), self.identifiers.local_id);

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
