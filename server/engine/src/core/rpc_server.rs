use crate::core::agent_manager::AgentManager;
use crate::proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use crate::proto::{
    CreateAgentRequest, DeleteAgentRequest, GeneralResponse, ServerInitRequest, SignalRequest,
    SignalResponse,
};
use crate::SharedAgentMap;
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
            // Use the create_agent handler directly
            let mut manager = self.agent_manager.lock().await;
            match crate::handlers::create::handle_create_agent(&mut *manager, agent_json).await {
                Ok(response) => {
                    println!("[INFO] Agent created successfully");
                    Ok(Response::new(response))
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

        // Use the delete_agent handler directly
        let mut manager = self.agent_manager.lock().await;
        match crate::handlers::delete::handle_delete_agent(&mut *manager, agent_uuid).await {
            Ok(response) => {
                println!("[INFO] Agent deleted successfully");
                Ok(Response::new(response))
            }
            Err(status) => {
                eprintln!("[ERROR] Failed to delete agent: {}", status);
                Err(status)
            }
        }
    }
}
