use crate::core::workflow_manager::WorkflowManager;
use crate::handlers::{create, delete};
use crate::services::workflow_planner::WorkflowPlannerService;
use crate::services::simple_workflow_planner::SimpleWorkflowPlannerService;
use crate::services::agent_monitoring::AgentMonitoringService;
use crate::services::agent_cache::AgentCacheService;
use crate::services::request_batcher::{RequestBatcherService, BatchConfig};
use crate::proto::bridge_service_server::{BridgeService, BridgeServiceServer};
use crate::proto::{
    CreateWorkflowRequest, DeleteWorkflowRequest, GeneralResponse,
    ServerInitRequest, SignalRequest, SignalResponse,
};
use crate::SharedWorkflowMap;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tonic::{Request, Response, Status};
use uuid;

// Bridge service implementation
pub struct RpcServer {
    workflow_manager: Arc<tokio::sync::Mutex<WorkflowManager>>,
    workflow_planner: Arc<tokio::sync::Mutex<WorkflowPlannerService>>,
    simple_planner: SimpleWorkflowPlannerService,  // Keep for fallback
    agent_monitor: Arc<AgentMonitoringService>,
    agent_cache: Arc<AgentCacheService>,
    request_batcher: Arc<RequestBatcherService>,
}

impl RpcServer {
    pub fn new(workflow_map: SharedWorkflowMap, db_pool: PgPool) -> Self {
        let workflow_manager = Arc::new(tokio::sync::Mutex::new(WorkflowManager::new(
            workflow_map, db_pool.clone(),
        )));

        let workflow_planner = Arc::new(tokio::sync::Mutex::new(WorkflowPlannerService::new(db_pool.clone())));
        let simple_planner = SimpleWorkflowPlannerService::new();
        let agent_monitor = Arc::new(AgentMonitoringService::new());
        let agent_cache = Arc::new(AgentCacheService::new(1000, 3600)); // 1000 entries, 1 hour TTL
        let request_batcher = Arc::new(RequestBatcherService::new(BatchConfig::default()));

        let instance = Self {
            workflow_manager,
            workflow_planner,
            simple_planner,
            agent_monitor: agent_monitor.clone(),
            agent_cache: agent_cache.clone(),
            request_batcher,
        };

        // Start background monitoring cleanup task
        let monitor_clone = Arc::clone(&agent_monitor);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Every hour
            loop {
                interval.tick().await;
                monitor_clone.cleanup_old_metrics().await;
            }
        });

        // Start background cache cleanup task
        let cache_clone = Arc::clone(&agent_cache);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800)); // Every 30 minutes
            loop {
                interval.tick().await;
                cache_clone.cleanup_expired().await;
            }
        });

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
        let Some(ref workflow_json) = create_request.workflow_json else {
            return Err(Status::invalid_argument("workflow_json is required"));
        };
        let response = create::handle_create_workflow(&mut manager, workflow_json).await?;

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
        let start_time = Instant::now();

        println!(
            "[INFO] Received plan workflow request from agent_id: {} with objective: '{}'",
            plan_request.agent_id,
            plan_request.objective
        );

        // Convert protobuf context to JSON
        let context = plan_request.context.map(|ctx| crate::proto_struct_to_json(&ctx));
        let constraints = plan_request.constraints.map(|cons| crate::proto_struct_to_json(&cons));

        // Extract tools that might be used (simple analysis from objective)
        let tools_used = self.extract_tools_from_objective(&plan_request.objective);

        // Try DB-backed planner first, fallback to simple planner
        let use_db_planner = false; // Feature flag to switch between planners (disabled due to DB compilation issues)

        let result = if use_db_planner {
            // Use DB-backed WorkflowPlannerService
            let mut planner = self.workflow_planner.lock().await;
            let planning_result = planner.plan_workflow_for_agent(
                plan_request.agent_id,
                plan_request.objective.clone(),
                context.clone(),
                constraints.clone(),
                plan_request.is_ephemeral,
            ).await;

            // Convert to SimpleWorkflowPlan format for consistent handling
            planning_result.map(|result| {
                crate::services::simple_workflow_planner::SimpleWorkflowPlan {
                    success: result.validation_result.is_valid,
                    message: if result.validation_result.is_valid {
                        format!("Workflow planned successfully with {} steps", result.plan_response.workflow_spec.steps.len())
                    } else {
                        format!("Workflow validation failed: {:?}", result.validation_result.errors.join(", "))
                    },
                    workflow_spec: Some(serde_json::to_value(&result.plan_response.workflow_spec).unwrap_or_default()),
                    validation_errors: result.validation_result.errors,
                    estimated_steps: result.plan_response.workflow_spec.steps.len() as u32,
                    requires_approval: result.validation_result.requires_approval,
                    workflow_uuid: uuid::Uuid::new_v4().to_string(),
                }
            }).map_err(|e| Status::internal(format!("Planning failed: {}", e)))
        } else {
            // Fallback to simple planner
            self.simple_planner.plan_workflow(
                plan_request.agent_id,
                plan_request.objective.clone(),
                context,
                constraints,
                plan_request.is_ephemeral,
            ).await
        };

        let planning_duration = start_time.elapsed();

        match result {
            Ok(plan) => {
                // Record successful planning metrics
                self.agent_monitor.record_planning_attempt(
                    plan_request.agent_id,
                    format!("Agent {}", plan_request.agent_id), // Would get real name from database
                    plan_request.objective.clone(),
                    planning_duration,
                    plan.success,
                    plan.estimated_steps,
                    plan.requires_approval,
                    None,
                    tools_used,
                ).await;

                // If planning was successful and has a workflow spec, persist the workflow
                let final_uuid = if plan.success && plan.workflow_spec.is_some() {
                    // Convert workflow spec to protobuf Struct for persistence
                    let spec_value = serde_json::to_value(&plan.workflow_spec).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    let spec_struct = crate::json_to_proto_struct(&spec_value);

                    // Create workflow through existing handler
                    let mut manager = self.workflow_manager.lock().await;
                    match create::handle_create_workflow(&mut manager, &spec_struct).await {
                        Ok(create_response) => {
                            // Extract workflow UUID from creation response
                            let created_uuid = create_response.message
                                .split("Workflow ")
                                .nth(1)
                                .and_then(|s| s.split(" created").next())
                                .unwrap_or(&plan.workflow_uuid)
                                .to_string();

                            println!("[INFO] Workflow persisted with UUID: {}", created_uuid);
                            created_uuid
                        }
                        Err(e) => {
                            println!("[WARN] Failed to persist workflow: {}", e);
                            plan.workflow_uuid.clone() // Use planning UUID as fallback
                        }
                    }
                } else {
                    plan.workflow_uuid.clone()
                };

                // Convert workflow spec to protobuf if available
                let workflow_spec = plan.workflow_spec.as_ref()
                    .map(|spec| crate::json_to_proto_struct(spec));

                let response = crate::proto::PlanWorkflowResponse {
                    success: plan.success,
                    message: plan.message,
                    workflow_spec,
                    validation_errors: plan.validation_errors,
                    estimated_steps: plan.estimated_steps as i32,
                    requires_approval: plan.requires_approval,
                    workflow_uuid: final_uuid,
                };

                println!("[INFO] Plan workflow completed: {}", response.message);
                Ok(Response::new(response))
            }
            Err(status) => {
                // Record failed planning metrics
                self.agent_monitor.record_planning_attempt(
                    plan_request.agent_id,
                    format!("Agent {}", plan_request.agent_id), // Would get real name from database
                    plan_request.objective.clone(),
                    planning_duration,
                    false,
                    0,
                    false,
                    Some("planning_error".to_string()),
                    tools_used,
                ).await;

                println!("[ERROR] Plan workflow failed: {}", status.message());

                let response = crate::proto::PlanWorkflowResponse {
                    success: false,
                    message: format!("Workflow planning failed: {}", status.message()),
                    workflow_spec: None,
                    validation_errors: vec![status.message().to_string()],
                    estimated_steps: 0,
                    requires_approval: false,
                    workflow_uuid: String::new(),
                };

                Ok(Response::new(response))
            }
        }
    }
}

// Additional methods for RpcServer (not part of the trait)
impl RpcServer {
    /// Extract likely tools from the objective text (simple heuristic)
    fn extract_tools_from_objective(&self, objective: &str) -> Vec<String> {
        let objective_lower = objective.to_lowercase();
        let mut tools = Vec::new();

        if objective_lower.contains("python") || objective_lower.contains("data") || objective_lower.contains("analyze") {
            tools.push("python".to_string());
        }
        if objective_lower.contains("web") || objective_lower.contains("scrape") || objective_lower.contains("website") {
            tools.push("webscrape".to_string());
        }
        if objective_lower.contains("prompt") || objective_lower.contains("generate") || objective_lower.contains("write") {
            tools.push("prompt".to_string());
        }

        tools
    }

    /// Get agent statistics (could be exposed via additional gRPC endpoint)
    pub async fn get_agent_statistics(&self, agent_id: i32) -> Option<crate::services::agent_monitoring::AgentStats> {
        self.agent_monitor.get_agent_stats(agent_id).await
    }

    /// Get system statistics (could be exposed via additional gRPC endpoint)
    pub async fn get_system_statistics(&self) -> crate::services::agent_monitoring::SystemStats {
        self.agent_monitor.get_system_stats().await
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> crate::services::agent_cache::CacheStats {
        self.agent_cache.get_stats().await
    }

    /// Get batch processing statistics
    pub async fn get_batch_statistics(&self) -> crate::services::request_batcher::BatchStats {
        self.request_batcher.get_stats().await
    }

    /// Get agent from cache or fallback to database
    pub async fn get_cached_agent(&self, agent_id: i32) -> Option<crate::services::agent_cache::CachedAgent> {
        // Try cache first
        if let Some(agent) = self.agent_cache.get_agent(agent_id).await {
            return Some(agent);
        }

        // TODO: Fallback to database lookup
        // This would typically query the database and then cache the result
        None
    }

    /// Batch plan workflows for improved performance
    pub async fn plan_workflow_batched(
        &self,
        agent_id: i32,
        objective: String,
        context: Option<serde_json::Value>,
        constraints: Option<serde_json::Value>,
        is_ephemeral: bool,
    ) -> Result<crate::services::request_batcher::BatchResponse, String> {
        self.request_batcher
            .plan_workflow_batched(agent_id, objective, context, constraints, is_ephemeral)
            .await
    }
}
