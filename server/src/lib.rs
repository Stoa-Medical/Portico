#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

/// Lib module (access with `crate::` or `portico_server::``
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
pub use models::{Agent, Step, RuntimeSession};
pub use models::jobs::{Job, JobStatus};

// ============ Custom Enums / Traits ============
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use async_trait::async_trait;
use tokio::fs::read_to_string;
use serde::{Serialize, Deserialize};
use std::env;
use thiserror::Error;
use reqwest::Client;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataSource {
    Json(Value),
    File(PathBuf),
    Url(String), // Performs a GET request and processes HTML as string
}

impl DataSource {
    /// Extract the content into a Value
    pub async fn extract(&self) -> Result<Value, anyhow::Error> {
        match self {
            DataSource::Json(value) => Ok(value.clone()),
            DataSource::File(path) => {
                let content = read_to_string(path).await?;
                Ok(serde_json::from_str(&content)?)
            },
            // Assume if URL data, it should be JSON
            // TODO: allow HTML as option
            DataSource::Url(url) => {
                let client = reqwest::Client::new();
                Ok(client.get(url)
                    .send()
                    .await?
                    .json()
                    .await?)
            }
        }
    }
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

// Move trait definitions to the top with other traits
#[async_trait]
pub trait CanAct {
    async fn act(&mut self, source: DataSource, job: Option<&mut Job>) -> Result<Value, anyhow::Error>;
}

#[async_trait]
pub trait CanReact {
    async fn react(&mut self, source: DataSource, job: Option<&mut Job>) -> Result<Value, anyhow::Error>;
}
