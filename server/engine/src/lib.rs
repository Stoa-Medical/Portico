use anyhow::Result;
use portico_shared::models::Agent;
use portico_shared::{DatabaseItem, JsonLike};
use prost_types::Struct;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod proto {
    tonic::include_proto!("portico");
}

use proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use proto::{
    CommandOperation, EntityType, ServerInitRequest, ServerInitResponse, SignalRequest,
    SignalResponse, SignalType, SyncScope,
};

// Thread-safe Agent map type
pub type AgentMap = Arc<RwLock<HashMap<String, Agent>>>;

// Agent manager handles message queuing and processing
pub struct AgentManager {
    agents: AgentMap,
    message_queues: HashMap<String, mpsc::Sender<SignalRequest>>,
    db_pool: PgPool,
}

impl AgentManager {
    pub fn new(agents: AgentMap, db_pool: PgPool) -> Self {
        Self {
            agents,
            message_queues: HashMap::new(),
            db_pool,
        }
    }

    // Set up message queues for all existing agents
    pub async fn init_agent_queues(&mut self) -> Result<(), Status> {
        // Collect all agent UUIDs first to avoid borrowing conflicts
        let agent_uuids: Vec<String> = {
            let agents = self.agents.read().await;
            println!(
                "[INFO] Initializing message queues for {} existing agents",
                agents.len()
            );
            agents.keys().cloned().collect()
        };

        for agent_uuid in agent_uuids {
            if let Err(e) = self.setup_agent_queue(agent_uuid.clone()).await {
                eprintln!(
                    "[ERROR] Failed to initialize queue for agent {}: {}",
                    agent_uuid, e
                );
            }
        }

        Ok(())
    }

    // Process a new SignalRequest coming from gRPC
    pub async fn process_signal(
        &mut self,
        signal: SignalRequest,
    ) -> Result<SignalResponse, Status> {
        let runtime_session_uuid = uuid::Uuid::new_v4().to_string();

        match signal.signal_type() {
            SignalType::Command => {
                if let Some(proto::signal_request::Payload::Command(cmd)) = &signal.payload {
                    match cmd.operation() {
                        CommandOperation::Create => {
                            self.handle_create(cmd, runtime_session_uuid).await
                        }
                        CommandOperation::Update => {
                            self.handle_update(cmd, runtime_session_uuid).await
                        }
                        CommandOperation::Delete => {
                            self.handle_delete(cmd, runtime_session_uuid).await
                        }
                        CommandOperation::Run => {
                            self.handle_run(signal.clone(), runtime_session_uuid).await
                        }
                    }
                } else {
                    Err(Status::invalid_argument(
                        "Missing command payload for COMMAND signal type",
                    ))
                }
            }
            SignalType::Sync => self.handle_sync(&signal, runtime_session_uuid).await,
            SignalType::Fyi => self.handle_fyi(&signal, runtime_session_uuid).await,
        }
    }

    // Set up processing for a specific agent
    async fn setup_agent_queue(&mut self, agent_uuid: String) -> Result<(), Status> {
        // Check if queue already exists
        if self.message_queues.contains_key(&agent_uuid) {
            return Ok(());
        }

        println!("[INFO] Setting up message queue for agent {}", agent_uuid);

        // Create a channel for this agent
        let (tx, mut rx) = mpsc::channel::<SignalRequest>(32);
        self.message_queues.insert(agent_uuid.clone(), tx);

        // Clone shared resources for the worker task
        let agents = Arc::clone(&self.agents);
        let db_pool = self.db_pool.clone();

        // Spawn a dedicated worker for this agent
        tokio::spawn(async move {
            println!("[INFO] Started worker for agent {}", agent_uuid);
            while let Some(signal) = rx.recv().await {
                println!(
                    "[INFO] Agent {} worker processing signal type {:?}",
                    agent_uuid,
                    signal.signal_type()
                );

                if let SignalType::Command = signal.signal_type() {
                    if let Some(proto::signal_request::Payload::Command(cmd)) = &signal.payload {
                        if cmd.entity_type() == EntityType::Agent
                            && cmd.operation() == CommandOperation::Run
                        {
                            if let Some(data) = &cmd.data {
                                let run_data = proto_struct_to_json(data);
                                let agents_guard = agents.read().await;

                                if let Some(agent) = agents_guard.get(&agent_uuid) {
                                    println!("[INFO] Running agent {} with data", agent_uuid);
                                    match agent.run(run_data).await {
                                        Ok(session) => {
                                            println!(
                                                "[INFO] Agent execution successful, saving session"
                                            );
                                            if let Err(e) = session.try_db_create(&db_pool).await {
                                                eprintln!("[ERROR] Failed to save session: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[ERROR] Agent execution failed: {}", e);
                                        }
                                    }
                                } else {
                                    eprintln!("[ERROR] Agent {} not found in map", agent_uuid);
                                }
                            }
                        }
                    }
                }
            }
            println!("[INFO] Worker for agent {} shutting down", agent_uuid);
        });

        Ok(())
    }

    // Create operation handler
    async fn handle_create(
        &mut self,
        cmd: &proto::CommandPayload,
        runtime_session_uuid: String,
    ) -> Result<SignalResponse, Status> {
        match cmd.entity_type() {
            EntityType::Agent => {
                if let Some(data) = &cmd.data {
                    let mut agent_json = proto_struct_to_json(data);

                    // Check if global_uuid is present in the data, if not, use entity_uuid
                    if !agent_json.get("global_uuid").is_some() && !cmd.entity_uuid.is_empty() {
                        // Add entity_uuid as global_uuid if it doesn't exist
                        if let serde_json::Value::Object(ref mut obj) = agent_json {
                            obj.insert("global_uuid".to_string(), serde_json::Value::String(cmd.entity_uuid.clone()));
                            println!("[INFO] Using entity_uuid as global_uuid: {}", cmd.entity_uuid);
                        }
                    }

                    println!("[INFO] Processing Agent creation: {}", agent_json);

                    match Agent::from_json(agent_json.clone()) {
                        Ok(agent) => {
                            let agent_uuid = agent.identifiers.global_uuid.clone();
                            println!("[INFO] Adding agent with UUID: {}", agent_uuid);

                            // Setup a queue for this agent
                            if let Err(e) = self.setup_agent_queue(agent_uuid.clone()).await {
                                eprintln!("[ERROR] Failed to setup agent queue: {}", e);
                                return Err(Status::internal(
                                    "Failed to setup message queue for agent",
                                ));
                            }

                            // Store the agent
                            let mut agents_guard = self.agents.write().await;
                            agents_guard.insert(agent_uuid.clone(), agent);

                            // Save to database if not already there
                            let agent = agents_guard.get(&agent_uuid).unwrap();
                            if let Err(e) = agent.try_db_create(&self.db_pool).await {
                                if !e.to_string().contains("duplicate key") {
                                    eprintln!("[ERROR] Failed to save agent to database: {}", e);
                                    return Err(Status::internal(
                                        "Failed to save agent to database",
                                    ));
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
                } else {
                    Err(Status::invalid_argument("Missing agent data"))
                }
            }
            EntityType::Step => {
                eprintln!("[ERROR] Step creation not implemented yet");
                Err(Status::unimplemented("Step creation not implemented yet"))
            }
        }
    }

    // Update operation handler
    async fn handle_update(
        &mut self,
        cmd: &proto::CommandPayload,
        runtime_session_uuid: String,
    ) -> Result<SignalResponse, Status> {
        match cmd.entity_type() {
            EntityType::Agent => {
                if let Some(data) = &cmd.data {
                    let agent_json = proto_struct_to_json(data);

                    println!("[INFO] Processing Agent update: {}", agent_json);

                    let global_uuid = agent_json
                        .get("global_uuid")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            Status::invalid_argument("Missing global_uuid in agent data")
                        })?;

                    // Ensure we have a queue for this agent
                    if let Err(e) = self.setup_agent_queue(global_uuid.to_string()).await {
                        eprintln!("[ERROR] Failed to setup agent queue: {}", e);
                    }

                    let mut agents_guard = self.agents.write().await;
                    if let Some(existing_agent) = agents_guard.get_mut(global_uuid) {
                        match existing_agent.update_from_json(agent_json.clone()) {
                            Ok(updated_fields) => {
                                // Save changes to database
                                if let Err(e) = existing_agent.try_db_update(&self.db_pool).await {
                                    eprintln!("[ERROR] Failed to update agent in database: {}", e);
                                    return Err(Status::internal(
                                        "Failed to update agent in database",
                                    ));
                                }

                                if !updated_fields.is_empty() {
                                    println!(
                                        "[INFO] Agent updated: fields changed: {:?}",
                                        updated_fields
                                    );
                                    Ok(SignalResponse {
                                        success: true,
                                        message: format!(
                                            "Agent {} updated successfully. Fields changed: {:?}",
                                            global_uuid, updated_fields
                                        ),
                                        runtime_session_uuid,
                                        result_data: None,
                                    })
                                } else {
                                    println!("[INFO] No fields were changed during agent update");
                                    Ok(SignalResponse {
                                        success: true,
                                        message: format!(
                                            "Agent {} update: no fields changed",
                                            global_uuid
                                        ),
                                        runtime_session_uuid,
                                        result_data: None,
                                    })
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Agent update failed: {}", e);
                                Err(Status::invalid_argument(format!(
                                    "Invalid agent update data: {}",
                                    e
                                )))
                            }
                        }
                    } else {
                        eprintln!(
                            "[ERROR] Agent update failed: Agent with UUID {} not found",
                            global_uuid
                        );
                        Err(Status::not_found(format!(
                            "Agent with UUID {} not found",
                            global_uuid
                        )))
                    }
                } else {
                    Err(Status::invalid_argument("Missing agent data"))
                }
            }
            EntityType::Step => {
                eprintln!("[ERROR] Step update not implemented yet");
                Err(Status::unimplemented("Step update not implemented yet"))
            }
        }
    }

    // Delete operation handler
    async fn handle_delete(
        &mut self,
        cmd: &proto::CommandPayload,
        runtime_session_uuid: String,
    ) -> Result<SignalResponse, Status> {
        match cmd.entity_type() {
            EntityType::Agent => {
                let entity_uuid = cmd.entity_uuid.clone();
                if entity_uuid.is_empty() {
                    return Err(Status::invalid_argument(
                        "Missing entity_uuid for delete operation",
                    ));
                }

                println!("[INFO] Processing Agent deletion for UUID: {}", entity_uuid);

                // Remove the agent from the map
                let agent_existed = {
                    let mut agents_guard = self.agents.write().await;
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
                self.message_queues.remove(&entity_uuid);

                // First delete associated steps for the agent
                if let Err(e) = sqlx::query("DELETE FROM steps WHERE agent_id IN (SELECT id FROM agents WHERE global_uuid = $1)")
                    .bind(&entity_uuid)
                    .execute(&self.db_pool)
                    .await {
                    eprintln!("[ERROR] Failed to delete agent's steps from database: {}", e);
                    return Err(Status::internal("Failed to delete agent's steps from database"));
                }

                // Then delete the agent itself
                if let Err(e) = sqlx::query("DELETE FROM agents WHERE global_uuid = $1")
                    .bind(&entity_uuid)
                    .execute(&self.db_pool)
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
            EntityType::Step => {
                eprintln!("[ERROR] Step deletion not implemented yet");
                Err(Status::unimplemented("Step deletion not implemented yet"))
            }
        }
    }

    // Run operation handler - enqueues the run operation to the agent's queue
    async fn handle_run(
        &self,
        signal: SignalRequest,
        runtime_session_uuid: String,
    ) -> Result<SignalResponse, Status> {
        if let Some(proto::signal_request::Payload::Command(cmd)) = &signal.payload {
            let agent_uuid = cmd.entity_uuid.clone();
            if agent_uuid.is_empty() {
                return Err(Status::invalid_argument(
                    "Missing entity_uuid for run operation",
                ));
            }

            println!("[INFO] Queueing run operation for agent {}", agent_uuid);

            // Check if agent exists
            let agents_exist = {
                let agents_guard = self.agents.read().await;
                agents_guard.contains_key(&agent_uuid)
            };

            if !agents_exist {
                eprintln!(
                    "[ERROR] Agent run failed: Agent with UUID {} not found",
                    agent_uuid
                );
                return Err(Status::not_found(format!(
                    "Agent with UUID {} not found",
                    agent_uuid
                )));
            }

            // Send to the agent's queue
            if let Some(tx) = self.message_queues.get(&agent_uuid) {
                match tx.send(signal).await {
                    Ok(_) => {
                        println!("[INFO] Run operation queued for agent {}", agent_uuid);
                        Ok(SignalResponse {
                            success: true,
                            message: format!("Run operation queued for agent {}", agent_uuid),
                            runtime_session_uuid,
                            result_data: None,
                        })
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to send run command to agent queue: {}", e);
                        Err(Status::internal("Failed to queue run operation"))
                    }
                }
            } else {
                eprintln!("[ERROR] Agent queue for UUID {} not found", agent_uuid);
                Err(Status::internal(format!(
                    "Message queue for agent {} not found",
                    agent_uuid
                )))
            }
        } else {
            Err(Status::invalid_argument("Missing command payload"))
        }
    }

    // Sync operation handler
    async fn handle_sync(
        &self,
        signal: &SignalRequest,
        runtime_session_uuid: String,
    ) -> Result<SignalResponse, Status> {
        println!("[INFO] Processing sync operation");

        if let Some(proto::signal_request::Payload::Sync(sync)) = &signal.payload {
            match sync.scope() {
                SyncScope::All => {
                    println!("[INFO] Syncing all entities");

                    // Return the current agents
                    let agents = self.agents.read().await;
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
                    let agents = self.agents.read().await;
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

    // FYI operation handler
    async fn handle_fyi(
        &self,
        signal: &SignalRequest,
        runtime_session_uuid: String,
    ) -> Result<SignalResponse, Status> {
        if let Some(proto::signal_request::Payload::FyiData(fyi_data)) = &signal.payload {
            // Handle the FYI data
            println!("[INFO] Received FYI data signal");

            // Convert the data to JSON for processing
            let fyi_json = proto_struct_to_json(fyi_data);

            // For now, we'll just log the data and return success
            println!("[INFO] FYI data: {}", fyi_json);

            Ok(SignalResponse {
                success: true,
                message: "FYI data received".to_string(),
                runtime_session_uuid,
                result_data: None,
            })
        } else {
            Err(Status::invalid_argument(
                "Missing FYI data for FYI signal type",
            ))
        }
    }
}

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
        Value::Null => {
            prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue.into())
        }
        Value::Bool(b) => prost_types::value::Kind::BoolValue(*b),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                prost_types::value::Kind::NumberValue(f)
            } else {
                prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue.into())
            }
        }
        Value::String(s) => prost_types::value::Kind::StringValue(s.clone()),
        Value::Array(a) => {
            let values = a.iter().map(json_to_proto_value).collect();
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        }
        Value::Object(o) => {
            let mut fields = std::collections::BTreeMap::new();
            for (k, v) in o {
                fields.insert(k.clone(), json_to_proto_value(v));
            }
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
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
    agent_manager: Arc<tokio::sync::Mutex<AgentManager>>,
}

impl BridgeServiceImpl {
    pub fn new(agent_map: AgentMap, db_pool: PgPool) -> Self {
        let agent_manager = Arc::new(tokio::sync::Mutex::new(AgentManager::new(
            agent_map, db_pool,
        )));

        let instance = Self { agent_manager };

        // Initialize agent queues in the background
        let manager_clone = Arc::clone(&instance.agent_manager);
        tokio::spawn(async move {
            if let Err(e) = manager_clone.lock().await.init_agent_queues().await {
                eprintln!("[ERROR] Failed to initialize agent queues: {}", e);
            }
        });

        instance
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

        println!(
            "[INFO] Received signal: type={:?}, global_uuid={}",
            signal.signal_type(),
            signal.global_uuid
        );

        // Process the signal using the agent manager
        let mut manager = self.agent_manager.lock().await;
        match manager.process_signal(signal).await {
            Ok(response) => Ok(Response::new(response)),
            Err(status) => Err(status),
        }
    }
}
