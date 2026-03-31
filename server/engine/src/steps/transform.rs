use anyhow::Result;
use serde_json::Value;

/// Execute a Transform step (HL7v2/X12 to FHIR mapping).
///
/// This is a stub implementation that passes input through unchanged.
/// Future versions will implement native HL7v2 and X12 to FHIR transformations.
pub async fn execute(_config: &Value, input: Value) -> Result<Value> {
    println!("[INFO] Transform step not yet implemented — passing input through unchanged");
    Ok(input)
}
