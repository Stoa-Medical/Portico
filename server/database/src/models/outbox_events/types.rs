use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An outbox event for reliable event publishing (transactional outbox pattern)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: Option<i64>,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub payload: Value,
    pub published: bool,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OutboxEvent {
    pub fn new(
        event_type: String,
        aggregate_type: String,
        aggregate_id: String,
        payload: Value,
    ) -> Self {
        Self {
            id: None,
            event_type,
            aggregate_type,
            aggregate_id,
            payload,
            published: false,
            published_at: None,
            created_at: chrono::Utc::now(),
        }
    }
}
