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
    AgentRequest, OperationResponse, ServerInitRequest, ServerInitResponse, SignalRequest,
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

    async fn create_signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let data = request.into_inner();

        println!("[INFO] Received create_signal request");

        // Extract the record data from the SupabaseData structure
        let signal_json = match &data.data {
            Some(data) => match data.record.as_ref() {
                Some(record) => proto_struct_to_json(record),
                None => {
                    eprintln!("[ERROR] Create signal failed: Missing record in signal data");
                    return Err(Status::invalid_argument("Missing record in signal data"));
                }
            },
            None => {
                eprintln!("[ERROR] Create signal failed: Missing signal data");
                return Err(Status::invalid_argument("Missing signal data"));
            }
        };

        // Validate agent_id - required for signal processing as per system design
        // Signals must be associated with an Agent that will process them
        // See: server/scheme.hcl - signals.agent_id foreign key to agents table
        let agent_id = match signal_json.get("agent_id") {
            Some(id) => match id.as_str() {
                Some(id_str) if !id_str.is_empty() => id_str.to_string(),
                _ => {
                    eprintln!("[ERROR] Create signal failed: Invalid or empty agent_id in record");
                    return Err(Status::invalid_argument(
                        "Invalid or empty agent_id in record. Signals must be associated with an Agent for processing."
                    ));
                }
            },
            None => {
                eprintln!("[ERROR] Create signal failed: Missing agent_id field in record");
                return Err(Status::invalid_argument(
                    "Missing agent_id field in record. Signals must be associated with an Agent for processing."
                ));
            }
        };

        match &self.db_pool {
            Some(pool) => {
                let agent_map_clone = self.agent_map.clone();
                let pool_clone = pool.clone();
                let signal_json = signal_json.clone();

                tokio::spawn(async move {
                    handle_create_signal(signal_json, agent_id, agent_map_clone, pool_clone).await;
                });

                println!("[INFO] Signal processing initiated successfully");
                Ok(Response::new(OperationResponse {
                    success: true,
                    message: "Signal processing initiated".to_string(),
                }))
            }
            None => {
                eprintln!("[ERROR] Create signal failed: Database connection not available");
                Err(Status::unavailable("Database connection not available"))
            }
        }
    }

    async fn create_agent(
        &self,
        request: Request<AgentRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let data = request.into_inner();

        println!("[INFO] Received create_agent request");

        // Extract the record data from the SupabaseData structure
        let agent_json = match &data.data {
            Some(data) => match data.record.as_ref() {
                Some(record) => proto_struct_to_json(record),
                None => {
                    eprintln!("[ERROR] Create agent failed: Missing record in agent data");
                    return Err(Status::invalid_argument("Missing record in agent data"));
                }
            },
            None => {
                eprintln!("[ERROR] Create agent failed: Missing agent data");
                return Err(Status::invalid_argument("Missing agent data"));
            }
        };

        // Check if the agent already exists
        let global_uuid = match agent_json.get("global_uuid").and_then(|v| v.as_str()) {
            Some(uuid) if !uuid.is_empty() => uuid.to_string(),
            _ => {
                eprintln!("[ERROR] Create agent failed: Missing or invalid global_uuid in record");
                return Err(Status::invalid_argument("Missing or invalid global_uuid in record"));
            }
        };

        {
            let agents = self.agent_map.read().await;
            if agents.contains_key(&global_uuid) {
                eprintln!("[ERROR] Create agent failed: Agent with UUID {} already exists", global_uuid);
                return Err(Status::already_exists(format!("Agent with UUID {} already exists", global_uuid)));
            }
        }

        let agent_map_clone = self.agent_map.clone();
        let agent_json = agent_json.clone();

        tokio::spawn(async move {
            handle_create_agent(agent_json, agent_map_clone).await;
        });

        println!("[INFO] Agent creation initiated successfully");
        Ok(Response::new(OperationResponse {
            success: true,
            message: "Agent creation initiated".to_string(),
        }))
    }

    async fn update_agent(
        &self,
        request: Request<AgentRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let data = request.into_inner();

        println!("[INFO] Received update_agent request");

        // Extract the record data from the SupabaseData structure
        let agent_json = match &data.data {
            Some(data) => match data.record.as_ref() {
                Some(record) => proto_struct_to_json(record),
                None => {
                    eprintln!("[ERROR] Update agent failed: Missing record in agent data");
                    return Err(Status::invalid_argument("Missing record in agent data"));
                }
            },
            None => {
                eprintln!("[ERROR] Update agent failed: Missing agent data");
                return Err(Status::invalid_argument("Missing agent data"));
            }
        };

        let global_uuid = match agent_json.get("global_uuid").and_then(|v| v.as_str()) {
            Some(uuid) if !uuid.is_empty() => uuid.to_string(),
            _ => {
                eprintln!("[ERROR] Update agent failed: Missing or invalid global_uuid in record");
                return Err(Status::invalid_argument("Missing or invalid global_uuid in record"));
            }
        };

        {
            let agents = self.agent_map.read().await;
            if !agents.contains_key(&global_uuid) {
                eprintln!("[ERROR] Update agent failed: Agent with UUID {} not found", global_uuid);
                return Err(Status::not_found(format!("Agent with UUID {} not found", global_uuid)));
            }
        }

        let agent_map_clone = self.agent_map.clone();
        let agent_json = agent_json.clone();

        tokio::spawn(async move {
            handle_update_agent(agent_json, agent_map_clone).await;
        });

        println!("[INFO] Agent update initiated successfully");
        Ok(Response::new(OperationResponse {
            success: true,
            message: "Agent update initiated".to_string(),
        }))
    }

    async fn delete_agent(
        &self,
        request: Request<AgentRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let data = request.into_inner();

        println!("[INFO] Received delete_agent request");

        // Extract the record data from the SupabaseData structure
        let agent_json = match &data.data {
            Some(data) => match data.record.as_ref() {
                Some(record) => proto_struct_to_json(record),
                None => {
                    eprintln!("[ERROR] Delete agent failed: Missing record in agent data");
                    return Err(Status::invalid_argument("Missing record in agent data"));
                }
            },
            None => {
                eprintln!("[ERROR] Delete agent failed: Missing agent data");
                return Err(Status::invalid_argument("Missing agent data"));
            }
        };

        let global_uuid = match agent_json.get("global_uuid").and_then(|v| v.as_str()) {
            Some(uuid) if !uuid.is_empty() => uuid.to_string(),
            _ => {
                eprintln!("[ERROR] Delete agent failed: Missing or invalid global_uuid in record");
                return Err(Status::invalid_argument("Missing or invalid global_uuid in record"));
            }
        };

        let agent_map_clone = self.agent_map.clone();

        tokio::spawn(async move {
            handle_delete_agent(global_uuid, agent_map_clone).await;
        });

        println!("[INFO] Agent deletion initiated successfully");
        Ok(Response::new(OperationResponse {
            success: true,
            message: "Agent deletion initiated".to_string(),
        }))
    }
}

// Handler for CreateSignal messages
pub async fn handle_create_signal(signal_json: Value, agent_id: String, agent_map: AgentMap, pool: PgPool) {
    println!("[INFO] Processing Signal creation: {}", signal_json);

    // Find the agent and process it
    {
        let agents_guard = match agent_map.read().await {
            guard => guard,
        };

        if let Some(agent_ref) = agents_guard.get(&agent_id) {
            println!("[INFO] Found agent with ID: {}", agent_id);
            match agent_ref.run(signal_json).await {
                Ok(session) => {
                    println!("[INFO] Agent execution successful, saving session");
                    if let Err(e) = session.try_db_create(&pool).await {
                        eprintln!("[ERROR] Signal processing failed: Unable to save session to database: {}", e);
                        eprintln!("[DEBUG] Session data that failed to save: {:?}", session);
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Signal processing failed: Agent execution error: {}", e);
                    eprintln!("[DEBUG] Agent ID: {}", agent_id);
                }
            }
        } else {
            eprintln!("[ERROR] Signal processing failed: Agent with ID {} not found in agent map", agent_id);
            let available_agents: Vec<String> = agents_guard.keys().cloned().collect();
            eprintln!("[DEBUG] Available agents: {:?}", available_agents);
        }
    }
}

// Handler for CreateAgent messages
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

// Handler for UpdateAgent messages
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

// Handler for DeleteAgent messages
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
