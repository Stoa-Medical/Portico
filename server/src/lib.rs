#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

/// Lib module (access with `crate::` or `portico_server::``
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
pub use models::{Agent, Step, RuntimeSession};

// ============ Custom Enums / Traits ============
use anyhow::Result;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::env;
use thiserror::Error;
use reqwest::Client;

// === Shared Enum definitions ===
pub enum RunningStatus {
    Pending,
    InProgress,
    Completed,
    Failed
}

// ============ Struct definitions =============

pub struct IdFields {
    id: u64,
    global_uuid: String
}

impl IdFields {
    // TODO: Have some way to increment correctly
    //    Primarily matters for `RuntimeSession` which is created here
    //    (everything else is created in the UI, and Supabase is the source-of-truth)
    pub fn new(id: u64, global_uuid: String) -> Self {
        Self { id, global_uuid }
    }
}

pub struct TimestampFields {
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime
}

impl TimestampFields {
    pub fn new() -> Self {
        let now = chrono::Local::now().naive_utc();
        Self {
            created: now,
            updated: now
        }
    }

    pub fn update(&mut self) {
        self.updated = chrono::Local::now().naive_utc();
    }
}


// ============ Trait definitions =============

/// Item that is in the `public` schema (Portico-custom, not Supabase-predefined)
trait DatabaseItem {
    /// Default implementations
    fn try_create(&self) -> Result<()>;
    fn try_read(&self) -> Result<()>;
    fn try_update(&self) -> Result<()>;
    fn try_delete(&self) -> Result<()>;

    /// Struct-specific implementations
    fn get_table_name(&self) -> &str;
}

trait AuditLogger {

    fn update_log_json(&self) -> Result<()>;

}


// ============ Supabase Realtime things =============
#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeMessage {
    #[serde(rename = "type")]
    message_type: MessageType,
    table: String,
    #[serde(rename = "eventType")]
    event_type: MessageType,
    new: Value,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MessageType {
    Insert,
    Update,
    Delete,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub schema: String,
    pub table: String,
    #[serde(rename = "filter")]
    pub event_filter: String,
}


// ============ Shared functions ============

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageContent {
    Json(Value),
    Text(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    timestamp: i64,
    content: MessageContent,
    #[serde(default)]
    metadata: Option<Value>, // Optional metadata for additional context
}

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

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| LLMError::InvalidResponse("No completion found".to_string()))
}
