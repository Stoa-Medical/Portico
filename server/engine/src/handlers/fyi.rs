use crate::core::agent_manager::AgentManager;
use crate::proto::{SignalRequest, SignalResponse};
use crate::proto_struct_to_json;
use tonic::Status;

// FYI operation handler
pub async fn handle_fyi(
    _manager: &AgentManager,
    signal: &SignalRequest,
    runtime_session_uuid: String,
) -> Result<SignalResponse, Status> {
    if let Some(crate::proto::signal_request::Payload::FyiData(fyi_data)) = &signal.payload {
        // Handle the FYI data
        println!("[INFO] Received FYI data signal: {}", signal.signal_uuid);

        // Convert the data to JSON for processing
        let fyi_json = proto_struct_to_json(fyi_data);

        // Log any associated agent info
        if !signal.agent_uuid.is_empty() {
            println!("[INFO] FYI related to agent: {}", signal.agent_uuid);
        }

        // For now, we'll just log the data and return success
        println!("[INFO] FYI data: {}", fyi_json);

        Ok(SignalResponse {
            success: true,
            message: "FYI data received".to_string(),
            runtime_session_uuid,
            result_data: None,
        })
    } else {
        Err(Status::invalid_argument(
            "Missing FYI data for FYI signal type",
        ))
    }
}
