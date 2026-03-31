use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Option<i64>,
    pub actor: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AuditLogEntry {
    pub fn new(
        actor: String,
        action: String,
        resource_type: String,
        resource_id: Option<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            id: None,
            actor,
            action,
            resource_type,
            resource_id,
            details,
            created_at: chrono::Utc::now(),
        }
    }
}
