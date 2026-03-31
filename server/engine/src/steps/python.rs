use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

/// Execute a Python step by calling the Python sidecar service over HTTP.
///
/// The sidecar expects a POST to /execute with:
///   { "code": "<python source>", "input_json": <json value>, "timeout_seconds": <int> }
///
/// Config fields:
///   - script (string): Python source code to execute
///   - timeout_seconds (int, optional): execution timeout, default 30
pub async fn execute(config: &Value, input: Value) -> Result<Value> {
    let sidecar_url = std::env::var("PYTHON_SIDECAR_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());

    let code = config
        .get("script")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Python step config missing 'script' field"))?;

    let timeout_seconds = config
        .get("timeout_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);

    let request_body = json!({
        "code": code,
        "input_json": input,
        "timeout_seconds": timeout_seconds,
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/execute", sidecar_url))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(timeout_seconds + 5))
        .send()
        .await
        .context("Failed to connect to Python sidecar")?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .context("Failed to parse Python sidecar response")?;

    if !status.is_success() {
        let error_msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error from Python sidecar");
        return Err(anyhow!(
            "Python sidecar returned {}: {}",
            status,
            error_msg
        ));
    }

    // The sidecar returns { "output_json": <value>, ... }
    body.get("output_json")
        .cloned()
        .ok_or_else(|| anyhow!("Python sidecar response missing 'output_json' field"))
}
