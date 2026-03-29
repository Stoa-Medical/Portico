use crate::{IdFields, TimestampFields};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataMapping {
    pub identifiers: IdFields,
    pub timestamps: TimestampFields,
    pub name: String,
    pub source_schema: Option<Value>,
    pub target_schema: Option<Value>,
    pub field_mappings: Option<Value>,
    pub ai_generated: bool,
    pub agent_id: Option<i32>,
}

impl DataMapping {
    pub fn new(name: String, agent_id: Option<i32>) -> Self {
        Self {
            identifiers: IdFields::new(),
            timestamps: TimestampFields::new(),
            name,
            source_schema: None,
            target_schema: None,
            field_mappings: None,
            ai_generated: false,
            agent_id,
        }
    }
}
