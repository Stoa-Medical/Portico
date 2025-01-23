/// The goal of a Session is to complete a series of steps and return the curr_result
/// It receives a pointer to steps and is expected to run those steps

use crate::DataSource;
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
    pub input_data: DataSource,
    /// The end result of a session
    curr_result: Result<Option<Value>, SessionError>,
    /// Whether the session ran to completion or not
    completed: bool,
    /// Current index of the session step
    curr_idx: usize,
    /// The state of the data at each step (e.g. index i is state after running step i)
    res_state: Vec<Result<Option<Value>, SessionError>>
}

impl<'steps> Session<'steps> {
    pub fn new(steps: &'steps Vec<Step>, input_data: DataSource) -> Self {
        Self {
            steps,
            input_data,
            curr_result: Ok(None),
            completed: false,
            curr_idx: 0,
            res_state: vec![]
        }
    }

    /// Runs a single step and returns its result
    async fn run_one_step(&mut self, idx: usize) -> Result<Option<Value>, SessionError> {
        let step = &self.steps[idx];
        let input = if idx == 0 {
            self.input_data.extract().await.map_err(|e| SessionError::StepFailed { 
                step_idx: idx, 
                message: e.to_string() 
            })?
        } else if idx == self.curr_idx && !self.res_state.is_empty() {
            // If retrying a step, use the previous step's result
            self.res_state[idx-1].clone()?.ok_or(SessionError::NoInput(idx))?
        } else {
            self.curr_result.clone()?.ok_or(SessionError::NoInput(idx))?
        };

        match step.run(DataSource::Json(input), idx).await {
            Ok(result) => {
                self.curr_result = Ok(result.clone());
                self.res_state.push(self.curr_result.clone());
                self.curr_idx += 1;
                Ok(result)
            }
            Err(e) => Err(SessionError::StepFailed { 
                step_idx: idx, 
                message: e.to_string() 
            })
        }
    }
    /// Rolls back the session state to before the given index
    fn rollback_to(&mut self, idx: usize, initial_res: Result<Option<Value>, SessionError>) {
        self.curr_result = initial_res;
        self.curr_idx = idx;
        self.res_state.truncate(idx);
    }

    /// Returns whether all steps have been completed
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Returns the current step index
    pub fn current_step(&self) -> usize {
        self.curr_idx
    }

    pub async fn run_n_steps(&mut self, n_steps: usize, reset_on_err: bool) -> Result<Option<Value>, SessionError> {
        let target_idx = self.curr_idx + n_steps;
        if target_idx > self.steps.len() {
            return Err(SessionError::BoundsExceeded { 
                requested: target_idx, 
                max: self.steps.len() 
            });
        }
    
        let initial_res = self.curr_result.clone();
        
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
        }

        Ok(self.curr_result.clone()?)
    }

    pub async fn run_all(&mut self, reset_on_err: bool) -> Result<Option<Value>, SessionError> {
        let remaining_steps = self.steps.len() - self.curr_idx;
        self.run_n_steps(remaining_steps, reset_on_err).await
    }
}