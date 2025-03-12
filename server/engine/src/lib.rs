/// Lib module (access with `crate::`)
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
pub use models::{Agent, RuntimeSession, Signal, Step};

// ============ Custom Enums / Traits ============
// === Imports ===
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use std::env;
use sqlx::postgres::PgPool;
use std::ffi::CString;
use pyo3::prelude::*;
use pyo3::types::PyDict;

// === Shared Enum definitions ===
#[derive(Debug, PartialEq)]
pub enum RunningStatus {
    Waiting,
    Running,
    Completed,
    Cancelled,
}

// ============ Struct definitions =============

#[derive(Clone)]
pub struct IdFields {
    local_id: Option<u64>,
    global_uuid: String,
}

impl Default for IdFields {
    fn default() -> Self {
        Self::new()
    }
}

impl IdFields {
    // TODO: Have some way to increment correctly
    //    Primarily matters for `RuntimeSession` which is created here
    //    (everything else is created in the UI, and Supabase is the source-of-truth)
    pub fn new() -> Self {
        Self {
            local_id: None,
            global_uuid: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_values(local_id: Option<u64>, global_uuid: String) -> Self {
        Self { local_id, global_uuid }
    }
}

#[derive(Clone)]
pub struct TimestampFields {
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}

impl Default for TimestampFields {
    fn default() -> Self {
        Self::new()
    }
}

impl TimestampFields {
    pub fn new() -> Self {
        let now = chrono::Local::now().naive_utc();
        Self {
            created: now,
            updated: now,
        }
    }

    pub fn update(&mut self) {
        self.updated = chrono::Local::now().naive_utc();
    }
}

// ============ Trait definitions =============

/// Item that is in the `public` schema (Portico-custom, not Supabase-predefined)
trait DatabaseItem {
    // Default implementations
    // NOTE: `read` is not implemented since this works on an already-serialized item.
    //   So this function is mainly taking the serialized Rust state and syncing it with the database

    // TODO: Fix implementation of these: https://docs.rs/sqlx/latest/sqlx/macro.query.html#
    async fn generate_create_query(&self) -> Result<bool> {
        // Make CREATE query with `query!` macro
        // Return QUERY
        Ok(false)
    }

    async fn generate_update_query(&self) -> Result<bool> {
        // Make UPDATE query with `query!` macro
        // Return QUERY
        Ok(false)
    }

    async fn generate_delete_query(&self) -> Result<bool> {
        // Make DELETE query with `query!` macro
        // Return QUERY
        Ok(false)
    }

    async fn generate_select_query(pool: &PgPool) -> Result<bool> {
        // Make DELETE query with `query!` macro
        // Return QUERY
        Ok(false)
    }

    // Struct-specific implementations
    fn get_table_name(&self) -> &'static str;
    fn get_db_fields(&self) -> Vec<&str>;
}

// ============ Shared functions ============

// TODO: Add various supported TogetherAI models
pub async fn call_llm(prompt: &str, context: Value) -> Result<String> {
    let api_key = env::var("LLM_API_KEY").unwrap();
    let api_endpoint = env::var("LLM_API_ENDPOINT").unwrap();

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

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No completion found"))
}

/// Executes provided python code
// TODO: Refactor this to enforce that the code is a pure function, and the function should just be called
pub fn exec_python(source: Value, the_code: &str) -> Result<Value> {
    // Preps python interpreter (needs to run at least once, and repeat calls are negligible)
    pyo3::prepare_freethreaded_python();
    // Run code with independent context
    Python::with_gil(|py| {
        // Have clean state at each start
        let locals = PyDict::new(py);
        // Convert serde_json::Value to PyObject
        let incoming_data = serde_json::to_string(&source)?;
        locals.set_item("source", incoming_data)?;

        // Convert String to CString correctly
        let code_as_cstr = CString::new(the_code.as_bytes())?;
        py.run(code_as_cstr.as_c_str(), None, Some(&locals))?;

        // Get result and convert back to serde_json::Value if it exists
        match locals.get_item("result") {
            Ok(Some(res)) => {
                let res_str = res.to_string();
                let json_value: Value = serde_json::from_str(&res_str)?;
                Ok(json_value)
            }
            Ok(None) => Err(anyhow!("Runtime error: unable to find return value (`result`)")),
            Err(err) => Err(anyhow!("Python error: {}", err)),
        }
    })
}
