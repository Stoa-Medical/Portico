/// Class for defining Steps
///   Each Step is a series of Python code that users define

use std::ffi::CString;
use std::sync::atomic::{AtomicU64, Ordering};

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;

use crate::call_llm;
use crate::models::session::SessionError;

#[derive(Debug)]
pub enum StepAction {
    /// Python that will be executed within the current interpreter session
    Python(String),
    /// An LLM prompt that will query the configured LLM
    Prompt(String)
}

#[derive(Debug)]
pub struct Step {
    pub name: String,
    pub instruction: StepAction,
    run_count: AtomicU64
}

impl Step {
    pub fn new(
        name: String,
        instruction: StepAction
    ) -> Self {
        Self {
            name,
            instruction,
            run_count: AtomicU64::new(0)
        }
    }

    /// Runs the step with fresh context
    /// NOTE: If python code, expects the input value to be called `source` and expect result to be `res`
    pub async fn run(&self, source: Value, step_idx: usize) -> Result<Option<Value>, SessionError> {
        // Increment FIRST, before any potential errors
        self.run_count.fetch_add(1, Ordering::SeqCst);
        
        match &self.instruction {
            StepAction::Prompt(the_prompt) => {
                match call_llm(the_prompt, source).await {
                    Ok(res_str) => Ok(Some(Value::String(res_str))),
                    Err(err) => Err(SessionError::StepFailed { 
                        step_idx: step_idx,  // This should probably come from Session
                        message: err.to_string() 
                    })
                }
            }
            StepAction::Python(the_code) => {
                match self.exec_python(&source, the_code) {
                    Ok(result) => Ok(result),
                    Err(err) => Err(SessionError::StepFailed { 
                        step_idx: step_idx,  // This should probably come from Session
                        message: err.to_string() 
                    })
                }
            }
        }
    }

    // Getter for analytics
    pub fn get_run_count(&self) -> u64 {
        self.run_count.load(Ordering::SeqCst)
    }

    fn exec_python(&self, source: &Value, the_code: &str) -> anyhow::Result<Option<Value>> {
        // Preps python interpreter (only needs to run once, though repeat calls are negligible)
        pyo3::prepare_freethreaded_python();
        // Run code with independent context
        Python::with_gil(|py| {
            // Have clean state at each start
            let locals = PyDict::new(py);
            // Convert serde_json::Value to PyObject
            let py_source = serde_json::to_string(source)?;
            locals.set_item("source", py_source)?;
            
            // Convert String to CString correctly
            let code_as_cstr = CString::new(the_code.as_bytes())?;
            py.run(code_as_cstr.as_c_str(), None, Some(&locals))?;
            
            // Get result and convert back to serde_json::Value if it exists
            match locals.get_item("res") {
                Ok(Some(res)) => {
                    let res_str = res.to_string();
                    let json_value: Value = serde_json::from_str(&res_str)?;
                    Ok(Some(json_value))
                }
                Ok(None) => Ok(None),
                Err(err) => Err(anyhow::anyhow!("Python error: {}", err))
            }
        })
    }
}
