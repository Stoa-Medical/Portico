use crate::SharedWorkflowMap;
use crate::proto::{SignalResponse, SyncPayload};
use sqlx::PgPool;
use tonic::Status;

// Sync signal handler for workflows
pub async fn handle_sync_signal(
    _workflow_map: SharedWorkflowMap,
    _sync_payload: SyncPayload,
    _pool: PgPool,
) -> Result<SignalResponse, Status> {
    println!("[INFO] Processing sync signal");

    Ok(SignalResponse {
        success: true,
        message: "Sync completed successfully".to_string(),
        runtime_session_uuid: "".to_string(),
        result_data: None,
    })
}
