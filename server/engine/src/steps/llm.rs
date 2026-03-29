use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

/// Execute an LLM step by calling the AI Gateway with an OpenAI-compatible
/// chat completion request.
///
/// Config fields:
///   - model (string): model identifier (e.g. "gpt-4o", "claude-sonnet-4-20250514")
///   - system_prompt (string): system message content
///   - output_schema (object, optional): JSON schema for structured output
pub async fn execute(config: &Value, input: Value) -> Result<Value> {
    let gateway_url = std::env::var("AI_GATEWAY_URL")
        .context("AI_GATEWAY_URL environment variable not set")?;
    let api_key = std::env::var("AI_GATEWAY_API_KEY")
        .context("AI_GATEWAY_API_KEY environment variable not set")?;

    let model = config
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("LLM step config missing 'model' field"))?;

    let system_prompt = config
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("LLM step config missing 'system_prompt' field"))?;

    // Build the user message from the input data
    let user_content = serde_json::to_string_pretty(&input)
        .context("Failed to serialize input for LLM step")?;

    let messages = vec![
        json!({ "role": "system", "content": system_prompt }),
        json!({ "role": "user", "content": user_content }),
    ];

    // Build the request body
    let mut request_body = json!({
        "model": model,
        "messages": messages,
    });

    // If an output schema is provided, include it for structured output
    if let Some(output_schema) = config.get("output_schema") {
        request_body["response_format"] = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "step_output",
                "schema": output_schema,
            },
        });
    }

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", gateway_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .context("Failed to connect to AI Gateway")?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .context("Failed to parse AI Gateway response")?;

    if !status.is_success() {
        let error_msg = body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error from AI Gateway");
        return Err(anyhow!("AI Gateway returned {}: {}", status, error_msg));
    }

    // Extract the assistant's message content from the chat completion response
    let content = body
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow!("AI Gateway response missing choices[0].message.content"))?;

    // Try to parse the content as JSON; if it fails, wrap it as a string value
    match serde_json::from_str::<Value>(content) {
        Ok(parsed) => Ok(parsed),
        Err(_) => Ok(json!({ "response": content })),
    }
}
