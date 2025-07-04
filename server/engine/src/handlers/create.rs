use crate::core::agent_manager::AgentManager;
use crate::proto::GeneralResponse;
use crate::proto_struct_to_json;
use portico_shared::models::Agent;
use portico_shared::JsonLike;
use portico_shared::DatabaseItem;
use prost_types::Struct;
use tonic::Status;

// Create agent operation handler
pub async fn handle_create_agent(
    manager: &mut AgentManager,
    agent_json_struct: &Struct,
) -> Result<GeneralResponse, Status> {
    let agent_json = proto_struct_to_json(agent_json_struct);

    println!("[INFO] Processing Agent creation: {}", agent_json);

    match Agent::from_json(agent_json.clone()) {
        Ok(agent) => {
            let agent_uuid = agent.identifiers.global_uuid.clone();
            println!("[INFO] Adding agent with UUID: {}", agent_uuid);

            // Setup a queue for this agent
            if let Err(e) = manager.setup_agent_queue(agent_uuid.clone()).await {
                eprintln!("[ERROR] Failed to setup agent queue: {}", e);
                return Err(Status::internal("Failed to setup message queue for agent"));
            }

            // Store the agent
            let mut agents_guard = manager.agents.write().await;
            agents_guard.insert(agent_uuid.clone(), agent);

            // Save to database if not already there
            let agent = agents_guard.get(&agent_uuid).unwrap();
            if let Err(e) = agent.try_db_create(&manager.db_pool).await {
                if !e.to_string().contains("duplicate key") {
                    eprintln!("[ERROR] Failed to save agent to database: {}", e);
                    return Err(Status::internal("Failed to save agent to database"));
                }
            }

            // Add to local_id_map if the agent has a local_id
            if let Some(local_id) = agent.identifiers.local_id {
                let local_id_str = local_id.to_string();
                println!("[INFO] Adding mapping from local ID {} to UUID {}", local_id_str, agent_uuid);
                manager.local_id_map.insert(local_id_str, agent_uuid.clone());
            }

            Ok(GeneralResponse {
                success: true,
                message: format!("Agent {} created successfully", agent_uuid),
            })
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to parse agent JSON: {}", e);
            Err(Status::invalid_argument(format!(
                "Invalid agent data: {}",
                e
            )))
        }
    }
}
