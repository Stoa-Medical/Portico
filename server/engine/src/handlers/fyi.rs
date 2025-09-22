use crate::SharedWorkflowMap;
use crate::proto::SignalResponse;
use serde_json::Value;
use sqlx::PgPool;
use tonic::Status;

// FYI signal handler for workflows
pub async fn handle_fyi_signal(
    _workflow_map: SharedWorkflowMap,
    _workflow_id: i32,
    _fyi_data: Value,
    _pool: PgPool,
) -> Result<SignalResponse, Status> {
    println!("[INFO] Processing FYI signal");

    Ok(SignalResponse {
        success: true,
        message: "FYI signal processed successfully".to_string(),
        runtime_session_uuid: "".to_string(),
        result_data: None,
    })
}
