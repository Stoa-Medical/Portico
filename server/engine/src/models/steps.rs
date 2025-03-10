use crate::models::agents::Agent;
use crate::{IdFields, TimestampFields};

use std::ffi::CString;
use std::sync::atomic::{AtomicU64, Ordering};

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum StepType {
    Python,
    Prompt,
}

pub struct Step<'a> {
    // Database fields
    identifiers: IdFields,
    timestamps: TimestampFields,
    agent_owner: &'a Agent<'a>,
    step_type: StepType,
    step_content: String,
    run_count: u64,
    success_count: u64,
    // Runtime fields
    run_count_atomic: AtomicU64,
    success_count_atomic: AtomicU64,
    name: String, // Consider adding this to database fields if needed
}

impl<'a> Step<'a> {
    pub fn new(
        identifiers: IdFields,
        agent_owner: &'a Agent<'a>,
        step_type: StepType,
        step_content: String,
        name: String,
    ) -> Self {
        Self {
            identifiers,
            timestamps: TimestampFields::new(),
            agent_owner,
            step_type,
            step_content,
            run_count: 0,
            success_count: 0,
            run_count_atomic: AtomicU64::new(0),
            success_count_atomic: AtomicU64::new(0),
            name,
        }
    }

    /// Runs the step with fresh context
    /// NOTE: If python code, expects the input value to be called `source` and expect result to be `res`
    pub async fn run(&self, source_data: Value, step_idx: usize) -> Result<Option<Value>> {
        // Increment FIRST, before any potential errors
        self.run_count_atomic.fetch_add(1, Ordering::SeqCst);

        match &self.step_type {
            StepType::Prompt => match self.call_llm(&self.step_content, &source_data).await {
                Ok(res_str) => {
                    self.success_count_atomic.fetch_add(1, Ordering::SeqCst);
                    Ok(Some(Value::String(res_str)))
                }
                Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
            },
            StepType::Python => match self.exec_python(&source_data, &self.step_content) {
                Ok(result) => {
                    self.success_count_atomic.fetch_add(1, Ordering::SeqCst);
                    Ok(result)
                }
                Err(err) => Err(anyhow!("Step {} failed: {}", step_idx, err)),
            },
        }
    }

    pub fn get_run_count(&self) -> u64 {
        self.run_count_atomic.load(Ordering::SeqCst)
    }

    pub fn get_success_count(&self) -> u64 {
        self.success_count_atomic.load(Ordering::SeqCst)
    }

    // Placeholder for LLM call function
    async fn call_llm(&self, prompt: &str, context: &Value) -> Result<String> {
        // This would be implemented with your actual LLM calling logic
        // For now, just a placeholder
        Ok(format!("LLM response for prompt: {}", prompt))
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
                Err(err) => Err(anyhow::anyhow!("Python error: {}", err)),
            }
        })
    }

    // Methods to sync runtime counters with database fields
    pub fn sync_counters_to_db(&mut self) {
        self.run_count = self.run_count_atomic.load(Ordering::SeqCst);
        self.success_count = self.success_count_atomic.load(Ordering::SeqCst);
    }

    pub fn init_atomic_counters(&mut self) {
        self.run_count_atomic
            .store(self.run_count, Ordering::SeqCst);
        self.success_count_atomic
            .store(self.success_count, Ordering::SeqCst);
    }
}
