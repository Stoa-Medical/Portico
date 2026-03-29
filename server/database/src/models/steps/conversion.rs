use super::types::{Step, StepType};
use crate::{IdFields, JsonLike, TimestampFields};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use uuid::Uuid;

impl Step {
    pub fn from_json_array(steps_json: &Value) -> Vec<Self> {
        if let Some(steps_array) = steps_json.as_array() {
            steps_array
                .iter()
                .filter_map(|step_json| Step::from_json(step_json.clone()).ok())
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl JsonLike for Step {
    fn to_json(&self) -> Value {
        json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "name": self.name,
            "description": self.description,
            "step_type": self.step_type.as_str(),
            "config": self.config,
            "step_order": self.step_order,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    fn from_json(obj: Value) -> Result<Self> {
        let step_type_str = obj["step_type"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing step_type"))?;

        let step_type = StepType::from_str(step_type_str)
            .map_err(|e| anyhow!("Invalid step_type: {}", e))?;

        let config = obj.get("config").cloned();
        let name = obj["name"].as_str().map(|s| s.to_string());
        let description = obj["description"].as_str().map(|s| s.to_string());
        let step_order = obj.get("step_order")
            .and_then(|v| v.as_i64())
            .and_then(|v| i32::try_from(v).ok());

        // Handle ID fields
        let local_id = obj["id"].as_i64().map(|id| id as i32);
        let global_uuid = if let Some(uuid_str) = obj["global_uuid"].as_str() {
            uuid_str.to_string()
        } else {
            Uuid::new_v4().to_string()
        };

        // Handle timestamp fields
        let created = if let Some(ts) = obj["created_at"].as_str() {
            chrono::DateTime::parse_from_rfc3339(ts)
                .map_err(|e| anyhow!("Invalid created_at timestamp: {}", e))?
                .with_timezone(&chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        let updated = if let Some(ts) = obj["updated_at"].as_str() {
            chrono::DateTime::parse_from_rfc3339(ts)
                .map_err(|e| anyhow!("Invalid updated_at timestamp: {}", e))?
                .with_timezone(&chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        Ok(Self {
            identifiers: IdFields {
                local_id,
                global_uuid,
            },
            timestamps: TimestampFields { created, updated },
            name,
            description,
            step_type,
            config,
            step_order,
        })
    }
}
