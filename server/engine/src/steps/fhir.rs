use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::services::fhir_client::MedplumClient;

/// Execute a FHIR step by performing operations against the Medplum server.
///
/// Config fields:
///   - operation (string): one of "create", "read", "search", "transaction"
///   - resource_type (string): FHIR resource type (e.g. "Patient", "Observation")
///   - id (string, optional): resource ID for read operations
///   - params (object, optional): search parameters for search operations
pub async fn execute(config: &Value, input: Value) -> Result<Value> {
    let mut client = MedplumClient::from_env()
        .context("Failed to initialize Medplum client from environment")?;

    client
        .authenticate()
        .await
        .context("Failed to authenticate with Medplum")?;

    let operation = config
        .get("operation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("FHIR step config missing 'operation' field"))?;

    let resource_type = config
        .get("resource_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Bundle");

    match operation {
        "create" => {
            client
                .create_resource(resource_type, &input)
                .await
                .context("FHIR create operation failed")
        }
        "read" => {
            let id = config
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("FHIR read operation requires 'id' in config"))?;
            client
                .read_resource(resource_type, id)
                .await
                .context("FHIR read operation failed")
        }
        "search" => {
            let params = config
                .get("params")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            client
                .search(resource_type, &params)
                .await
                .context("FHIR search operation failed")
        }
        "transaction" => {
            client
                .transaction(&input)
                .await
                .context("FHIR transaction operation failed")
        }
        other => Err(anyhow!("Unknown FHIR operation: {}", other)),
    }
}
