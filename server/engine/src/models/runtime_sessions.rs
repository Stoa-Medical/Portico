use crate::Step;
use crate::{IdFields, RunningStatus, TimestampFields};
use anyhow::{anyhow, Result};
use serde_json::Value;

#[derive(Clone)]
pub struct RuntimeCheckpoint {
    current_step: usize,
    saved_result: Option<Value>,
}

// Define events that RuntimeSession can emit
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    Started,
    StepCompleted { step_idx: usize, result: Value },
    Failed { step_idx: usize, error: String },
    Completed { result: Value },
}

// Define a SessionResult to return to the Agent
#[derive(Debug)]
pub struct SessionResult {
    pub completed: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub error_rate: f32,
}

pub struct RuntimeSession<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    agent_id: String,
    status: RunningStatus,
    initial_data: Value,
    latest_step_idx: i32,
    latest_result: Result<Value>,

    // Runtime fields
    steps: &'a Vec<Step<'a>>,
    curr_result: Result<Option<Value>>,
    completed: bool,
    curr_idx: usize,
    res_state: Vec<Result<Option<Value>>>,
    success_count: usize,
    total_count: usize,
}

impl<'a> RuntimeSession<'a> {
    pub fn new(
        identifiers: IdFields,
        agent_id: String,
        steps: &'a Vec<Step<'a>>,
        initial_data: Value,
    ) -> Self {
        Self {
            // Database fields
            identifiers,
            timestamps: TimestampFields::new(),
            agent_id,
            status: RunningStatus::Pending,
            initial_data,
            latest_step_idx: -1,
            latest_result: Ok(Value::Null),

            // Runtime fields
            steps,
            curr_result: Ok(None),
            completed: false,
            curr_idx: 0,
            res_state: vec![],
            success_count: 0,
            total_count: 0,
        }
    }

    pub fn save_checkpoint(&self) -> Option<RuntimeCheckpoint> {
        let saved_result = match &self.curr_result {
            Ok(val) => val.clone(),
            Err(_) => return None,
        };

        Some(RuntimeCheckpoint {
            current_step: self.curr_idx,
            saved_result,
        })
    }

    pub fn resume_from_checkpoint(&mut self, checkpoint: RuntimeCheckpoint) {
        self.curr_idx = checkpoint.current_step;
        self.curr_result = Ok(checkpoint.saved_result);
    }

    /// Runs a single step and returns its result
    async fn run_one_step(&mut self, idx: usize) -> Result<Option<Value>> {
        let step = &self.steps[idx];
        self.total_count += 1;

        // Update session status
        self.status = RunningStatus::InProgress;

        let input = if idx == 0 {
            self.initial_data.clone()
        } else if idx == self.curr_idx && !self.res_state.is_empty() {
            match &self.res_state[idx - 1] {
                Ok(Some(val)) => val.clone(),
                _ => return Err(anyhow!("No input available for step {}", idx)),
            }
        } else {
            match &self.curr_result {
                Ok(Some(val)) => val.clone(),
                _ => return Err(anyhow!("No input available for step {}", idx)),
            }
        };

        match step.run(input, idx).await {
            Ok(result) => {
                self.curr_result = Ok(result.clone());
                self.res_state.push(Ok(result.clone()));
                self.curr_idx += 1;
                self.latest_step_idx = idx as i32;
                self.success_count += 1;

                // Update latest_result for database
                if let Some(val) = &result {
                    self.latest_result = Ok(val.clone());
                }

                Ok(result)
            }
            Err(e) => {
                // Update session status
                self.status = RunningStatus::Failed;

                // Create error without storing the message string
                let error = anyhow!("Step {} failed: {}", idx, e);

                // Store the error in latest_result for database
                self.latest_result = Err(anyhow!("Step {} failed: {}", idx, e));

                Err(error)
            }
        }
    }

    /// Rolls back the RuntimeSession state to before the given index
    fn rollback_to(&mut self, idx: usize, initial_res: Result<Option<Value>>) {
        self.curr_result = match &initial_res {
            Ok(val) => Ok(val.clone()),
            Err(e) => Err(anyhow!("{}", e)),
        };
        self.curr_idx = idx;
        self.res_state.truncate(idx);
        self.latest_step_idx = (idx as i32) - 1;
    }

    /// Returns whether all steps have been completed
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Returns the current step index
    pub fn current_step(&self) -> usize {
        self.curr_idx
    }

    /// Returns the current error rate
    pub fn get_error_rate(&self) -> f32 {
        if self.total_count == 0 {
            return 0.0;
        }
        (self.total_count - self.success_count) as f32 / self.total_count as f32
    }

    pub async fn run_n_steps(
        &mut self,
        n_steps: usize,
        reset_on_err: bool,
    ) -> Result<Option<Value>> {
        let target_idx = self.curr_idx + n_steps;
        if target_idx > self.steps.len() {
            return Err(anyhow!(
                "Step bounds exceeded: requested {}, max {}",
                target_idx,
                self.steps.len()
            ));
        }

        let initial_res = match &self.curr_result {
            Ok(val) => Ok(val.clone()),
            Err(e) => Err(anyhow!("{}", e)),
        };

        for idx in self.curr_idx..target_idx {
            match self.run_one_step(idx).await {
                Ok(_) => continue,
                Err(e) => {
                    if reset_on_err {
                        self.rollback_to(idx, initial_res);
                    }
                    return Err(e);
                }
            }
        }

        if self.curr_idx >= self.steps.len() {
            self.completed = true;
            self.status = RunningStatus::Completed;
        }

        match &self.curr_result {
            Ok(val) => Ok(val.clone()),
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    pub async fn run_all(&mut self, reset_on_err: bool) -> Result<SessionResult> {
        let remaining_steps = self.steps.len() - self.curr_idx;
        let result = self.run_n_steps(remaining_steps, reset_on_err).await;

        // Create a SessionResult to return to the Agent
        let session_result = match &result {
            Ok(value) => SessionResult {
                completed: self.completed,
                result: value.clone(),
                error: None,
                error_rate: self.get_error_rate(),
            },
            Err(e) => SessionResult {
                completed: false,
                result: None,
                error: Some(e.to_string()),
                error_rate: self.get_error_rate(),
            },
        };

        // Return the result or propagate the error
        match result {
            Ok(_) => Ok(session_result),
            Err(_) => Ok(session_result), // We're returning the SessionResult even on error
        }
    }

    // Methods to sync runtime state with database fields
    pub fn sync_to_db(&mut self) {
        // Update database fields based on runtime state
        self.latest_step_idx = self.curr_idx as i32 - 1;

        // Update status based on runtime state
        if self.completed {
            self.status = RunningStatus::Completed;
        } else if self.status == RunningStatus::Pending && self.curr_idx > 0 {
            self.status = RunningStatus::InProgress;
        }

        // Update latest_result based on curr_result
        match &self.curr_result {
            Ok(Some(val)) => self.latest_result = Ok(val.clone()),
            Ok(None) => self.latest_result = Ok(Value::Null),
            Err(e) => self.latest_result = Err(anyhow!("{}", e)),
        }
    }

    // Getters for database fields
    pub fn get_status(&self) -> &RunningStatus {
        &self.status
    }

    pub fn get_latest_step_idx(&self) -> i32 {
        self.latest_step_idx
    }

    pub fn get_latest_result(&self) -> &Result<Value> {
        &self.latest_result
    }
}
