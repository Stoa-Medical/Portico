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
        let mut json = json!({
            "id": self.identifiers.local_id,
            "global_uuid": self.identifiers.global_uuid,
            "description": self.description,
            "step_type": self.step_type.as_str(),
            "step_content": self.step_content,
            "created_at": self.timestamps.created.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updated_at": self.timestamps.updated.format("%Y-%m-%d %H:%M:%S").to_string(),
        });

        // Add llm_model field only for Prompt steps
        if let StepType::Prompt(model) = &self.step_type {
            json["llm_model"] = json!(model);
        }

        json
    }

    fn from_json(obj: Value) -> Result<Self> {
        let step_type_str = obj["step_type"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing step_type"))?;

        let step_content = obj["step_content"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing step_content"))?;

        // Handle optional fields
        let description = obj["description"].as_str().map(|s| s.to_string());
        let llm_model = obj["llm_model"].as_str().map(|s| s.to_string());

        // Create the appropriate StepType based on the type string and llm_model
        let step_type = match step_type_str {
            "python" => StepType::Python,
            "prompt" => StepType::Prompt(
                llm_model.unwrap_or_else(|| crate::JsonModeLLMs::MetaLlama33_70b.to_string()),
            ),
            _ => return Err(anyhow!("Invalid step type: {}", step_type_str)),
        };

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
            description,
            step_type,
            step_content: step_content.to_string(),
        })
    }
}
