use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

/// HTTP client for communicating with a Medplum FHIR server.
///
/// Handles OAuth2 client_credentials authentication and provides methods
/// for FHIR create, read, search, and transaction operations.
pub struct MedplumClient {
    base_url: String,
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
    http: reqwest::Client,
}

impl MedplumClient {
    /// Create a new MedplumClient from explicit parameters.
    pub fn new(base_url: String, client_id: String, client_secret: String) -> Self {
        Self {
            base_url,
            client_id,
            client_secret,
            access_token: None,
            http: reqwest::Client::new(),
        }
    }

    /// Create a MedplumClient from environment variables:
    ///   MEDPLUM_BASE_URL, MEDPLUM_CLIENT_ID, MEDPLUM_CLIENT_SECRET
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("MEDPLUM_BASE_URL")
            .context("MEDPLUM_BASE_URL environment variable not set")?;
        let client_id = std::env::var("MEDPLUM_CLIENT_ID")
            .context("MEDPLUM_CLIENT_ID environment variable not set")?;
        let client_secret = std::env::var("MEDPLUM_CLIENT_SECRET")
            .context("MEDPLUM_CLIENT_SECRET environment variable not set")?;

        Ok(Self::new(base_url, client_id, client_secret))
    }

    /// Authenticate using OAuth2 client_credentials flow.
    /// Stores the access token for subsequent requests.
    pub async fn authenticate(&mut self) -> Result<()> {
        let token_url = format!("{}/oauth2/token", self.base_url);

        let response = self
            .http
            .post(&token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .context("Failed to send authentication request to Medplum")?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .context("Failed to parse Medplum auth response")?;

        if !status.is_success() {
            let error_msg = body
                .get("error_description")
                .or_else(|| body.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown authentication error");
            return Err(anyhow!(
                "Medplum authentication failed ({}): {}",
                status,
                error_msg
            ));
        }

        let token = body
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Medplum auth response missing access_token"))?;

        self.access_token = Some(token.to_string());
        Ok(())
    }

    /// Get the current access token, or error if not authenticated.
    fn token(&self) -> Result<&str> {
        self.access_token
            .as_deref()
            .ok_or_else(|| anyhow!("Medplum client not authenticated — call authenticate() first"))
    }

    /// Create a FHIR resource.
    /// POST {base_url}/fhir/R4/{resource_type}
    pub async fn create_resource(&self, resource_type: &str, body: &Value) -> Result<Value> {
        let url = format!("{}/fhir/R4/{}", self.base_url, resource_type);
        let token = self.token()?;

        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .header("Content-Type", "application/fhir+json")
            .json(body)
            .send()
            .await
            .context("Failed to send create request to Medplum")?;

        Self::parse_fhir_response(response, "create").await
    }

    /// Read a FHIR resource by type and ID.
    /// GET {base_url}/fhir/R4/{resource_type}/{id}
    pub async fn read_resource(&self, resource_type: &str, id: &str) -> Result<Value> {
        let url = format!("{}/fhir/R4/{}/{}", self.base_url, resource_type, id);
        let token = self.token()?;

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .header("Accept", "application/fhir+json")
            .send()
            .await
            .context("Failed to send read request to Medplum")?;

        Self::parse_fhir_response(response, "read").await
    }

    /// Search for FHIR resources.
    /// GET {base_url}/fhir/R4/{resource_type}?{params}
    pub async fn search(&self, resource_type: &str, params: &Value) -> Result<Value> {
        let url = format!("{}/fhir/R4/{}", self.base_url, resource_type);
        let token = self.token()?;

        // Convert the params JSON object to query parameters
        let mut query_pairs: Vec<(String, String)> = Vec::new();
        if let Some(obj) = params.as_object() {
            for (key, val) in obj {
                let val_str = match val {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                query_pairs.push((key.clone(), val_str));
            }
        }

        let response = self
            .http
            .get(&url)
            .bearer_auth(token)
            .header("Accept", "application/fhir+json")
            .query(&query_pairs)
            .send()
            .await
            .context("Failed to send search request to Medplum")?;

        Self::parse_fhir_response(response, "search").await
    }

    /// Submit a FHIR transaction Bundle.
    /// POST {base_url}/fhir/R4/
    pub async fn transaction(&self, bundle: &Value) -> Result<Value> {
        let url = format!("{}/fhir/R4/", self.base_url);
        let token = self.token()?;

        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .header("Content-Type", "application/fhir+json")
            .json(bundle)
            .send()
            .await
            .context("Failed to send transaction request to Medplum")?;

        Self::parse_fhir_response(response, "transaction").await
    }

    /// Parse a FHIR HTTP response, returning the JSON body or an error.
    async fn parse_fhir_response(
        response: reqwest::Response,
        operation: &str,
    ) -> Result<Value> {
        let status = response.status();
        let body: Value = response
            .json()
            .await
            .context(format!("Failed to parse Medplum {} response body", operation))?;

        if !status.is_success() {
            // Try to extract an OperationOutcome issue message
            let issue_msg = body
                .get("issue")
                .and_then(|issues| issues.get(0))
                .and_then(|issue| issue.get("diagnostics"))
                .and_then(|d| d.as_str())
                .unwrap_or("Unknown FHIR error");
            return Err(anyhow!(
                "Medplum {} failed ({}): {}",
                operation,
                status,
                issue_msg
            ));
        }

        Ok(body)
    }
}
