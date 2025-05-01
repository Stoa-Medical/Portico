/// Lib module (access with `crate::`)
///   Enums + traits go here (stylistic choice)!

/// Tests
#[cfg(test)]
mod tests;

/// Module with different data models
pub mod models;
pub use models::{Agent, RuntimeSession, Signal, Step};

/// Module for web scraping functionality
pub mod webscrape;
pub use webscrape::scrape_webpage;

// ============ Custom Enums / Traits ============
// === Imports ===
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use pyo3::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::{PgArgumentBuffer, PgPool};
use sqlx::{Postgres, Row};
use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use uuid::Uuid;

// === Shared Enum definitions ===
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone, Default)]
pub enum RunningStatus {
    #[default]
    Waiting,
    Running,
    Completed,
    Cancelled,
}

impl RunningStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunningStatus::Waiting => "waiting",
            RunningStatus::Running => "running",
            RunningStatus::Completed => "completed",
            RunningStatus::Cancelled => "cancelled",
        }
    }
}

impl sqlx::Type<Postgres> for RunningStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("running_status")
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for RunningStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match value.as_str()? {
            "waiting" => Ok(RunningStatus::Waiting),
            "running" => Ok(RunningStatus::Running),
            "completed" => Ok(RunningStatus::Completed),
            "cancelled" => Ok(RunningStatus::Cancelled),
            _ => Err("Invalid running status".into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for RunningStatus {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.as_str();
        buf.extend_from_slice(s.as_bytes());
        Ok(sqlx::encode::IsNull::No)
    }
}

impl std::str::FromStr for RunningStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "waiting" => Ok(RunningStatus::Waiting),
            "running" => Ok(RunningStatus::Running),
            "completed" => Ok(RunningStatus::Completed),
            "cancelled" => Ok(RunningStatus::Cancelled),
            _ => Err(format!("Invalid running status: {}", s)),
        }
    }
}

// ============ Struct definitions =============

#[derive(Clone, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct IdFields<I = i32>
where
    I: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static,
{
    pub local_id: Option<I>,
    pub global_uuid: String,
}

// Type aliases for common use cases
pub type IdFields32 = IdFields<i32>;
pub type IdFields64 = IdFields<i64>;

impl<I> Default for IdFields<I>
where
    I: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I> IdFields<I>
where
    I: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static,
{
    pub fn new() -> Self {
        Self {
            local_id: None,
            global_uuid: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_values(local_id: Option<I>, global_uuid: String) -> Self {
        Self {
            local_id,
            global_uuid,
        }
    }
}

#[derive(Clone, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct TimestampFields {
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

impl Default for TimestampFields {
    fn default() -> Self {
        Self::new()
    }
}

impl TimestampFields {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            created: now,
            updated: now,
        }
    }

    pub fn update(&mut self) {
        self.updated = chrono::Utc::now();
    }
}

/// Manages a Python execution environment for Agents
pub struct PythonRuntime {
    /// Python module containing all step functions for this agent
    module: Py<PyModule>,
    /// Maps step UUIDs to their Python function names
    step_functions: HashMap<String, String>,
}

impl PythonRuntime {
    /// Create a new Python runtime with a unique module name
    pub fn new(name: &str) -> Result<Self> {
        Python::with_gil(|py| {
            let module_name = format!("agent_{}", name.replace("-", "_"));
            let module = PyModule::new(py, &module_name)?;

            // Import common modules - just make it available in Python context
            let _ = py.import("json")?;

            Ok(Self {
                module: module.into(),
                step_functions: HashMap::new(),
            })
        })
    }

    /// Add a step to the runtime
    pub fn add_step(&mut self, step: &Step) -> Result<()> {
        if !step.is_python_step() {
            return Ok(()); // Skip non-Python steps
        }

        Python::with_gil(|py| {
            // Get the Python function code for this step
            let func_code = step.to_python_function();
            let func_name = step.python_function_name();

            // Get a reference to the module
            let module_ref = &self.module.bind(py);

            // Add the function to the module
            let locals = module_ref.dict();

            // Convert to CString for py.run
            let code_cstring = CString::new(func_code.as_bytes())?;
            py.run(code_cstring.as_c_str(), None, Some(&locals))?;

            // Store the function name mapped to the step UUID
            self.step_functions
                .insert(step.identifiers.global_uuid.clone(), func_name);

            Ok(())
        })
    }

    /// Execute a step with the given input data
    pub fn execute_step(&self, step_uuid: &str, input: Value) -> Result<Value> {
        let func_name = self
            .step_functions
            .get(step_uuid)
            .ok_or_else(|| anyhow!("Step function not found: {}", step_uuid))?;

        Python::with_gil(|py| {
            // Get a reference to the module
            let module_ref = &self.module.bind(py);

            // Get the json module
            let py_json = py.import("json")?;

            // Convert input to Python object
            let json_str = serde_json::to_string(&input)?;
            let py_input = py_json.getattr("loads")?.call1((json_str,))?;

            // Call the function
            if let Ok(func) = module_ref.getattr(func_name) {
                let result = func.call1((py_input,))?;

                // Convert the result back to Rust
                let py_json_str = py_json.getattr("dumps")?.call1((result,))?;
                let rust_json_str: String = py_json_str.extract()?;
                let rust_value: Value = serde_json::from_str(&rust_json_str)?;

                Ok(rust_value)
            } else {
                Err(anyhow!("Function not found in module: {}", func_name))
            }
        })
    }
}

// ============ Trait definitions =============

/// Item that is in the `public` schema (Portico-custom, not Supabase-predefined)
#[async_trait]
pub trait DatabaseItem {
    /// The integer type used for the local_id (defaults to i32)
    type IdType: sqlx::Type<Postgres>
        + for<'r> sqlx::Decode<'r, Postgres>
        + Send
        + Sync
        + Clone
        + std::fmt::Debug
        + 'static;

    fn id(&self) -> &IdFields<Self::IdType>;
    async fn try_db_create(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_update(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_delete(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>>
    where
        Self: Sized;
    async fn try_db_select_by_id(
        pool: &PgPool,
        id: &IdFields<Self::IdType>,
    ) -> Result<Option<Self>>
    where
        Self: Sized;
}

pub trait JsonLike {
    fn to_json(&self) -> Value;
    /// Creates new object
    fn from_json(obj: Value) -> Result<Self>
    where
        Self: Sized;
}

// ============ Shared functions ============

/// Checks if a record with the given UUID already exists in the specified table
pub async fn check_exists_by_uuid(pool: &PgPool, table: &str, uuid: &str) -> Result<bool> {
    let uuid_parsed = Uuid::parse_str(uuid)?;
    let query = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE global_uuid = $1)",
        table
    );
    sqlx::query_scalar::<_, bool>(&query)
        .bind(uuid_parsed)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to check if record exists: {}", e))
}

/// Returns a SQL fragment for Step JSON aggregation that's used in several queries
pub fn steps_json_agg_sql(parent_table: &str, parent_id_column: &str) -> String {
    format!(
        r#"COALESCE(
            (
                SELECT json_agg(json_build_object(
                    'id', s.id,
                    'global_uuid', s.global_uuid,
                    'created_at', s.created_at,
                    'updated_at', s.updated_at,
                    'description', s.description,
                    'step_type', s.step_type,
                    'step_content', s.step_content
                ))
                FROM steps s
                WHERE s.{} = {}.id
            ),
            '[]'::json
        ) as steps"#,
        parent_id_column, parent_table
    )
}

/// Returns a SQL fragment for the common Signal-Agent JOIN query
pub fn signal_with_agent_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            s.*,
            a.id as agent_id,
            a.global_uuid as agent_global_uuid,
            a.description as agent_description,
            a.agent_state as agent_state,
            a.created_at as agent_created_at,
            a.updated_at as agent_updated_at
        FROM signals s
        LEFT JOIN agents a ON s.agent_id = a.id
        {}
        "#,
        where_clause
    )
}

// Prefer JSON-mode supported models. Docs: https://docs.together.ai/docs/json-mode
pub enum JsonModeLLMs {
    MetaLlama33_70b,
    Qwen25_72b,
    DeepseekV3_671b,
}

impl std::fmt::Display for JsonModeLLMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let model_name = match self {
            JsonModeLLMs::MetaLlama33_70b => "meta-llama/Llama-3.3-70B-Instruct-Turbo",
            JsonModeLLMs::DeepseekV3_671b => "deepseek-ai/DeepSeek-V3",
            JsonModeLLMs::Qwen25_72b => "Qwen/Qwen2.5-VL-72B-Instruct",
        };
        write!(f, "{}", model_name)
    }
}

// Call the LLM with a specific model or use the default
pub async fn call_llm(prompt: &str, context: Value, model: Option<String>) -> Result<String> {
    const MAX_RETRIES: usize = 3;
    const INITIAL_RETRY_DELAY_MS: u64 = 500;

    let api_key = env::var("LLM_API_KEY").map_err(|_| anyhow!("Missing LLM_API_KEY environment variable"))?;
    let api_endpoint = env::var("LLM_API_ENDPOINT").map_err(|_| anyhow!("Missing LLM_API_ENDPOINT environment variable"))?;

    // Determine which model to use
    let model_name = if let Some(model_str) = model {
        // Try to match the provided model string with a known model
        match model_str.as_str() {
            "meta-llama/Llama-3.3-70B-Instruct-Turbo" => JsonModeLLMs::MetaLlama33_70b.to_string(),
            "deepseek-ai/DeepSeek-V3" => JsonModeLLMs::DeepseekV3_671b.to_string(),
            "Qwen/Qwen2.5-VL-72B-Instruct" => JsonModeLLMs::Qwen25_72b.to_string(),
            _ => JsonModeLLMs::MetaLlama33_70b.to_string(), // Default if not recognized
        }
    } else {
        // Use default model if none specified
        JsonModeLLMs::MetaLlama33_70b.to_string()
    };

    let request = serde_json::json!({
        "model": model_name,
        "prompt": format!("{} | Context: ```json\n{}\n```", prompt, context),
        "max_tokens": 1000,
        "temperature": 0.7
    });

    let mut last_error = None;

    // Implement retry logic with exponential backoff
    for attempt in 0..MAX_RETRIES {
        match attempt_llm_call(&api_endpoint, &api_key, &request).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);

                // Don't sleep on the last attempt
                if attempt < MAX_RETRIES - 1 {
                    // Exponential backoff: delay * 2^attempt
                    let backoff_ms = INITIAL_RETRY_DELAY_MS * (1 << attempt);
                    eprintln!("LLM API call failed (attempt {}/{}), retrying after {}ms: {}",
                        attempt + 1, MAX_RETRIES, backoff_ms, last_error.as_ref().unwrap());
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                }
            }
        }
    }

    // If we got here, all retries failed
    Err(last_error.unwrap_or_else(|| anyhow!("All LLM API call attempts failed")))
}

// Helper function to perform a single LLM API call attempt
async fn attempt_llm_call(api_endpoint: &str, api_key: &str, request: &Value) -> Result<String> {
    let response: Value = Client::new()
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(request)
        .send()
        .await
        .map_err(|e| anyhow!("LLM API request failed: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse LLM API response: {}", e))?;

    // Check if there's an error in the response
    if let Some(error) = response.get("error") {
        return Err(anyhow!("LLM API returned an error: {}", error));
    }

    // Extract completion text with better error handling
    response.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(String::from)
        .ok_or_else(|| {
            // Debug log the response structure for troubleshooting
            eprintln!("Unexpected LLM API response structure: {:?}", response);
            anyhow!("No completion found in LLM response. Check API endpoint and model configuration.")
        })
}

/// Loads steps for an agent by ID
pub async fn load_agent_steps(pool: &PgPool, agent_id: i32) -> Result<Option<Value>> {
    let steps_query = format!(
        "SELECT {} FROM (SELECT {}) subq",
        steps_json_agg_sql("subq", "agent_id"),
        agent_id
    );

    let steps_row = sqlx::query(&steps_query).fetch_one(pool).await?;

    let steps_json: Option<Value> = steps_row.get("steps");
    Ok(steps_json)
}
