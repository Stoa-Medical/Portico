use crate::SharedWorkflowMap;
use crate::proto::SignalResponse;
use crate::json_to_proto_struct;
use portico_database::{DatabaseItem, RuntimeSession};
use serde_json::Value;
use sqlx::PgPool;
use tonic::Status;

// Run signal handler for workflows
pub async fn handle_run_signal(
    workflow_map: SharedWorkflowMap,
    workflow_id: i32,
    run_data: Value,
    pool: PgPool,
) -> Result<SignalResponse, Status> {
    println!("[INFO] Processing run signal for workflow_id: {}", workflow_id);

    // Look up the workflow by ID
    let workflow_uuid = {
        let workflows = workflow_map.read().await;
        let workflow = workflows
            .values()
            .find(|w| w.identifiers.local_id == Some(workflow_id))
            .ok_or_else(|| {
                Status::not_found(format!("Workflow with ID {} not found", workflow_id))
            })?;
        workflow.identifiers.global_uuid.clone()
    };

    // Get the workflow and run it
    let runtime_session = {
        let workflows = workflow_map.read().await;
        let workflow = workflows.get(&workflow_uuid).ok_or_else(|| {
            Status::not_found(format!("Workflow {} not found", workflow_uuid))
        })?;

        match workflow.run(run_data).await {
            Ok(session) => session,
            Err(e) => {
                eprintln!("[ERROR] Failed to run workflow {}: {}", workflow_uuid, e);
                return Err(Status::internal(format!(
                    "Failed to execute workflow: {}",
                    e
                )));
            }
        }
    };

    // Save the runtime session to database
    if let Err(e) = runtime_session.try_db_create(&pool).await {
        eprintln!(
            "[ERROR] Failed to save runtime session to database: {}",
            e
        );
        return Err(Status::internal("Failed to save runtime session"));
    }

    // Convert result to proto format
    let result_data = runtime_session
        .last_successful_result
        .as_ref()
        .map(|result| json_to_proto_struct(result));

    Ok(SignalResponse {
        success: true,
        message: "Workflow executed successfully".to_string(),
        runtime_session_uuid: runtime_session.identifiers.global_uuid,
        result_data,
    })
}
