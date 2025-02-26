/// Lib module (access with `crate::`)
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
pub use models::{Agent, Step, RuntimeSession, Signal};

// ============ Custom Enums / Traits ============
// === Imports ===
use anyhow::Result;
use serde_json::Value;
use std::env;
use reqwest::Client;

// === Shared Enum definitions ===
#[derive(Debug, PartialEq)]
pub enum RunningStatus {
    Pending,
    InProgress,
    Completed,
    Failed
}

// ============ Struct definitions =============

pub struct IdFields {
    id: Option<u64>,
    global_uuid: String
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
            id: None, 
            global_uuid: uuid::Uuid::new_v4().to_string() 
        }
    }
    
    pub fn with_values(id: Option<u64>, global_uuid: String) -> Self {
        Self { id, global_uuid }
    }
}

pub struct TimestampFields {
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime
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
