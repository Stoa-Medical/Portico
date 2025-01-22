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
    steps: &'steps Vec<Step>,
    /// The starting input of the session
    input_data: Value,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::steps::StepAction;
    use serde_json::json;

    #[tokio::test]
    async fn test_session_basic() {
        let steps = vec![
            Step::new(
                "step1".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            ),
            Step::new(
                "step2".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data + 1
"#.to_string())
            ),
        ];

        let input = json!({"value": 21});
        let mut session = Session::new(&steps, input);
        
        let result = session.run_all(true).await.unwrap();
        assert!(result.is_some());
        assert!(session.completed);
        assert_eq!(steps[0].get_run_count(), 1);
        assert_eq!(steps[1].get_run_count(), 1);
    }

    #[tokio::test]
    async fn test_session_step_by_step() {
        let steps = vec![
            Step::new(
                "step1".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            ),
            Step::new(
                "step2".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data + 1
"#.to_string())
            ),
        ];

        let input = json!({"value": 21});
        let mut session = Session::new(&steps, input);
        
        // Run first step
        let result1 = session.run_steps(1, true).await.unwrap();
        assert!(result1.is_some());
        assert!(!session.completed);
        assert_eq!(steps[0].get_run_count(), 1);
        assert_eq!(steps[1].get_run_count(), 0);

        // Run second step
        let result2 = session.run_steps(1, true).await.unwrap();
        assert!(result2.is_some());
        assert!(session.completed);
        assert_eq!(steps[0].get_run_count(), 1);
        assert_eq!(steps[1].get_run_count(), 1);
    }

    #[tokio::test]
    async fn test_session_retry_on_error() {
        let steps = vec![
            Step::new(
                "step1".to_string(),
                StepAction::Python(r#"
import json
source_data = json.loads(source)
res = source_data['value'] * 2
"#.to_string())
            ),
            Step::new(
                "step2".to_string(),
                StepAction::Python("invalid python code".to_string())
            ),
        ];

        let input = json!({"value": 21});
        let mut session = Session::new(&steps, input);
        
        // First attempt fails at step2
        let result1 = session.run_all(true).await;
        println!("After first run: {:?}", result1);
        println!("Current idx: {}", session.curr_idx);
        assert!(result1.is_err());
        
        // Retry the failed step
        let result2 = session.run_all(true).await;
        println!("After second run: {:?}", result2);
        println!("Current idx: {}", session.curr_idx);
        assert!(result2.is_err());
        
        // Check run counts
        assert_eq!(steps[0].get_run_count(), 1); // Expect this to run once
        assert_eq!(steps[1].get_run_count(), 2); // Second step fails twice
    }
}