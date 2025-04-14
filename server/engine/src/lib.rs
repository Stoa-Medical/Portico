use anyhow::Result;
use portico_shared::models::Agent;
use portico_shared::{DatabaseItem, JsonLike};
use prost_types::Struct;
use serde::{Deserialize, Serialize};
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

// Signal data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalData {
    pub id: String,
    pub agent_id: String,
    pub signal_type: String,
    pub content: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

// Agent data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentData {
    pub global_uuid: String,
    pub name: String,
    pub description: String,
    pub configuration: HashMap<String, String>,
}

// Extract data from Struct to a specific type
fn extract_struct_to<T: for<'a> Deserialize<'a>>(
    proto_struct: Option<Struct>,
) -> Result<T, Status> {
    match proto_struct {
        Some(s) => {
            // Convert Struct to serde_json::Value using our manual converter
            let json_value = proto_struct_to_json(&s);

            // Convert Value to target type
            match serde_json::from_value::<T>(json_value) {
                Ok(data) => Ok(data),
                Err(err) => Err(Status::internal(format!(
                    "Failed to deserialize JSON to target type: {}",
                    err
                ))),
            }
        }
        None => Err(Status::invalid_argument("Missing required data")),
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
            println!("Received init message from bridge service");

            Ok(Response::new(ServerInitResponse {
                success: true,
                message: "Bridge service initialized successfully".to_string(),
            }))
        } else {
            Err(Status::invalid_argument("Expected server_init to be true"))
        }
    }

    async fn create_signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let data = request.into_inner();

        let signal_data: SignalData = match &data.data {
            Some(data) => extract_struct_to(data.record.clone())?,
            None => return Err(Status::invalid_argument("Missing signal data")),
        };

        match &self.db_pool {
            Some(pool) => {
                let agent_map_clone = self.agent_map.clone();
                let pool_clone = pool.clone();

                tokio::spawn(async move {
                    handle_create_signal(signal_data, agent_map_clone, pool_clone).await;
                });

                Ok(Response::new(OperationResponse {
                    success: true,
                    message: "Signal processing initiated".to_string(),
                }))
            }
            None => Err(Status::unavailable("Database connection not available")),
        }
    }

    async fn create_agent(
        &self,
        request: Request<AgentRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let data = request.into_inner();

        let agent_data: AgentData = match &data.data {
            Some(data) => extract_struct_to(data.record.clone())?,
            None => return Err(Status::invalid_argument("Missing agent data")),
        };

        let agent_map_clone = self.agent_map.clone();
        tokio::spawn(async move {
            handle_create_agent(agent_data, agent_map_clone).await;
        });

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

        let agent_data: AgentData = match &data.data {
            Some(data) => extract_struct_to(data.record.clone())?,
            None => return Err(Status::invalid_argument("Missing agent data")),
        };

        let agent_map_clone = self.agent_map.clone();
        tokio::spawn(async move {
            handle_update_agent(agent_data, agent_map_clone).await;
        });

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

        let agent_data: AgentData = match &data.data {
            Some(data) => extract_struct_to(data.record.clone())?,
            None => return Err(Status::invalid_argument("Missing agent data")),
        };

        let agent_map_clone = self.agent_map.clone();
        tokio::spawn(async move {
            handle_delete_agent(agent_data, agent_map_clone).await;
        });

        Ok(Response::new(OperationResponse {
            success: true,
            message: "Agent deletion initiated".to_string(),
        }))
    }
}

// Handler for CreateSignal messages
pub async fn handle_create_signal(data: SignalData, agent_map: AgentMap, pool: PgPool) {
    println!("Processing Signal creation: {:?}", data);

    // Extract agent_id from signal data
    let agent_id = &data.agent_id;

    if agent_id.is_empty() {
        eprintln!("Signal missing agent_id field");
        return;
    }

    // Find the agent and process it
    {
        // Scope the read lock to this block
        let agents_guard = agent_map.read().await;

        // Get a reference to the agent
        if let Some(agent_ref) = agents_guard.get(agent_id) {
            // Convert SignalData to Value for compatibility with existing agent.run method
            let data_value = match serde_json::to_value(&data) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Failed to convert SignalData to Value: {}", e);
                    return;
                }
            };

            // Now we can use the immutable reference directly
            println!("Found agent with ID: {}", agent_id);
            match agent_ref.run(data_value).await {
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
pub async fn handle_create_agent(data: AgentData, agent_map: AgentMap) {
    println!("Processing Agent creation: {:?}", data);

    // Convert AgentData to Value for compatibility with existing from_json method
    let data_value = match serde_json::to_value(&data) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to convert AgentData to Value: {}", e);
            return;
        }
    };

    // Attempt to deserialize the Agent from JSON using the from_json method
    match portico_shared::models::agents::Agent::from_json(data_value) {
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
pub async fn handle_update_agent(data: AgentData, agent_map: AgentMap) {
    println!("Processing Agent update: {:?}", data);

    // Extract the UUID to identify which agent to update
    let uuid = &data.global_uuid;

    if uuid.is_empty() {
        eprintln!("Update Agent data missing valid global_uuid field");
        return;
    }

    println!("Updating agent with UUID: {}", uuid);

    // Convert AgentData to Value for compatibility with existing update_from_json method
    let data_value = match serde_json::to_value(&data) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Failed to convert AgentData to Value: {}", e);
            return;
        }
    };

    // Get write access to the agent map
    let mut agents = agent_map.write().await;

    if let Some(existing_agent) = agents.get_mut(uuid) {
        // Update the existing agent with the new data
        match existing_agent.update_from_json(data_value.clone()) {
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
        match portico_shared::models::agents::Agent::from_json(data_value) {
            Ok(new_agent) => {
                agents.insert(uuid.clone(), new_agent);
                println!("New agent inserted into map");
            }
            Err(e) => {
                eprintln!("Failed to deserialize agent for insert: {}", e);
            }
        }
    }
}

// Handler for DeleteAgent messages
pub async fn handle_delete_agent(data: AgentData, agent_map: AgentMap) {
    println!("Processing Agent deletion: {:?}", data);

    // Extract agent UUID
    let uuid = &data.global_uuid;

    if uuid.is_empty() {
        eprintln!("Delete Agent data missing valid global_uuid field");
        return;
    }

    println!("Removing agent with UUID: {}", uuid);

    // Get write access to the agent map and remove the agent
    let mut agents = agent_map.write().await;

    if agents.remove(uuid).is_some() {
        println!("Agent successfully removed from map");

        // Optionally, we could also delete from the database if needed
        // This would be useful if the bridge needs to manage database state
    } else {
        println!("Agent with UUID {} not found in map", uuid);
    }
}
