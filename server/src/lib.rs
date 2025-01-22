#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

/// Lib module (access with `crate::` or `portico_server::``
///   Enums + traits go here (stylistic choice)!

/// Module with different data models
pub mod models;
/// API routes
pub mod api;

// /// Supported network protocols for Agents
// pub enum DataProtocol {
//     // PlaintextTcpIp,  // TODO: Implement this for HL7 messages
//     // XmlSoap,
//     JsonRest
// }

// /// Something that can receive data on an incoming port
// pub trait CanReceive {
//     /// The port the channel can receive on (only 1 allowed)
//     fn incoming_port(&self) -> u16;
//     /// Starts listener on the incoming port
//     fn receive(&self, protocol: DataProtocol) -> Result<serde_json::Value, std::io::Error>;
// }

// /// Something that can send data to an outgoing destination
// pub trait CanReact {
//     /// The destination for the resulting action (expressed as a String)
//     fn outgoing_destinations(&self) -> Vec<String>;
//     /// Performs the action in reaction to the data, and returns to the outgoing destination
//     fn react(&self, source: serde_json::Value, protocol: DataProtocol) -> Result<(), std::io::Error>;
// }

// pub trait CanSave {

// }

// ============ Shared functions ============

use std::env;
use thiserror::Error;
use reqwest::Client;
use anyhow::Result;
use serde_json::Value;
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

/// Tests
#[cfg(test)]
mod tests {
    use super::*;

    
}