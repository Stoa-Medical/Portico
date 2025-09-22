use crate::SharedWorkflowMap;
use crate::proto::SignalResponse;
use crate::json_to_proto_struct;
use portico_database::{DatabaseItem, IdFields};
use portico_database::models::Step;
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

    // Get the workflow with steps
    let workflow = {
        let workflows = workflow_map.read().await;
        workflows.get(&workflow_uuid)
            .cloned()
            .ok_or_else(|| Status::not_found(format!("Workflow {} not found", workflow_uuid)))?
    };
    let step_ids = workflow.step_ids.clone().unwrap_or_default();

    // Load steps by local IDs
    let mut steps: Vec<Step> = Vec::new();
    for local_id in step_ids {
        let id = IdFields::with_values(Some(local_id), String::new());
        if let Some(step) = Step::try_db_select_by_id(&pool, &id)
            .await
            .map_err(|e| Status::internal(format!("Failed to load step {}: {}", local_id, e)))?
        {
            steps.push(step);
        }
    }

    // Run workflow with loaded steps
    let runtime_session = match workflow.run_with_steps(run_data, steps).await {
        Ok(session) => session,
        Err(e) => {
            eprintln!("[ERROR] Failed to run workflow {}: {}", workflow_uuid, e);
            return Err(Status::internal(format!("Failed to execute workflow: {}", e)));
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
