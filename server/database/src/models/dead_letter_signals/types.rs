use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A signal that has failed processing and been moved to the dead letter queue
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadLetterSignal {
    pub id: Option<i64>,
    pub original_signal_id: i64,
    pub signal_type: String,
    pub initial_data: Option<Value>,
    pub error_message: String,
    pub retry_count: i32,
    pub last_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DeadLetterSignal {
    pub fn new(
        original_signal_id: i64,
        signal_type: String,
        initial_data: Option<Value>,
        error_message: String,
    ) -> Self {
        Self {
            id: None,
            original_signal_id,
            signal_type,
            initial_data,
            error_message,
            retry_count: 0,
            last_retry_at: None,
            created_at: chrono::Utc::now(),
        }
    }
}
