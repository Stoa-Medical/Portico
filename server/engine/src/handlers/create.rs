use crate::core::workflow_manager::WorkflowManager;
use crate::proto::GeneralResponse;
use crate::proto_struct_to_json;
use portico_shared::models::Workflow;
use portico_shared::JsonLike;
use portico_shared::DatabaseItem;
use prost_types::Struct;
use tonic::Status;

// Create workflow operation handler
pub async fn handle_create_workflow(
    manager: &mut WorkflowManager,
    workflow_json_struct: &Struct,
) -> Result<GeneralResponse, Status> {
    let workflow_json = proto_struct_to_json(workflow_json_struct);

    println!("[INFO] Processing Workflow creation: {}", workflow_json);

    match Workflow::from_json(workflow_json.clone()) {
        Ok(workflow) => {
            let workflow_uuid = workflow.identifiers.global_uuid.clone();
            println!("[INFO] Adding workflow with UUID: {}", workflow_uuid);

            // Setup a queue for this workflow
            if let Err(e) = manager.setup_workflow_queue(workflow_uuid.clone()).await {
                eprintln!("[ERROR] Failed to setup workflow queue: {}", e);
                return Err(Status::internal("Failed to setup message queue for workflow"));
            }

            // Store the workflow
            let mut workflows_guard = manager.workflows.write().await;
            workflows_guard.insert(workflow_uuid.clone(), workflow);

            // Save to database if not already there
            let workflow = workflows_guard.get(&workflow_uuid).unwrap();
            if let Err(e) = workflow.try_db_create(&manager.db_pool).await {
                if !e.to_string().contains("duplicate key") {
                    eprintln!("[ERROR] Failed to save workflow to database: {}", e);
                    return Err(Status::internal("Failed to save workflow to database"));
                }
            }

            // Add to local_id_map if the workflow has a local_id
            if let Some(local_id) = workflow.identifiers.local_id {
                let local_id_str = local_id.to_string();
                println!("[INFO] Adding mapping from local ID {} to UUID {}", local_id_str, workflow_uuid);
                manager.local_id_map.insert(local_id_str, workflow_uuid.clone());
            }

            Ok(GeneralResponse {
                success: true,
                message: format!("Workflow {} created successfully", workflow_uuid),
            })
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to parse workflow JSON: {}", e);
            Err(Status::invalid_argument(format!(
                "Invalid workflow data: {}",
                e
            )))
        }
    }
}
