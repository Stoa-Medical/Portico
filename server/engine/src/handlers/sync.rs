use crate::core::agent_manager::AgentManager;
use crate::json_to_proto_struct;
use crate::proto::{SignalRequest, SignalResponse, SyncScope};
use portico_shared::JsonLike;
use serde_json::Value;
use tonic::Status;

// Sync operation handler
pub async fn handle_sync(
    manager: &AgentManager,
    signal: &SignalRequest,
    runtime_session_uuid: String,
) -> Result<SignalResponse, Status> {
    println!("[INFO] Processing sync operation");

    if let Some(crate::proto::signal_request::Payload::Sync(sync)) = &signal.payload {
        match sync.scope() {
            SyncScope::All => {
                println!("[INFO] Syncing all entities");

                // Return the current agents
                let agents = manager.agents.read().await;
                let mut result_map = serde_json::Map::new();

                for (uuid, agent) in agents.iter() {
                    let agent_json = agent.to_json();
                    result_map.insert(uuid.clone(), agent_json);
                }

                let result_value = Value::Object(result_map);

                Ok(SignalResponse {
                    success: true,
                    message: "All entities synced".to_string(),
                    runtime_session_uuid,
                    result_data: Some(json_to_proto_struct(&result_value)),
                })
            }
            SyncScope::Specific => {
                // Check if specific UUIDs were provided
                if sync.entity_uuids.is_empty() {
                    return Err(Status::invalid_argument(
                        "Missing entity_uuids for SPECIFIC sync scope",
                    ));
                }

                println!("[INFO] Syncing specific entities: {:?}", sync.entity_uuids);

                // Sync specific entities by UUID
                let agents = manager.agents.read().await;
                let mut result_map = serde_json::Map::new();

                for uuid in &sync.entity_uuids {
                    if let Some(agent) = agents.get(uuid) {
                        let agent_json = agent.to_json();
                        result_map.insert(uuid.clone(), agent_json);
                    }
                }

                let result_value = Value::Object(result_map);

                Ok(SignalResponse {
                    success: true,
                    message: "Specific entities synced".to_string(),
                    runtime_session_uuid,
                    result_data: Some(json_to_proto_struct(&result_value)),
                })
            }
        }
    } else {
        Err(Status::invalid_argument(
            "Missing sync payload for SYNC signal type",
        ))
    }
}
