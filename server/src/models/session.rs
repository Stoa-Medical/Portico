/// The goal of a Session is to complete a series of steps and return the curr_result
/// It receives a pointer to steps and is expected to run those steps

use crate::models::steps::Step;
use serde_json::Value;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SessionError {
    #[error("Step {step_idx} failed: {message}")]
    StepFailed { step_idx: usize, message: String },
    
    #[error("Step bounds exceeded: requested {requested}, max {max}")]
    BoundsExceeded { requested: usize, max: usize },
    
    #[error("No input available for step {0}")]
    NoInput(usize),
}


pub struct Session<'steps> {
    /// Pointer to steps that should be executed
    pub steps: &'steps Vec<Step>,
    /// The starting input of the session
    pub input_data: Value,
    /// The end result of a session
    curr_result: Result<Option<Value>, SessionError>,
    /// Whether the session ran to completion or not
    pub completed: bool,
    /// Current index of the session step
    pub curr_idx: usize,
    /// The state of the data at each step (e.g. index i is state after running step i)
    res_state: Vec<Result<Option<Value>, SessionError>>
}

impl<'steps> Session<'steps> {
    pub fn new(steps: &'steps Vec<Step>, input_data: Value) -> Self {
        Self {
            steps,
            input_data,
            curr_result: Ok(None),
            completed: false,
            curr_idx: 0,
            res_state: vec![]
        }
    }

    /// Attempts to execute steps. If successful, retuns the number of steps run (stops at last step)
    ///   Returns the number of steps executed. Returns Err if cannot complete that number of steps
    pub async fn run_steps(&mut self, n_steps: usize, reset_on_err: bool) -> Result<Option<Value>, SessionError> {
        let target_idx = self.curr_idx + n_steps;
        if target_idx > self.steps.len() {
            return Err(SessionError::BoundsExceeded { 
                requested: target_idx, 
                max: self.steps.len() 
            });
        }
    
        // Store initial state in case we need to reset
        let initial_res = self.curr_result.clone();
        
        // Execute steps
        for idx in self.curr_idx..target_idx {
            let step = &self.steps[idx];
            let input = if idx == 0 {
                self.input_data.clone()
            } else if idx == self.curr_idx && !self.res_state.is_empty() {
                // If retrying a step, use the previous step's result
                self.res_state[idx-1].clone()?.ok_or(SessionError::NoInput(idx))?
            } else {
                self.curr_result.clone()?.ok_or(SessionError::NoInput(idx))?
            };
    

            match step.run(input, idx).await {
                Ok(result) => {
                    self.curr_result = Ok(result);
                    self.res_state.push(self.curr_result.clone());
                    self.curr_idx += 1;
                }
                Err(e) => {
                    if reset_on_err {
                        // Reset the result and the index
                        self.curr_result = initial_res;
                        self.curr_idx = idx; // Reset to the failed step
                        self.res_state.truncate(idx);
                    }
                    return Err(SessionError::StepFailed { 
                        step_idx: idx, 
                        message: e.to_string() 
                    });
                }
            }
        }

        if self.curr_idx >= self.steps.len() {
            self.completed = true;
        }

        Ok(self.curr_result.clone()?)
    }

    /// Attempts to run steps to completion. Starts from the current step. Returns whether all 
    pub async fn run_all(&mut self, reset_on_err: bool) -> Result<Option<Value>, SessionError> {
        let remaining_steps = self.steps.len() - self.curr_idx;
        self.run_steps(remaining_steps, reset_on_err).await
    }
}
