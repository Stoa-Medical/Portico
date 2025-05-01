use super::types::RuntimeSession;
use crate::{PythonRuntime, RunningStatus};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::time::Instant;

impl RuntimeSession {
    /// Start executing the session with an optional Python runtime
    /// This is a unified method that works with or without a runtime.
    /// If no runtime is provided, only Prompt steps can be executed.
    pub async fn unified_start(&mut self, runtime: Option<&PythonRuntime>) -> Result<Value> {
        // Set status to Running
        self.status = RunningStatus::Running;

        // Check if a runtime is required but not provided
        if runtime.is_none() && self.steps.iter().any(|step| step.is_python_step()) {
            self.status = RunningStatus::Cancelled;
            return Err(anyhow!(
                "Python steps require a runtime but none was provided"
            ));
        }

        // Initialize timing fields
        self.step_execution_times = Vec::with_capacity(self.steps.len());
        self.total_execution_time = std::time::Duration::ZERO;

        // Initialize step_results with None values for each step
        self.step_results = vec![None; self.steps.len()];

        let start_time = Instant::now();

        // Execute each step in order, passing the result of each step to the next
        let mut current_value = self.source_data.clone();

        // Track step execution
        for (idx, step) in self.steps.iter().enumerate() {
            // Update latest step index before execution
            self.last_step_idx = Some(idx as i32);

            // Track this step's execution time
            let step_start = Instant::now();

            // Use step.run which will handle the runtime appropriately for each step type
            let result = step.run(current_value.clone(), idx, runtime).await;

            match result {
                Ok(value) => {
                    // Record execution time for this step
                    let step_duration = step_start.elapsed();
                    self.step_execution_times.push(step_duration);

                    // Update current value for next step
                    current_value = value.clone();

                    // Store the intermediate result
                    self.last_successful_result = Some(value.clone());

                    // Store the step result
                    self.step_results[idx] = Some(value);
                }
                Err(e) => {
                    // Still record execution time for the failed step
                    let step_duration = step_start.elapsed();
                    self.step_execution_times.push(step_duration);

                    // Calculate total time before returning
                    self.total_execution_time = start_time.elapsed();

                    // Update status to cancelled
                    self.status = RunningStatus::Cancelled;
                    return Err(anyhow!("Step execution failed: {}", e));
                }
            }
        }

        // All steps completed successfully
        self.status = RunningStatus::Completed;

        // Record total execution time
        self.total_execution_time = start_time.elapsed();

        // Store the final result and return it
        self.last_successful_result = Some(current_value.clone());
        Ok(current_value)
    }

    /// Start executing the session with a Python runtime
    pub async fn start_with_runtime(&mut self, runtime: &PythonRuntime) -> Result<Value> {
        self.unified_start(Some(runtime)).await
    }

    /// Start the session without a Python runtime.
    /// This method can only execute sessions that have no Python steps.
    /// Use start_with_runtime for sessions with Python steps.
    pub async fn start(&mut self) -> Result<Value> {
        // Check if this session has any Python steps
        if self.steps.iter().any(|step| step.is_python_step()) {
            return Err(anyhow!("This session contains Python steps which require a runtime. Use start_with_runtime() instead."));
        }

        self.unified_start(None).await
    }
}
