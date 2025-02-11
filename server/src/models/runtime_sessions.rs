/// The goal of a RuntimeSession is to complete a series of steps and return the curr_result
/// It receives a pointer to steps and is expected to run those steps

use crate::DataSource;
use crate::models::steps::Step;
use serde_json::Value;
use super::user_jobs::{Job, JobStatus};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("Step {step_idx} failed: {message}")]
    StepFailed { step_idx: usize, message: String },
    
    #[error("Step bounds exceeded: requested {requested}, max {max}")]
    BoundsExceeded { requested: usize, max: usize },
    
    #[error("No input available for step {0}")]
    NoInput(usize),
}

#[derive(Clone)]
pub struct RuntimeCheckpoint {
    current_step: usize,
    saved_result: Option<Value>,
}


pub struct RuntimeSession<'steps> {
    /// Pointer to steps that should be executed
    pub steps: &'steps mut Vec<Step>,
    /// The starting input of the RuntimeSession
    pub input_data: DataSource,
    /// The end result of a RuntimeSession
    curr_result: Result<Option<Value>, RuntimeError>,
    /// Optional reference to parent job for status updates
    parent_job: Option<&'steps mut Job>,
    /// Whether the RuntimeSession ran to completion or not
    completed: bool,
    /// Current index of the RuntimeSession step
    curr_idx: usize,
    /// The state of the data at each step (e.g. index i is state after running step i)
    res_state: Vec<Result<Option<Value>, RuntimeError>>
}

impl<'steps> RuntimeSession<'steps> {
    pub fn new(steps: &'steps mut Vec<Step>, input_data: DataSource, parent_job: Option<&'steps mut Job>) -> Self {
        Self {
            steps,
            input_data,
            curr_result: Ok(None),
            parent_job,
            completed: false,
            curr_idx: 0,
            res_state: vec![]
        }
    }

    pub fn save_checkpoint(&self) -> Option<RuntimeCheckpoint> {
        Some(RuntimeCheckpoint {
            current_step: self.curr_idx,
            saved_result: self.curr_result.clone().expect("Expected non-Err checkpoint to save"),
        })
    }

    pub fn resume_from_checkpoint(
        steps: &'steps mut Vec<Step>, 
        input: DataSource, 
        checkpoint: RuntimeCheckpoint,
        parent_job: Option<&'steps mut Job>
    ) -> Self {
        let mut rts = Self::new(steps, input, parent_job);
        rts.curr_idx = checkpoint.current_step;
        rts.curr_result = Ok(checkpoint.saved_result);
        rts
    }

    /// Runs a single step and returns its result
    async fn run_one_step(&mut self, idx: usize) -> Result<Option<Value>, RuntimeError> {
        let step = &mut self.steps[idx];

        // Update job status if we have a parent job
        if let Some(job) = &mut self.parent_job {
            job.update_status(JobStatus::Running, None);
        }

        let input = if idx == 0 {
            self.input_data.extract().await.map_err(|e| RuntimeError::StepFailed { 
                step_idx: idx, 
                message: e.to_string() 
            })?
        } else if idx == self.curr_idx && !self.res_state.is_empty() {
            // If retrying a step, use the previous step's result
            self.res_state[idx-1].clone()?.ok_or(RuntimeError::NoInput(idx))?
        } else {
            self.curr_result.clone()?.ok_or(RuntimeError::NoInput(idx))?
        };

        match step.run(DataSource::Json(input), idx).await {
            Ok(result) => {
                self.curr_result = Ok(result.clone());
                self.res_state.push(self.curr_result.clone());
                self.curr_idx += 1;
                Ok(result)
            }
            Err(e) => {
                if let Some(job) = &mut self.parent_job {
                    job.update_status(
                        JobStatus::Failed, 
                        Some(format!("Step {} failed: {}", idx, e))
                    );
                }
                Err(RuntimeError::StepFailed { 
                    step_idx: idx, 
                    message: e.to_string() 
                })
            }
        }
    }
    /// Rolls back the RuntimeSession state to before the given index
    fn rollback_to(&mut self, idx: usize, initial_res: Result<Option<Value>, RuntimeError>) {
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

    pub async fn run_n_steps(&mut self, n_steps: usize, reset_on_err: bool) -> Result<Option<Value>, RuntimeError> {
        let target_idx = self.curr_idx + n_steps;
        if target_idx > self.steps.len() {
            return Err(RuntimeError::BoundsExceeded { 
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
            // Update job completion status if we have a parent
            if let Some(job) = &mut self.parent_job {
                match &self.curr_result {
                    Ok(Some(result)) => {
                        job.update_status_from_result(&Ok(result.clone()));
                    }
                    Ok(None) => {
                        job.update_status_from_result(&Ok(Value::Null));
                    }
                    Err(e) => {
                        job.update_status(JobStatus::Failed, Some(e.to_string()));
                    }
                }
            }
        }

        Ok(self.curr_result.clone()?)
    }

    pub async fn run_all(&mut self, reset_on_err: bool) -> Result<Option<Value>, RuntimeError> {
        let remaining_steps = self.steps.len() - self.curr_idx;
        self.run_n_steps(remaining_steps, reset_on_err).await
    }
}