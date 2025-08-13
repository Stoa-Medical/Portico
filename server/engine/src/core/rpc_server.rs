use crate::core::workflow_manager::WorkflowManager;
use crate::handlers::{create, delete};
use crate::proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use crate::proto::{
    CreateWorkflowRequest, DeleteWorkflowRequest, GeneralResponse, PlanWorkflowRequest,
    PlanWorkflowResponse, ServerInitRequest, SignalRequest, SignalResponse,
};
use crate::SharedWorkflowMap;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// Bridge service implementation
pub struct RpcServer {
    workflow_manager: Arc<tokio::sync::Mutex<WorkflowManager>>,
}

impl RpcServer {
    pub fn new(workflow_map: SharedWorkflowMap, db_pool: PgPool) -> Self {
        let workflow_manager = Arc::new(tokio::sync::Mutex::new(WorkflowManager::new(
            workflow_map, db_pool,
        )));

        let instance = Self { workflow_manager };

        // Initialize workflow queues in the background
        let manager_clone = Arc::clone(&instance.workflow_manager);
        tokio::spawn(async move {
            if let Err(e) = manager_clone.lock().await.init_workflow_queues().await {
                eprintln!("[ERROR] Failed to initialize workflow queues: {}", e);
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

            let workflow_count = self.workflow_manager.lock().await.workflow_count().await;

            let reply = GeneralResponse {
                success: true,
                message: format!("Engine initialized successfully with {} workflows", workflow_count),
            };

            Ok(Response::new(reply))
        } else {
            let reply = GeneralResponse {
                success: false,
                message: "Invalid server initialization request".to_string(),
            };

            Ok(Response::new(reply))
        }
    }

    async fn process_signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<SignalResponse>, Status> {
        let signal_request = request.into_inner();
        println!(
            "[INFO] Received signal for workflow_id: {} with type: {:?}",
            signal_request.workflow_id,
            signal_request.signal_type()
        );

        // Queue the signal through the workflow manager
        let response = self
            .workflow_manager
            .lock()
            .await
            .queue_signal(signal_request)
            .await?;

        Ok(Response::new(response))
    }

    async fn create_workflow(
        &self,
        request: Request<CreateWorkflowRequest>,
    ) -> Result<Response<GeneralResponse>, Status> {
        let create_request = request.into_inner();
        println!("[INFO] Received create workflow request");

        // Handle the workflow creation through the create handler
        let mut manager = self.workflow_manager.lock().await;
        let response = create::handle_create_workflow(&mut manager, &create_request.workflow_json).await?;

        Ok(Response::new(response))
    }

    async fn delete_workflow(
        &self,
        request: Request<DeleteWorkflowRequest>,
    ) -> Result<Response<GeneralResponse>, Status> {
        let delete_request = request.into_inner();
        println!(
            "[INFO] Received delete workflow request for workflow_id: {}",
            delete_request.workflow_id
        );

        // Handle the workflow deletion through the delete handler
        let mut manager = self.workflow_manager.lock().await;
        let response = delete::handle_delete_workflow(&mut manager, delete_request.workflow_id).await?;

        Ok(Response::new(response))
    }

    async fn plan_workflow(
        &self,
        request: Request<crate::proto::PlanWorkflowRequest>,
    ) -> Result<Response<crate::proto::PlanWorkflowResponse>, Status> {
        let plan_request = request.into_inner();
        println!(
            "[INFO] Received plan workflow request from agent_id: {} with objective: '{}'",
            plan_request.agent_id,
            plan_request.objective
        );

        // For now, create a simple response indicating the request was received
        // In a full implementation, this would:
        // 1. Load the agent from database
        // 2. Use the WorkflowPlanner to create workflow spec
        // 3. Create and save the workflow
        // 4. Return detailed response with workflow UUID and validation results

        let response = crate::proto::PlanWorkflowResponse {
            success: true,
            message: format!(
                "Plan workflow request received for agent {} with objective: '{}'",
                plan_request.agent_id, plan_request.objective
            ),
            workflow_spec: None, // Would contain the actual workflow specification
            validation_errors: vec![], // Would contain any validation errors
            estimated_steps: 1, // Would contain actual step count
            requires_approval: false, // Would be determined by agent policy
            workflow_uuid: String::new(), // Would contain UUID if workflow was created
        };

        println!("[INFO] Plan workflow request processed: {}", response.message);
        Ok(Response::new(response))
    }
}
