use crate::core::workflow_manager::WorkflowManager;
use crate::proto::GeneralResponse;
use sqlx;
use tonic::Status;

// Delete workflow operation handler
pub async fn handle_delete_workflow(
    manager: &mut WorkflowManager,
    workflow_id: i32,
) -> Result<GeneralResponse, Status> {
    if workflow_id == 0 {
        return Err(Status::invalid_argument(
            "Missing workflow_id for delete operation",
        ));
    }

    // For now, we'll skip the in-memory removal since we're using IDs directly
    // In a production system, you'd want to look up the workflow by ID and then remove it
    println!("[INFO] Removing workflow with ID {} from database", workflow_id);

    // First delete associated steps for the workflow
    if let Err(e) = sqlx::query("DELETE FROM steps WHERE workflow_id = $1")
        .bind(workflow_id)
        .execute(&manager.db_pool)
        .await {
        eprintln!("[ERROR] Failed to delete workflow's steps from database: {}", e);
        return Err(Status::internal("Failed to delete workflow's steps from database"));
    }

    // Then delete the workflow itself
    if let Err(e) = sqlx::query("DELETE FROM workflows WHERE id = $1")
        .bind(workflow_id)
        .execute(&manager.db_pool)
        .await
    {
        eprintln!("[ERROR] Failed to delete workflow from database: {}", e);
        return Err(Status::internal("Failed to delete workflow from database"));
    }

    println!("[INFO] Workflow successfully removed");
    Ok(GeneralResponse {
        success: true,
        message: format!("Workflow with ID {} deleted successfully", workflow_id),
    })
}
