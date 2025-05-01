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
        signal.signal_id
    );

    let agent_uuid_or_id = signal.agent_id.to_string();

    if agent_uuid_or_id.is_empty() {
        return Err(Status::invalid_argument(
            "Missing agent_uuid for RUN operation",
        ));
    }

    // Check if the agent_uuid is actually a numeric local ID
    let agent_uuid = if agent_uuid_or_id.parse::<i32>().is_ok() {
        // This is a numeric ID, try to look it up in the local_id_map
        if let Some(uuid) = manager.local_id_map.get(&agent_uuid_or_id) {
            println!("[INFO] Found UUID {} for local ID {}", uuid, agent_uuid_or_id);
            uuid.clone()
        } else {
            // No mapping found, return an error
            eprintln!("[ERROR] No UUID mapping found for local ID: {}", agent_uuid_or_id);
            return Err(Status::not_found(format!(
                "Agent with local ID {} not found in UUID map",
                agent_uuid_or_id
            )));
        }
    } else {
        // This is already a UUID, use it directly
        agent_uuid_or_id
    };

    // Forward the signal to the agent's queue if it exists
    if let Some(queue) = manager.message_queues.get(&agent_uuid) {
        // Create a modified signal with the correct UUID
        let mut modified_signal = signal.clone();
        modified_signal.agent_id = agent_uuid.parse::<i32>().unwrap_or(0);

        if let Err(e) = queue.send(modified_signal).await {
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
