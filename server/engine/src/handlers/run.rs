use crate::core::agent_manager::AgentManager;
use crate::proto::{SignalRequest, SignalResponse};
use tonic::Status;

// Run operation handler
pub async fn handle_run(
    manager: &AgentManager,
    signal: SignalRequest,
    runtime_session_uuid: String,
) -> Result<SignalResponse, Status> {
    // Process run signal
    println!(
        "[INFO] Processing run operation for signal: {}",
        signal.signal_uuid
    );

    let agent_uuid = signal.agent_uuid.clone();

    if agent_uuid.is_empty() {
        return Err(Status::invalid_argument(
            "Missing agent_uuid for RUN operation",
        ));
    }

    // Forward the signal to the agent's queue if it exists
    if let Some(queue) = manager.message_queues.get(&agent_uuid) {
        if let Err(e) = queue.send(signal.clone()).await {
            eprintln!("[ERROR] Failed to send signal to agent queue: {}", e);
            return Err(Status::internal("Failed to forward signal to agent queue"));
        }

        Ok(SignalResponse {
            success: true,
            message: format!("Signal forwarded to agent {}", agent_uuid),
            runtime_session_uuid,
            result_data: None,
        })
    } else {
        eprintln!("[ERROR] No queue found for agent: {}", agent_uuid);
        Err(Status::not_found(format!(
            "Agent with UUID {} not found",
            agent_uuid
        )))
    }
}
