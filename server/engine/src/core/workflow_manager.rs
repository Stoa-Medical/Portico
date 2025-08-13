use crate::handlers::{run, fyi, sync};
use crate::proto::{SignalRequest, SignalResponse, SignalType};
use crate::proto_struct_to_json;
use crate::SharedWorkflowMap;
use portico_shared::{DatabaseItem, RunningStatus, RuntimeSession};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::Status;
use uuid;

// Workflow manager handles message queuing and processing
pub struct WorkflowManager {
    pub workflows: SharedWorkflowMap,
    // Map from local ID (as string) to global UUID for quick lookups
    pub local_id_map: HashMap<String, String>,
    pub message_queues: HashMap<String, mpsc::Sender<SignalRequest>>,
    pub db_pool: PgPool,
}

impl WorkflowManager {
    pub fn new(workflows: SharedWorkflowMap, db_pool: PgPool) -> Self {
        Self {
            workflows,
            local_id_map: HashMap::new(),
            message_queues: HashMap::new(),
            db_pool,
        }
    }

    // Set up message queues for all existing workflows
    pub async fn init_workflow_queues(&mut self) -> Result<(), Status> {
        // Collect all workflow UUIDs and their local IDs first to avoid borrowing conflicts
        let workflow_data: Vec<(String, Option<i32>)> = {
            let workflows = self.workflows.read().await;
            println!(
                "[INFO] Initializing message queues for {} existing workflows",
                workflows.len()
            );
            workflows
                .iter()
                .map(|(uuid, workflow)| (uuid.clone(), workflow.identifiers.local_id))
                .collect()
        };

        // Populate the local_id_map
        for (workflow_uuid, maybe_local_id) in &workflow_data {
            if let Some(local_id) = maybe_local_id {
                self.local_id_map
                    .insert(local_id.to_string(), workflow_uuid.clone());
            }
        }

        // Create message queues for each workflow
        for (workflow_uuid, _) in workflow_data {
            self.setup_workflow_queue(workflow_uuid).await?;
        }

        Ok(())
    }

    // Set up a queue for a specific workflow
    pub async fn setup_workflow_queue(&mut self, workflow_uuid: String) -> Result<(), Status> {
        let (tx, mut rx) = mpsc::channel::<SignalRequest>(1024);

        // Store the sender for this workflow
        self.message_queues.insert(workflow_uuid.clone(), tx);

        // Clone values for the async task
        let workflow_map = Arc::clone(&self.workflows);
        let pool = self.db_pool.clone();

        // Spawn a worker task for this workflow
        tokio::spawn(async move {
            println!("[INFO] Worker started for workflow: {}", workflow_uuid);

            while let Some(signal_request) = rx.recv().await {
                // Process the signal for this specific workflow
                let result = WorkflowManager::process_workflow_signal(
                    Arc::clone(&workflow_map),
                    signal_request,
                    pool.clone(),
                )
                .await;

                match result {
                    Ok(response) => {
                        println!(
                            "[SUCCESS] Processed signal for workflow {}: {}",
                            workflow_uuid, response.message
                        );
                    }
                    Err(e) => {
                        println!(
                            "[ERROR] Failed to process signal for workflow {}: {}",
                            workflow_uuid, e
                        );
                    }
                }
            }

            println!("[INFO] Worker terminated for workflow: {}", workflow_uuid);
        });

        Ok(())
    }

    // Queue a signal for a specific workflow
    pub async fn queue_signal(&self, signal_request: SignalRequest) -> Result<SignalResponse, Status> {
        // Map the workflow_id to global UUID
        let workflow_uuid = if signal_request.workflow_id != 0 {
            // Look up by local ID
            if let Some(uuid) = self.local_id_map.get(&signal_request.workflow_id.to_string()) {
                uuid.clone()
            } else {
                return Err(Status::not_found(format!(
                    "Workflow with local ID {} not found",
                    signal_request.workflow_id
                )));
            }
        } else {
            return Err(Status::invalid_argument("workflow_id is required"));
        };

        // Get the message queue for this workflow
        if let Some(tx) = self.message_queues.get(&workflow_uuid) {
            match tx.send(signal_request).await {
                Ok(_) => Ok(SignalResponse {
                    success: true,
                    message: "Signal queued successfully".to_string(),
                    runtime_session_uuid: "".to_string(), // Will be filled by worker
                    result_data: None,
                }),
                Err(_) => Err(Status::internal("Failed to queue signal")),
            }
        } else {
            Err(Status::not_found(format!(
                "No queue found for workflow: {}",
                workflow_uuid
            )))
        }
    }

    // Process a signal for a specific workflow
    async fn process_workflow_signal(
        workflow_map: SharedWorkflowMap,
        signal_request: SignalRequest,
        pool: PgPool,
    ) -> Result<SignalResponse, Status> {
        match signal_request.signal_type() {
            SignalType::Run => {
                if let Some(ref payload) = signal_request.payload {
                    if let Some(run_data) = payload.run_data.as_ref() {
                        let json_data = proto_struct_to_json(run_data);
                        run::handle_run_signal(workflow_map, signal_request.workflow_id, json_data, pool)
                            .await
                    } else {
                        Err(Status::invalid_argument("Missing run_data for RUN signal"))
                    }
                } else {
                    Err(Status::invalid_argument("Missing payload for RUN signal"))
                }
            }
            SignalType::Sync => {
                if let Some(ref payload) = signal_request.payload {
                    if let Some(sync_payload) = payload.sync.as_ref() {
                        sync::handle_sync_signal(workflow_map, sync_payload.clone(), pool).await
                    } else {
                        Err(Status::invalid_argument("Missing sync payload for SYNC signal"))
                    }
                } else {
                    Err(Status::invalid_argument("Missing payload for SYNC signal"))
                }
            }
            SignalType::Fyi => {
                if let Some(ref payload) = signal_request.payload {
                    if let Some(fyi_data) = payload.fyi_data.as_ref() {
                        let json_data = proto_struct_to_json(fyi_data);
                        fyi::handle_fyi_signal(workflow_map, signal_request.workflow_id, json_data, pool)
                            .await
                    } else {
                        Err(Status::invalid_argument("Missing fyi_data for FYI signal"))
                    }
                } else {
                    Err(Status::invalid_argument("Missing payload for FYI signal"))
                }
            }
        }
    }

    // Add a new workflow to the system
    pub async fn add_workflow(&mut self, workflow_uuid: String) -> Result<(), Status> {
        // Set up a queue for the new workflow
        self.setup_workflow_queue(workflow_uuid).await?;
        println!("[INFO] Added new workflow queue");
        Ok(())
    }

    // Remove a workflow from the system
    pub async fn remove_workflow(&mut self, workflow_uuid: &str) -> Result<(), Status> {
        // Remove from workflows map
        {
            let mut workflows = self.workflows.write().await;
            workflows.remove(workflow_uuid);
        }

        // Remove local ID mapping if exists
        self.local_id_map.retain(|_k, v| v != workflow_uuid);

        // Remove message queue (dropping the sender will close the channel)
        self.message_queues.remove(workflow_uuid);

        println!("[INFO] Removed workflow: {}", workflow_uuid);
        Ok(())
    }

    // Get workflow count
    pub async fn workflow_count(&self) -> usize {
        let workflows = self.workflows.read().await;
        workflows.len()
    }
}
