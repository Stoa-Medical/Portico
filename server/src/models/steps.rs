/// Class for defining Steps
///   Each Step is a series of Python code that users define

use std::ffi::CString;
use std::env;

use thiserror::Error;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use reqwest::Client;
use anyhow::Result;
use serde_json::Value;

#[derive(Debug)]
enum StepAction {
    /// Python that will be executed within the current interpreter session
    Python(String),
    /// An LLM prompt that will query the configured LLM
    Prompt(String)
}

#[derive(Debug)]
pub struct Step {
    name: String,
    instruction: StepAction,
    // input: Value,
    // result: Option<Result<Option<Value>>>,
    run_count: u64
}

impl Step {
    pub fn new(
        name: String,
        instruction: StepAction
    ) -> Self {
        Self {
            name,
            instruction,
            run_count: 0,
        }
    }

    /// Runs the step with fresh context
    /// NOTE: If python code, expects the input value to be called `source` and expect result to be `res`
    pub async fn run(&self, source: Value) -> Result<Option<Value>> {
        match &self.instruction {
            StepAction::Prompt(the_prompt) => {
                // Do LLM call and return as a string
                match call_llm(the_prompt, source).await {
                    Ok(res_str) => Ok(Some(Value::String(res_str))),
                    Err(err) => Err(err.into())
                }
            }
            StepAction::Python(the_code) => {
                // Run code with independent context
                pyo3::prepare_freethreaded_python();
                Python::with_gil(|py| {
                    // Have clean state at each start
                    let locals = PyDict::new(py);
                    // Convert serde_json::Value to PyObject
                    let py_source = serde_json::to_string(&source)
                        .map_err(|e| anyhow::anyhow!("Failed to serialize source: {}", e))?;
                    locals.set_item("source", py_source)?;
                    
                    // Convert String to CString correctly
                    let code_as_cstr = CString::new(the_code.as_bytes())?;
                    py.run(code_as_cstr.as_c_str(), None, Some(&locals))?;
                    
                    // Get result and convert back to serde_json::Value if it exists
                    match locals.get_item("res") {
                        Ok(Some(res)) => {
                            let res_str = res.to_string();
                            let json_value: Value = serde_json::from_str(&res_str)
                                .map_err(|e| anyhow::anyhow!("Failed to parse result: {}", e))?;
                            Ok(Some(json_value))
                        }
                        Ok(None) => Ok(None),
                        Err(err) => Err(anyhow::anyhow!("Python error: {}", err))
                    }
                })
            }
        }
    }
}


/// The goal of a Session is to complete a series of steps and return the result
/// It receives a pointer to steps and is expected to run those steps
pub struct Session<'steps> {
    /// Pointer to steps that should be executed
    steps: &'steps Vec<Step>,
    /// The starting input of the session
    input_data: Value,
    /// The end result of a session
    result: Option<Result<Option<Value>>>,
    /// Whether the session ran to completion or not
    completed: bool,
    /// Current index of the session step
    curr_idx: usize,
}

impl<'steps> Session<'steps> {
    pub fn new(steps: &'steps Vec<Step>, input_data: Value) -> Self {
        Self {
            steps,
            input_data,
            result: None,
            completed: false,
            curr_idx: 0,
        }
    }
}

// ============ Internal functions ============

#[derive(Error, Debug)]
pub enum LLMError {
    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Missing API endpoint")]
    MissingApiEndpoint,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub async fn call_llm(prompt: &str, context: Value) -> Result<String, LLMError> {
    let api_key = env::var("LLM_API_KEY").map_err(|_| LLMError::MissingApiKey)?;
    let api_endpoint = env::var("LLM_API_ENDPOINT").map_err(|_| LLMError::MissingApiEndpoint)?;

    let request = serde_json::json!({
        "model": "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        "prompt": format!("{} | Context: {}", prompt, context),
        "max_tokens": 1000,
        "temperature": 0.7
    });

    let response: Value = Client::new()
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    // println!("{:?}", &response);

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| LLMError::InvalidResponse("No completion found".to_string()))
}