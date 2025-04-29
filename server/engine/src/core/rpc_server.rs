use crate::core::agent_manager::AgentManager;
use crate::proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use crate::proto::{ServerInitRequest, ServerInitResponse, SignalRequest, SignalResponse};
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
