use anyhow::{anyhow, Result};
use portico_shared::models::Agent;
use portico_shared::{DatabaseItem, JsonLike};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum BridgeMessage {
    ServerInit(bool),
    CreateSignal(Value),
    CreateAgent(Value),
    UpdateAgent(Value),
    DeleteAgent(Value),
}

// Thread-safe Agent map type
pub type AgentMap = Arc<RwLock<HashMap<String, Agent>>>;

pub fn read_json_message(stream: &mut TcpStream) -> Result<BridgeMessage> {
    // Read 4-byte length prefix (u32 in network byte order)
    let mut size_buffer = [0; 4];
    stream.read_exact(&mut size_buffer)?;
    let message_size = u32::from_be_bytes(size_buffer) as usize;

    // Allocate buffer of exact size and read the whole message
    let mut buffer = vec![0; message_size];
    stream.read_exact(&mut buffer)?;
    let data = String::from_utf8_lossy(&buffer);

    match serde_json::from_str::<Value>(&data) {
        Ok(json) => {
            // `ServerInit` if it contains the `server-init` field with a bool value
            //  (For the CRUD updates below, return the `data.record` value as part of the final result)
            // `CreateSignal` if the table is "signals"
            // `CreateAgent` if the table is "agents" (`data.table`) and and type is "CREATE" (`data.type`)
            // `UpdateAgent` ... "UPDATE" (`data.type`)
            // `DeleteAgent` ... "DELETE" (`data.type`)
            // Otherwise: return Err
            if let Some(init) = json.get("server-init") {
                if let Some(init_value) = init.as_bool() {
                    Ok(BridgeMessage::ServerInit(init_value))
                } else {
                    Err(anyhow!("server-init value must be a boolean"))
                }
            } else if let Some(data) = json.get("data") {
                let table = data
                    .get("table")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow!("Missing or invalid table field"))?;

                match table {
                    "signals" => Ok(BridgeMessage::CreateSignal(
                        data.get("record").cloned().unwrap_or(Value::Null),
                    )),
                    "agents" => {
                        let event_type = data
                            .get("type")
                            .and_then(|t| t.as_str())
                            .ok_or_else(|| anyhow!("Missing or invalid type field"))?;

                        let record = data.get("record").cloned().unwrap_or(Value::Null);

                        match event_type {
                            "INSERT" => Ok(BridgeMessage::CreateAgent(record)),
                            "UPDATE" => Ok(BridgeMessage::UpdateAgent(record)),
                            "DELETE" => Ok(BridgeMessage::DeleteAgent(record)),
                            _ => Err(anyhow!("Unsupported event type: {}", event_type)),
                        }
                    }
                    _ => Err(anyhow!("Unsupported table: {}", table)),
                }
            } else {
                Err(anyhow!("Unrecognized message format"))
            }
        }
        Err(e) => Err(anyhow!("Failed to parse JSON: {}", e)),
    }
}

// Handler for CreateSignal messages
pub async fn handle_create_signal(data: Value, agent_map: AgentMap, pool: PgPool) {
    println!("Processing Signal creation: {:?}", data);

    // Extract agent_id from signal data
    let agent_id = match data.get("agent_id").and_then(|id| id.as_str()) {
        Some(id) => id.to_string(),
        None => {
            eprintln!("Signal missing agent_id field");
            return;
        }
    };

    // Find the agent and process it
    {
        // Scope the read lock to this block
        let agents_guard = agent_map.read().await;

        // Get a reference to the agent
        if let Some(agent_ref) = agents_guard.get(&agent_id) {
            // Make a copy of the data
            let data_clone = data.clone();

            // Now we can use the immutable reference directly
            println!("Found agent with ID: {}", agent_id);
            match agent_ref.run(data_clone).await {
                Ok(session) => {
                    // Successfully ran the agent, try to save the session to the database
                    println!("Agent execution successful, saving session");
                    if let Err(e) = session.try_db_create(&pool).await {
                        eprintln!("Failed to save session to database: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Agent execution failed: {}", e);
                }
            }
        } else {
            eprintln!("Agent with ID {} not found", agent_id);
        }
    }
}

// Handler for CreateAgent messages
pub async fn handle_create_agent(data: Value, agent_map: AgentMap) {
    println!("Processing Agent creation: {:?}", data);

    // Attempt to deserialize the Agent from JSON using the from_json method
    match portico_shared::models::agents::Agent::from_json(data.clone()) {
        Ok(agent) => {
            // Extract the UUID to use as the map key
            let agent_uuid = agent.identifiers.global_uuid.clone();

            if agent_uuid.is_empty() {
                eprintln!("Agent has empty global_uuid, cannot add to map");
                return;
            }

            println!("Adding agent with UUID: {} to map", agent_uuid);

            // Get write access to the agent map
            let mut agents = agent_map.write().await;

            // Insert the agent into the map
            agents.insert(agent_uuid, agent);

            // Note: We might want to handle persisting to DB here as well if needed
            println!("Agent added to map successfully");
        }
        Err(e) => {
            eprintln!("Failed to deserialize Agent from JSON: {}", e);
            eprintln!("Agent data: {:?}", data);
        }
    }
}

// Handler for UpdateAgent messages
pub async fn handle_update_agent(data: Value, agent_map: AgentMap) {
    println!("Processing Agent update: {:?}", data);

    // Extract the UUID to identify which agent to update
    let uuid = match data.get("global_uuid").and_then(|v| v.as_str()) {
        Some(uuid) if !uuid.is_empty() => uuid.to_string(),
        _ => {
            eprintln!("Update Agent data missing valid global_uuid field");
            return;
        }
    };

    println!("Updating agent with UUID: {}", uuid);

    // Get write access to the agent map
    let mut agents = agent_map.write().await;

    if let Some(existing_agent) = agents.get_mut(&uuid) {
        // Update the existing agent with the new data
        match existing_agent.update_from_json(data.clone()) {
            Ok(updated_fields) => {
                if !updated_fields.is_empty() {
                    println!("Agent updated: fields changed: {:?}", updated_fields);

                    // Optionally update the agent in the database if needed
                    // This could be added if there's a need to persist changes made by the bridge
                } else {
                    println!("No fields were changed during agent update");
                }
            }
            Err(e) => {
                eprintln!("Failed to update agent: {}", e);
            }
        }
    } else {
        // Agent not found in the map, fallback to creating a new one
        println!(
            "Agent with UUID {} not found in map, creating new instead",
            uuid
        );
        match portico_shared::models::agents::Agent::from_json(data.clone()) {
            Ok(new_agent) => {
                agents.insert(uuid, new_agent);
                println!("New agent inserted into map");
            }
            Err(e) => {
                eprintln!("Failed to deserialize agent for insert: {}", e);
            }
        }
    }
}

// Handler for DeleteAgent messages
pub async fn handle_delete_agent(data: Value, agent_map: AgentMap) {
    println!("Processing Agent deletion: {:?}", data);

    // Extract agent UUID
    let uuid = match data.get("global_uuid").and_then(|v| v.as_str()) {
        Some(uuid) if !uuid.is_empty() => uuid.to_string(),
        _ => {
            eprintln!("Delete Agent data missing valid global_uuid field");
            return;
        }
    };

    println!("Removing agent with UUID: {}", uuid);

    // Get write access to the agent map and remove the agent
    let mut agents = agent_map.write().await;

    if agents.remove(&uuid).is_some() {
        println!("Agent successfully removed from map");

        // Optionally, we could also delete from the database if needed
        // This would be useful if the bridge needs to manage database state
    } else {
        println!("Agent with UUID {} not found in map", uuid);
    }
}
