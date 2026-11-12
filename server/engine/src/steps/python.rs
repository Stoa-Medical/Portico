use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

/// Runs a Python step in the sidecar.
///
/// Config fields:
///   - script (string): Python source. It reads `input_data` and assigns `output_data`.
///   - timeout_seconds (int, optional): execution timeout, default 30
///
/// The sidecar carries JSON as strings in both directions and reports step
/// failures as HTTP 200 with `success: false`.
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
        "input_json": serde_json::to_string(&input)?,
        "timeout_seconds": timeout_seconds,
    });

    let response = reqwest::Client::new()
        .post(format!("{sidecar_url}/execute"))
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
        return Err(anyhow!("Python sidecar returned {status}: {body}"));
    }

    if body.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let error = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("no error message");
        return Err(anyhow!("Python step failed: {error}"));
    }

    let output_json = body
        .get("output_json")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Python sidecar response missing 'output_json'"))?;

    serde_json::from_str(output_json).context("Python step output is not valid JSON")
}
