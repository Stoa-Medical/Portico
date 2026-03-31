use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Integration {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub name: String,
    pub connection_type: String,
    pub direction: String,
    pub config: Option<Value>,
    pub agent_id: Option<i32>,
    pub status: String,
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Integration {
    pub fn new(
        name: String,
        connection_type: String,
        direction: String,
        agent_id: Option<i32>,
    ) -> Self {
        Self {
            identifiers: IdFields::new(),
            timestamps: TimestampFields::new(),
            name,
            connection_type,
            direction,
            config: None,
            agent_id,
            status: "active".to_string(),
            last_seen_at: None,
        }
    }
}
