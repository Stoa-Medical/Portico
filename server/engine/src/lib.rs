use anyhow::{anyhow, Result};
use portico_shared::models::Agent;
use portico_shared::DatabaseItem;
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

// TODO: Have this serialize to BridgeMessage
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

            // TODO: Process the signal by creating a runtime session
            // Note: agent.run() needs a mutable reference
            // Since we only have an immutable reference to the agent inside the read lock,
            // we'll need to modify the API or do something more complex in a real implementation

            // For demonstration, we'll log what we would do
            println!("Found agent with ID: {}", agent_id);
            println!("Would execute agent with data: {:?}", data_clone);

            // In a real implementation, you might:
            // 1. Either modify agent.run() to take &self instead of &mut self
            // 2. Or find a way to get a mutable reference (would require changes to the design)

            // Simplified placeholder for what would happen:
            // let session = agent_ref.run_immutable(data_clone).await?;
            // session.try_db_create(&pool).await?;
        } else {
            eprintln!("Agent with ID {} not found", agent_id);
        }
    }
}

// Handler for CreateAgent messages
pub async fn handle_create_agent(data: Value, agent_map: AgentMap, _pool: PgPool) {
    println!("Processing Agent creation: {:?}", data);

    // TODO: Implement Agent deserialization from Value
    // For now, we'll log what we would do
    println!("Would deserialize Agent from data and add to map");
    println!("Agent data: {:?}", data);

    // The actual implementation would:
    // 1. Deserialize the agent from JSON
    // 2. Get the UUID
    // 3. Insert into the agent_map

    // This is just for demonstration - would be replaced with actual implementation
    if let Some(uuid) = data.get("global_uuid").and_then(|v| v.as_str()) {
        println!("Would add agent with UUID: {} to map", uuid);
        // In real implementation:
        // let mut agents = agent_map.write().await;
        // agents.insert(uuid.to_string(), agent);
    } else {
        eprintln!("Agent data missing global_uuid field");
    }
}

// Handler for UpdateAgent messages
pub async fn handle_update_agent(data: Value, agent_map: AgentMap, _pool: PgPool) {
    println!("Processing Agent update: {:?}", data);

    // TODO: Implement Agent deserialization and update
    // Extract the UUID to identify which agent to update
    if let Some(uuid) = data.get("global_uuid").and_then(|v| v.as_str()) {
        println!("Would update agent with UUID: {}", uuid);

        // In real implementation:
        // 1. Deserialize the updated agent from data
        // 2. Update in the map
        // let mut agents = agent_map.write().await;
        // if agents.contains_key(uuid) {
        //     agents.insert(uuid.to_string(), updated_agent);
        //     println!("Agent updated in map");
        // } else {
        //     println!("Agent with UUID {} not found in map", uuid);
        // }
    } else {
        eprintln!("Update Agent data missing global_uuid field");
    }
}

// Handler for DeleteAgent messages
pub async fn handle_delete_agent(data: Value, agent_map: AgentMap, _pool: PgPool) {
    println!("Processing Agent deletion: {:?}", data);

    // Extract agent UUID
    if let Some(uuid) = data.get("global_uuid").and_then(|v| v.as_str()) {
        println!("Would remove agent with UUID: {}", uuid);

        // TODO: The actual deletion would be:
        // let mut agents = agent_map.write().await;
        // if agents.remove(uuid).is_some() {
        //     println!("Agent removed from map");
        // } else {
        //     println!("Agent with UUID {} not found in map", uuid);
        // }
    } else {
        eprintln!("Delete Agent data missing global_uuid field");
    }
}

// TODO: This function should find the diff, and make the updates accordingly (maybe implement as a trait in shared lib)
// Helper function to parse Agent from JSON Value
// Note: You might want to replace this with a proper implementation
fn try_agent_from_value(value: &Value) -> Result<Agent> {
    // This is a placeholder - you should implement proper Agent deserialization
    // For now, we'll log what we would do and return an error
    eprintln!("TODO: Implement Agent deserialization from Value");
    eprintln!("Value to deserialize: {:?}", value);

    // In a real implementation, you would do something like:
    // serde_json::from_value(value.clone()).map_err(|e| anyhow!("Failed to deserialize Agent: {}", e))

    // Or instead use Agent's own deserialization method if it has one
    Err(anyhow!("Agent deserialization not implemented yet"))
}
