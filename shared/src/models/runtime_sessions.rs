use crate::{IdFields, RunningStatus, TimestampFields};
use crate::Step;
use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(sqlx::FromRow)]
pub struct RuntimeSession {
    #[sqlx(flatten)]
    identifiers: IdFields,
    #[sqlx(flatten)]
    timestamps: TimestampFields,
    steps: Vec<Step>,
    status: RunningStatus,
    #[sqlx(json)]
    source_data: Value,
    last_step_idx: Option<usize>,
    last_successful_result: Option<Value>,
}

impl RuntimeSession {
    pub fn new(
        source_data: Value,
        steps: Vec<Step>,
    ) -> Self {
        Self {
            identifiers: IdFields::new(),
            timestamps: TimestampFields::new(),
            steps,
            status: RunningStatus::Waiting,
            source_data,
            last_step_idx: None,
            last_successful_result: None,
        }
    }

    pub async fn start(&mut self) -> Result<Value> {
        // Set status to Running
        self.status = RunningStatus::Running;

        // Execute each step in order, passing the result of each step to the next
        let mut current_value = self.source_data.clone();

        // Track step execution
        for (idx, step) in self.steps.iter().enumerate() {
            // Update latest step index before execution
            self.last_step_idx = Some(idx);

            match step.run(current_value, idx).await {
                Ok(value) => {
                    // Update current value for next step
                    current_value = value.clone();

                    // Store the intermediate result
                    self.last_successful_result = Some(value);
                },
                Err(e) => {
                    // Update status to cancelled
                    self.status = RunningStatus::Cancelled;
                    return Err(anyhow!("Step execution failed: {}", e));
                }
            }
        }

        // All steps completed successfully
        self.status = RunningStatus::Completed;

        // Store the final result and return it
        self.last_successful_result = Some(current_value.clone());
        Ok(current_value)
    }
}
