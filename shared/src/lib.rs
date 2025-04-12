/// Lib module (access with `crate::`)
///   Enums + traits go here (stylistic choice)!

/// Tests
#[cfg(test)]
mod tests;

/// Module with different data models
pub mod models;
pub use models::{Agent, RuntimeSession, Signal, Step};

// ============ Custom Enums / Traits ============
// === Imports ===
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::{PgArgumentBuffer, PgPool};
use sqlx::{Postgres, Row};
use std::env;
use std::ffi::CString;
use uuid;

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
pub struct IdFields {
    pub local_id: Option<i32>,
    pub global_uuid: String,
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

    pub fn with_values(local_id: Option<i32>, global_uuid: String) -> Self {
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

// ============ Trait definitions =============

// TODO: NEXT STEP -- implement this for the different models
/// Item that is in the `public` schema (Portico-custom, not Supabase-predefined)
#[async_trait]
pub trait DatabaseItem: Sized {
    fn id(&self) -> &IdFields;
    async fn try_db_create(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_update(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_delete(&self, pool: &PgPool) -> Result<()>;
    async fn try_db_select_all(pool: &PgPool) -> Result<Vec<Self>>;
    async fn try_db_select_by_id(pool: &PgPool, id: &IdFields) -> Result<Option<Self>>;
}

pub trait JsonLike {
    fn to_json(&self) -> Value;
    /// Creates new object
    fn from_json(obj: Value) -> Result<Self>
    where
        Self: Sized;
    /// Updates existing object
    fn update_from_json(&mut self, obj: Value) -> Result<Vec<String>>;
}

// ============ Shared functions ============

/// Checks if a record with the given UUID already exists in the specified table
pub async fn check_exists_by_uuid(pool: &PgPool, table: &str, uuid: &str) -> Result<bool> {
    let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE global_uuid = $1)", table);
    sqlx::query_scalar::<_, bool>(&query)
        .bind(uuid)
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
                    'created_timestamp', s.created_timestamp,
                    'last_updated_timestamp', s.last_updated_timestamp,
                    'name', s.name,
                    'description', s.description,
                    'step_type', s.step_type,
                    'step_content', s.step_content,
                    'success_count', s.success_count,
                    'run_count', s.run_count
                ) ORDER BY s.sequence_number)
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
            a.accepted_completion_rate as agent_accepted_completion_rate,
            a.completion_count as agent_completion_count,
            a.run_count as agent_run_count,
            a.created_timestamp as agent_created_timestamp,
            a.last_updated_timestamp as agent_last_updated_timestamp
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

// TODO: Add options to use different LLMs
pub async fn call_llm(prompt: &str, context: Value) -> Result<String> {
    let api_key = env::var("LLM_API_KEY").unwrap();
    let api_endpoint = env::var("LLM_API_ENDPOINT").unwrap();

    let request = serde_json::json!({
        "model": JsonModeLLMs::MetaLlama33_70b.to_string(),
        "prompt": format!("{} | Context: ```json\n{}\n```", prompt, context),
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
pub fn exec_python(source: Value, the_code: &str) -> Result<Value> {
    // Preps python interpreter (needs to run at least once, and repeat calls are negligible)
    pyo3::prepare_freethreaded_python();
    // Run code with independent context
    Python::with_gil(|py| {
        // Have clean state at each start
        let locals = PyDict::new(py);

        // Convert serde_json::Value to PyObject directly
        let py_json = pyo3::types::PyModule::import(py, "json")?;
        let incoming_data = serde_json::to_string(&source)?;
        let py_source = py_json.getattr("loads")?.call1((incoming_data,))?;
        locals.set_item("source", py_source)?;

        // Convert String to CString correctly
        let code_as_cstr = CString::new(the_code.as_bytes())?;
        py.run(code_as_cstr.as_c_str(), None, Some(&locals))?;

        // Get result and convert back to serde_json::Value
        match locals.get_item("result") {
            Ok(Some(res)) => {
                let py_json_str = py_json.getattr("dumps")?.call1((res,))?;
                let json_str: String = py_json_str.extract()?;
                let json_value: Value = serde_json::from_str(&json_str)?;
                Ok(json_value)
            }
            Ok(None) => Err(anyhow!(
                "Runtime error: unable to find return value (`result`)"
            )),
            Err(err) => Err(anyhow!("Python error: {}", err)),
        }
    })
}

/// Loads steps for an agent by ID
pub async fn load_agent_steps(pool: &PgPool, agent_id: i32) -> Result<Option<Value>> {
    let steps_query = format!(
        "SELECT {} FROM (SELECT {}) subq",
        steps_json_agg_sql("subq", "agent_id"),
        agent_id
    );

    let steps_row = sqlx::query(&steps_query)
        .fetch_one(pool)
        .await?;

    let steps_json: Option<Value> = steps_row.get("steps");
    Ok(steps_json)
}
