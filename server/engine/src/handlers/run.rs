use crate::core::agent_manager::AgentManager;
use crate::proto::signal_request;
use crate::proto::{SignalRequest, SignalResponse};
use crate::proto_struct_to_json;
use portico_shared::models::Agent;
use portico_shared::DatabaseItem;
use portico_shared::JsonLike;
use prost_types::Struct;
use serde_json;
use tonic::Status;

// Create operation handler
pub async fn handle_create(
    manager: &mut AgentManager,
    run_data: &Struct,
    runtime_session_uuid: String,
) -> Result<SignalResponse, Status> {
    let entity_type_opt = run_data
        .fields
        .get("entity_type")
        .and_then(|f| f.kind.as_ref())
        .and_then(|k| match k {
            prost_types::value::Kind::StringValue(s) => Some(s.clone()),
            _ => None,
        });

    let entity_type_str = match entity_type_opt {
        Some(val) => val,
        None => {
            return Err(Status::invalid_argument(
                "Missing entity_type field in run_data",
            ))
        }
    };

    match entity_type_str.as_str() {
        "AGENT" => {
            let data_struct = match run_data
                .fields
                .get("data")
                .and_then(|f| f.kind.as_ref())
                .and_then(|k| match k {
                    prost_types::value::Kind::StructValue(s) => Some(s),
                    _ => None,
                }) {
                Some(s) => s,
                None => {
                    return Err(Status::invalid_argument(
                        "Missing or invalid data field in run_data",
                    ))
                }
            };

            let mut agent_json = proto_struct_to_json(data_struct);

            // Check if global_uuid is present in the data, if not, use entity_uuid
            let entity_uuid = run_data
                .fields
                .get("entity_uuid")
                .and_then(|f| f.kind.as_ref())
                .and_then(|k| match k {
                    prost_types::value::Kind::StringValue(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            if !agent_json.get("global_uuid").is_some() && !entity_uuid.is_empty() {
                // Add entity_uuid as global_uuid if it doesn't exist
                if let serde_json::Value::Object(ref mut obj) = agent_json {
                    obj.insert(
                        "global_uuid".to_string(),
                        serde_json::Value::String(entity_uuid.clone()),
                    );
                    println!("[INFO] Using entity_uuid as global_uuid: {}", entity_uuid);
                }
            }

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

                    Ok(SignalResponse {
                        success: true,
                        message: format!("Agent {} created successfully", agent_uuid),
                        runtime_session_uuid,
                        result_data: None,
                    })
                }
                Err(e) => {
                    eprintln!("[ERROR] Agent creation failed: {}", e);
                    Err(Status::invalid_argument(format!(
                        "Invalid agent data: {}",
                        e
                    )))
                }
            }
        }
        "STEP" => {
            eprintln!("[ERROR] Step creation not implemented yet");
            Err(Status::unimplemented("Step creation not implemented yet"))
        }
        _ => Err(Status::invalid_argument(format!(
            "Invalid entity_type: {}",
            entity_type_str
        ))),
    }
}

// Delete operation handler
pub async fn handle_delete(
    manager: &mut AgentManager,
    run_data: &Struct,
    runtime_session_uuid: String,
) -> Result<SignalResponse, Status> {
    let entity_type_opt = run_data
        .fields
        .get("entity_type")
        .and_then(|f| f.kind.as_ref())
        .and_then(|k| match k {
            prost_types::value::Kind::StringValue(s) => Some(s.clone()),
            _ => None,
        });

    let entity_type_str = match entity_type_opt {
        Some(val) => val,
        None => {
            return Err(Status::invalid_argument(
                "Missing entity_type field in run_data",
            ))
        }
    };

    match entity_type_str.as_str() {
        "AGENT" => {
            let entity_uuid = run_data
                .fields
                .get("entity_uuid")
                .and_then(|f| f.kind.as_ref())
                .and_then(|k| match k {
                    prost_types::value::Kind::StringValue(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            if entity_uuid.is_empty() {
                return Err(Status::invalid_argument(
                    "Missing entity_uuid for delete operation",
                ));
            }

            println!("[INFO] Processing Agent deletion for UUID: {}", entity_uuid);

            // Remove the agent from the map
            let agent_existed = {
                let mut agents_guard = manager.agents.write().await;
                agents_guard.remove(&entity_uuid).is_some()
            };

            if !agent_existed {
                eprintln!(
                    "[ERROR] Agent deletion failed: Agent with UUID {} not found in memory",
                    entity_uuid
                );
                return Err(Status::not_found(format!(
                    "Agent with UUID {} not found",
                    entity_uuid
                )));
            }

            // Remove the message queue
            manager.message_queues.remove(&entity_uuid);

            // First delete associated steps for the agent
            if let Err(e) = sqlx::query("DELETE FROM steps WHERE agent_id IN (SELECT id FROM agents WHERE global_uuid = $1)")
                .bind(&entity_uuid)
                .execute(&manager.db_pool)
                .await {
                eprintln!("[ERROR] Failed to delete agent's steps from database: {}", e);
                return Err(Status::internal("Failed to delete agent's steps from database"));
            }

            // Then delete the agent itself
            if let Err(e) = sqlx::query("DELETE FROM agents WHERE global_uuid = $1")
                .bind(&entity_uuid)
                .execute(&manager.db_pool)
                .await
            {
                eprintln!("[ERROR] Failed to delete agent from database: {}", e);
                return Err(Status::internal("Failed to delete agent from database"));
            }

            println!("[INFO] Agent successfully removed");
            Ok(SignalResponse {
                success: true,
                message: format!("Agent {} deleted successfully", entity_uuid),
                runtime_session_uuid,
                result_data: None,
            })
        }
        "STEP" => {
            eprintln!("[ERROR] Step deletion not implemented yet");
            Err(Status::unimplemented("Step deletion not implemented yet"))
        }
        _ => Err(Status::invalid_argument(format!(
            "Invalid entity_type: {}",
            entity_type_str
        ))),
    }
}

// Run operation handler
pub async fn handle_run(
    manager: &AgentManager,
    signal: SignalRequest,
    runtime_session_uuid: String,
) -> Result<SignalResponse, Status> {
    // Process run signal with original field names (signal_uuid, agent_uuid)
    println!(
        "[INFO] Processing run operation for signal: {}",
        signal.signal_uuid
    );

    if let Some(signal_request::Payload::RunData(run_data)) = &signal.payload {
        // Use agent_uuid as the agent UUID, or attempt to extract from run_data if not present
        let agent_uuid = if !signal.agent_uuid.is_empty() {
            signal.agent_uuid.clone()
        } else {
            run_data
                .fields
                .get("entity_uuid")
                .and_then(|f| f.kind.as_ref())
                .and_then(|k| match k {
                    prost_types::value::Kind::StringValue(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };

        if agent_uuid.is_empty() {
            return Err(Status::invalid_argument(
                "Missing agent_uuid for RUN operation",
            ));
        }

        // Forward the signal to the agent's queue if it exists
        if let Some(queue) = manager.message_queues.get(&agent_uuid) {
            if let Err(e) = queue.send(signal.clone()).await {
                eprintln!("[ERROR] Failed to send signal to agent queue: {}", e);
                return Err(Status::internal("Failed to forward signal to agent queue"));
            }

            Ok(SignalResponse {
                success: true,
                message: format!("Signal forwarded to agent {}", agent_uuid),
                runtime_session_uuid,
                result_data: None,
            })
        } else {
            eprintln!("[ERROR] No queue found for agent: {}", agent_uuid);
            Err(Status::not_found(format!(
                "Agent with UUID {} not found",
                agent_uuid
            )))
        }
    } else {
        Err(Status::invalid_argument(
            "Missing run_data for RUN signal type",
        ))
    }
}
