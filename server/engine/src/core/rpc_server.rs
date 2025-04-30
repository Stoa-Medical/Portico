use crate::core::agent_manager::AgentManager;
use crate::json_to_proto_struct;
use crate::proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use crate::proto::signal_request;
use crate::proto::SignalType;
use crate::proto::{
    CreateAgentRequest, DeleteAgentRequest, GeneralResponse, ServerInitRequest, SignalRequest,
    SignalResponse,
};
use crate::proto_struct_to_json;
use crate::SharedAgentMap;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// Bridge service implementation
pub struct RpcServer {
    agent_manager: Arc<tokio::sync::Mutex<AgentManager>>,
}

impl RpcServer {
    pub fn new(agent_map: SharedAgentMap, db_pool: PgPool) -> Self {
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
impl BridgeService for RpcServer {
    async fn init_server(
        &self,
        request: Request<ServerInitRequest>,
    ) -> Result<Response<GeneralResponse>, Status> {
        let server_init = request.into_inner().server_init;

        if server_init {
            println!("[INFO] Received init message from bridge service");

            Ok(Response::new(GeneralResponse {
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
            "[INFO] Received signal: type={:?}, signal_uuid={}",
            signal.signal_type(),
            signal.signal_uuid
        );

        // Process the signal using the agent manager
        let mut manager = self.agent_manager.lock().await;
        match manager.process_signal(signal).await {
            Ok(response) => Ok(Response::new(response)),
            Err(status) => Err(status),
        }
    }

    async fn create_agent(
        &self,
        request: Request<CreateAgentRequest>,
    ) -> Result<Response<GeneralResponse>, Status> {
        let agent_request = request.into_inner();

        println!("[INFO] Received create_agent request");

        if let Some(agent_json) = &agent_request.agent_json {
            let agent_data = proto_struct_to_json(agent_json);

            // Create a SignalRequest with RUN operation to reuse existing logic
            let mut run_data = serde_json::Map::new();
            run_data.insert("operation".to_string(), Value::String("CREATE".to_string()));
            run_data.insert(
                "entity_type".to_string(),
                Value::String("AGENT".to_string()),
            );
            run_data.insert("data".to_string(), agent_data);

            let signal_uuid = uuid::Uuid::new_v4().to_string();

            // Convert run_data to a proto struct
            let run_data_value = Value::Object(run_data);
            let run_data_struct = json_to_proto_struct(&run_data_value);

            // Create a SignalRequest
            let signal = SignalRequest {
                signal_uuid,
                agent_uuid: String::new(), // Not needed for creation
                signal_type: SignalType::Run as i32,
                payload: Some(signal_request::Payload::RunData(run_data_struct)),
            };

            // Process the signal
            let mut manager = self.agent_manager.lock().await;
            match manager.process_signal(signal).await {
                Ok(response) => {
                    println!("[INFO] Agent created successfully");
                    Ok(Response::new(GeneralResponse {
                        success: true,
                        message: format!("Agent created successfully: {}", response.message),
                    }))
                }
                Err(status) => {
                    eprintln!("[ERROR] Failed to create agent: {}", status);
                    Err(status)
                }
            }
        } else {
            Err(Status::invalid_argument(
                "Missing agent_json in CreateAgentRequest",
            ))
        }
    }

    async fn delete_agent(
        &self,
        request: Request<DeleteAgentRequest>,
    ) -> Result<Response<GeneralResponse>, Status> {
        let delete_request = request.into_inner();
        let agent_uuid = delete_request.agent_uuid;

        println!(
            "[INFO] Received delete_agent request for UUID: {}",
            agent_uuid
        );

        if agent_uuid.is_empty() {
            return Err(Status::invalid_argument(
                "Missing agent_uuid in DeleteAgentRequest",
            ));
        }

        // Create a SignalRequest with RUN operation to reuse existing logic
        let mut run_data = serde_json::Map::new();
        run_data.insert("operation".to_string(), Value::String("DELETE".to_string()));
        run_data.insert(
            "entity_type".to_string(),
            Value::String("AGENT".to_string()),
        );
        run_data.insert("entity_uuid".to_string(), Value::String(agent_uuid.clone()));

        // Convert run_data to a proto struct
        let run_data_value = Value::Object(run_data);
        let run_data_struct = json_to_proto_struct(&run_data_value);

        // Create a SignalRequest
        let signal = SignalRequest {
            signal_uuid: uuid::Uuid::new_v4().to_string(),
            agent_uuid: agent_uuid.clone(), // Use the agent_uuid from the request
            signal_type: SignalType::Run as i32,
            payload: Some(signal_request::Payload::RunData(run_data_struct)),
        };

        // Process the signal
        let mut manager = self.agent_manager.lock().await;
        match manager.process_signal(signal).await {
            Ok(response) => {
                println!("[INFO] Agent deleted successfully");
                Ok(Response::new(GeneralResponse {
                    success: true,
                    message: format!("Agent deleted successfully: {}", response.message),
                }))
            }
            Err(status) => {
                eprintln!("[ERROR] Failed to delete agent: {}", status);
                Err(status)
            }
        }
    }
}
