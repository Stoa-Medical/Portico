use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineState {
    pub id: i32,
    pub last_redis_stream_id: Option<String>,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            id: 1,
            last_redis_stream_id: None,
            last_heartbeat: None,
            updated_at: chrono::Utc::now(),
        }
    }
}
