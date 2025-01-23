#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

/// Lib module (access with `crate::` or `portico_server::``
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
/// API routes
pub mod api;


// ============ Traits ============
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::str::FromStr;
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use async_trait::async_trait;
use tokio::fs::read_to_string;

#[derive(Clone, Debug)]
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

/// Something that can respond to data
#[async_trait]
// NEXT-STEP: Modified to `DataSource` -- get this to work
pub trait CanReact {
    /// Configure what types of data this reactor accepts
    fn accepts(&self) -> Vec<&str> {
        vec!["application/json"]
    }

    /// React to a single piece of data
    async fn react(&self, source: DataSource) -> Result<Value>;
}


/// Something that can act on its own (based on a schedule)
#[async_trait]
pub trait CanAct {
    /// The CRON schedule for when this actor should run
    fn schedule(&self) -> &str;
    
    /// The action to perform on schedule
    async fn act(&self, source: DataSource) -> Result<Value>;
    
    /// Check if it's time to run based on the schedule
    fn should_run(&self, last_run: Option<DateTime<Utc>>) -> bool {
        let schedule = Schedule::from_str(self.schedule()).ok().expect("CRON load failed -- is your syntax right?");
        let now = Utc::now();
        
        match last_run {
            None => true, // Never run before
            Some(last) => schedule.after(&last).next().map_or(false, |next| next <= now)
        }
    }
}


// ============ Shared functions ============

use std::env;
use thiserror::Error;
use reqwest::Client;
use serde::{Serialize, Deserialize};

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
