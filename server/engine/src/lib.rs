use anyhow::Result;
use portico_shared::models::Agent;
use portico_shared::{DatabaseItem, JsonLike};
use prost_types::Struct;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod proto {
    tonic::include_proto!("portico");
}

use proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use proto::{
    ServerInitRequest, ServerInitResponse, SignalRequest, SignalResponse,
    CommandOperation, EntityType, SignalType, SyncScope,
};

// Thread-safe Agent map type
pub type AgentMap = Arc<RwLock<HashMap<String, Agent>>>;

// Convert a protobuf Value to a serde_json::Value
fn proto_value_to_json(proto_value: &prost_types::Value) -> Value {
    match &proto_value.kind {
        Some(prost_types::value::Kind::NullValue(_)) => Value::Null,
        Some(prost_types::value::Kind::NumberValue(n)) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Value::Number(num)
            } else {
                Value::Null
            }
        }
        Some(prost_types::value::Kind::StringValue(s)) => Value::String(s.clone()),
        Some(prost_types::value::Kind::BoolValue(b)) => Value::Bool(*b),
        Some(prost_types::value::Kind::StructValue(s)) => {
            let mut map = serde_json::Map::new();
            for (k, v) in &s.fields {
                map.insert(k.clone(), proto_value_to_json(v));
            }
            Value::Object(map)
        }
        Some(prost_types::value::Kind::ListValue(l)) => {
            Value::Array(l.values.iter().map(proto_value_to_json).collect())
        }
        None => Value::Null,
    }
}

// Convert a protobuf Struct to a serde_json::Value
fn proto_struct_to_json(proto_struct: &Struct) -> Value {
    let mut map = serde_json::Map::new();
    for (k, v) in &proto_struct.fields {
        map.insert(k.clone(), proto_value_to_json(v));
    }
    Value::Object(map)
}

// Convert a serde_json::Value to a protobuf Value
fn json_to_proto_value(json_value: &Value) -> prost_types::Value {
    let kind = match json_value {
        Value::Null => prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue.into()),
        Value::Bool(b) => prost_types::value::Kind::BoolValue(*b),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                prost_types::value::Kind::NumberValue(f)
            } else {
                prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue.into())
            }
        },
        Value::String(s) => prost_types::value::Kind::StringValue(s.clone()),
        Value::Array(a) => {
            let values = a.iter().map(json_to_proto_value).collect();
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        },
        Value::Object(o) => {
            let mut fields = std::collections::BTreeMap::new();
            for (k, v) in o {
                fields.insert(k.clone(), json_to_proto_value(v));
            }
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        },
    };
    prost_types::Value { kind: Some(kind) }
}

// Convert a serde_json::Value to a protobuf Struct
fn json_to_proto_struct(json_value: &Value) -> prost_types::Struct {
    if let Value::Object(map) = json_value {
        let mut fields = std::collections::BTreeMap::new();
        for (k, v) in map {
            fields.insert(k.clone(), json_to_proto_value(v));
        }
        prost_types::Struct { fields }
    } else {
        prost_types::Struct {
            fields: std::collections::BTreeMap::new(),
        }
    }
}

// Bridge service implementation
pub struct BridgeServiceImpl {
    agent_map: AgentMap,
    db_pool: Option<PgPool>,
}

impl BridgeServiceImpl {
    pub fn new(agent_map: AgentMap, db_pool: Option<PgPool>) -> Self {
        Self { agent_map, db_pool }
    }

    pub fn with_server(self) -> BridgeServiceServer<Self> {
        BridgeServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl BridgeService for BridgeServiceImpl {
    async fn init_server(
        &self,
        request: Request<ServerInitRequest>,
    ) -> Result<Response<ServerInitResponse>, Status> {
        let server_init = request.into_inner().server_init;

        if server_init {
            println!("[INFO] Received init message from bridge service");

            Ok(Response::new(ServerInitResponse {
                success: true,
                message: "Bridge service initialized successfully".to_string(),
            }))
        } else {
            eprintln!("[ERROR] Server init failed: expected server_init to be true, got false");
            Err(Status::invalid_argument("Expected server_init to be true"))
        }
    }

    async fn process_signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<SignalResponse>, Status> {
        let signal = request.into_inner();

        println!("[INFO] Received signal: type={:?}, global_uuid={}",
            signal.signal_type(), signal.global_uuid);

        // Ensure we have a database connection
        if self.db_pool.is_none() {
            eprintln!("[ERROR] Process signal failed: Database connection not available");
            return Err(Status::unavailable("Database connection not available"));
        }

        let pool = self.db_pool.as_ref().unwrap().clone();
        let agent_map_clone = self.agent_map.clone();
        let runtime_session_uuid = uuid::Uuid::new_v4().to_string();

        // Process different signal types
        match signal.signal_type() {
            SignalType::Command => {
                if let Some(command) = signal.payload.as_ref().and_then(|p| match p {
                    proto::signal_request::Payload::Command(cmd) => Some(cmd),
                    _ => None,
                }) {
                    match command.operation() {
                        CommandOperation::Create => {
                            match command.entity_type() {
                                EntityType::Agent => {
                                    if let Some(data) = &command.data {
                                        let agent_json = proto_struct_to_json(data);

                                        // Spawn a task to handle the agent creation
                                        tokio::spawn(async move {
                                            handle_create_agent(agent_json, agent_map_clone).await;
                                        });

                                        Ok(Response::new(SignalResponse {
                                            success: true,
                                            message: "Agent creation initiated".to_string(),
                                            runtime_session_uuid,
                                            result_data: None,
                                        }))
                                    } else {
                                        Err(Status::invalid_argument("Missing agent data in command payload"))
                                    }
                                },
                                EntityType::Step => {
                                    // TODO: Implement step creation
                                    Err(Status::unimplemented("Step creation not implemented yet"))
                                },
                            }
                        },
                        CommandOperation::Update => {
                            match command.entity_type() {
                                EntityType::Agent => {
                                    if let Some(data) = &command.data {
                                        let agent_json = proto_struct_to_json(data);

                                        // Spawn a task to handle the agent update
                                        tokio::spawn(async move {
                                            handle_update_agent(agent_json, agent_map_clone).await;
                                        });

                                        Ok(Response::new(SignalResponse {
                                            success: true,
                                            message: "Agent update initiated".to_string(),
                                            runtime_session_uuid,
                                            result_data: None,
                                        }))
                                    } else {
                                        Err(Status::invalid_argument("Missing agent data in command payload"))
                                    }
                                },
                                EntityType::Step => {
                                    // TODO: Implement step update
                                    Err(Status::unimplemented("Step update not implemented yet"))
                                },
                            }
                        },
                        CommandOperation::Delete => {
                            match command.entity_type() {
                                EntityType::Agent => {
                                    let entity_uuid = command.entity_uuid.clone();
                                    if entity_uuid.is_empty() {
                                        return Err(Status::invalid_argument("Missing entity_uuid in command payload"));
                                    }

                                    // Spawn a task to handle the agent deletion
                                    tokio::spawn(async move {
                                        handle_delete_agent(entity_uuid, agent_map_clone).await;
                                    });

                                    Ok(Response::new(SignalResponse {
                                        success: true,
                                        message: "Agent deletion initiated".to_string(),
                                        runtime_session_uuid,
                                        result_data: None,
                                    }))
                                },
                                EntityType::Step => {
                                    // TODO: Implement step deletion
                                    Err(Status::unimplemented("Step deletion not implemented yet"))
                                },
                            }
                        },
                        CommandOperation::Run => {
                            match command.entity_type() {
                                EntityType::Agent => {
                                    let entity_uuid = command.entity_uuid.clone();
                                    if entity_uuid.is_empty() {
                                        return Err(Status::invalid_argument("Missing entity_uuid in command payload"));
                                    }

                                    // Implement agent run logic
                                    let agent_map_clone = self.agent_map.clone();
                                    let pool_clone = pool.clone();
                                    let run_data = match &command.data {
                                        Some(data) => proto_struct_to_json(data),
                                        None => Value::Object(serde_json::Map::new()),
                                    };

                                    // Clone entity_uuid for the response message
                                    let entity_uuid_for_response = entity_uuid.clone();

                                    // Run agent in a separate task and return immediate response
                                    tokio::spawn(async move {
                                        // Find the agent and process it
                                        let agents_guard = agent_map_clone.read().await;
                                        if let Some(agent) = agents_guard.get(&entity_uuid) {
                                            println!("[INFO] Running agent with UUID: {}", entity_uuid);
                                            match agent.run(run_data).await {
                                                Ok(session) => {
                                                    println!("[INFO] Agent execution successful, saving session");
                                                    if let Err(e) = session.try_db_create(&pool_clone).await {
                                                        eprintln!("[ERROR] Agent execution failed: Unable to save session to database: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("[ERROR] Agent execution failed: {}", e);
                                                }
                                            }
                                        } else {
                                            eprintln!("[ERROR] Agent execution failed: Agent with UUID {} not found", entity_uuid);
                                        }
                                    });

                                    Ok(Response::new(SignalResponse {
                                        success: true,
                                        message: format!("Agent {} run initiated", entity_uuid_for_response),
                                        runtime_session_uuid,
                                        result_data: None,
                                    }))
                                },
                                EntityType::Step => {
                                    // TODO: Implement step run
                                    Err(Status::unimplemented("Step run not implemented yet"))
                                },
                            }
                        },
                    }
                } else {
                    Err(Status::invalid_argument("Missing command payload for COMMAND signal type"))
                }
            },
            SignalType::Sync => {
                if let Some(sync) = signal.payload.as_ref().and_then(|p| match p {
                    proto::signal_request::Payload::Sync(sync) => Some(sync),
                    _ => None,
                }) {
                    // Handle the sync operation
                    match sync.scope() {
                        SyncScope::All => {
                            // TODO: Implement all entities sync
                            println!("[INFO] Syncing all entities");

                            // For now, simply return the current agents
                            let agents = self.agent_map.read().await;
                            let mut result_map = serde_json::Map::new();

                            for (uuid, agent) in agents.iter() {
                                let agent_json = agent.to_json();
                                result_map.insert(uuid.clone(), agent_json);
                            }

                            let result_value = Value::Object(result_map);

                            Ok(Response::new(SignalResponse {
                                success: true,
                                message: "All entities synced".to_string(),
                                runtime_session_uuid,
                                result_data: Some(json_to_proto_struct(&result_value)),
                            }))
                        },
                        SyncScope::Specific => {
                            // Check if specific UUIDs were provided
                            if sync.entity_uuids.is_empty() {
                                return Err(Status::invalid_argument("Missing entity_uuids for SPECIFIC sync scope"));
                            }

                            println!("[INFO] Syncing specific entities: {:?}", sync.entity_uuids);

                            // Sync specific entities by UUID
                            let agents = self.agent_map.read().await;
                            let mut result_map = serde_json::Map::new();

                            for uuid in &sync.entity_uuids {
                                if let Some(agent) = agents.get(uuid) {
                                    let agent_json = agent.to_json();
                                    result_map.insert(uuid.clone(), agent_json);
                                }
                            }

                            let result_value = Value::Object(result_map);

                            Ok(Response::new(SignalResponse {
                                success: true,
                                message: "Specific entities synced".to_string(),
                                runtime_session_uuid,
                                result_data: Some(json_to_proto_struct(&result_value)),
                            }))
                        },
                    }
                } else {
                    Err(Status::invalid_argument("Missing sync payload for SYNC signal type"))
                }
            },
            SignalType::Fyi => {
                if let Some(fyi_data) = signal.payload.as_ref().and_then(|p| match p {
                    proto::signal_request::Payload::FyiData(data) => Some(data),
                    _ => None,
                }) {
                    // Handle the FYI data
                    println!("[INFO] Received FYI data signal");

                    // Convert the data to JSON for processing
                    let fyi_json = proto_struct_to_json(fyi_data);

                    // For now, we'll just log the data and return success
                    println!("[INFO] FYI data: {}", fyi_json);

                    Ok(Response::new(SignalResponse {
                        success: true,
                        message: "FYI data received".to_string(),
                        runtime_session_uuid,
                        result_data: None,
                    }))
                } else {
                    Err(Status::invalid_argument("Missing FYI data for FYI signal type"))
                }
            },
        }
    }
}

// Helper functions
pub async fn handle_create_agent(agent_json: Value, agent_map: AgentMap) {
    println!("[INFO] Processing Agent creation: {}", agent_json);

    match Agent::from_json(agent_json.clone()) {
        Ok(agent) => {
            let agent_uuid = agent.identifiers.global_uuid.clone();
            println!("[INFO] Adding agent with UUID: {} to map", agent_uuid);

            let mut agents = agent_map.write().await;
            agents.insert(agent_uuid, agent);
            println!("[INFO] Agent added to map successfully");
        }
        Err(e) => {
            eprintln!("[ERROR] Agent creation failed: Unable to create Agent from JSON: {}", e);
            eprintln!("[DEBUG] JSON that failed to parse: {}", agent_json);
        }
    }
}

pub async fn handle_update_agent(agent_json: Value, agent_map: AgentMap) {
    println!("[INFO] Processing Agent update: {}", agent_json);

    let global_uuid = agent_json.get("global_uuid")
        .and_then(|v| v.as_str())
        .expect("global_uuid validation already performed");

    let mut agents = agent_map.write().await;
    if let Some(existing_agent) = agents.get_mut(global_uuid) {
        match existing_agent.update_from_json(agent_json.clone()) {
            Ok(updated_fields) => {
                if !updated_fields.is_empty() {
                    println!("[INFO] Agent updated: fields changed: {:?}", updated_fields);
                } else {
                    println!("[INFO] No fields were changed during agent update");
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Agent update failed: {}", e);
                eprintln!("[DEBUG] Agent UUID: {}, JSON: {}", global_uuid, agent_json);
            }
        }
    }
}

pub async fn handle_delete_agent(global_uuid: String, agent_map: AgentMap) {
    println!("[INFO] Processing Agent deletion for UUID: {}", global_uuid);

    let mut agents = agent_map.write().await;
    if agents.remove(&global_uuid).is_some() {
        println!("[INFO] Agent successfully removed from map");
    } else {
        eprintln!("[ERROR] Agent deletion failed: Agent with UUID {} not found in map", global_uuid);
        let available_agents: Vec<String> = agents.keys().cloned().collect();
        eprintln!("[DEBUG] Currently available agents: {:?}", available_agents);
    }
}
