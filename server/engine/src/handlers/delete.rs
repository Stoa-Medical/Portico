use crate::core::agent_manager::AgentManager;
use crate::proto::GeneralResponse;
use sqlx;
use tonic::Status;

// Delete agent operation handler
pub async fn handle_delete_agent(
    manager: &mut AgentManager,
    agent_uuid: String,
) -> Result<GeneralResponse, Status> {
    if agent_uuid.is_empty() {
        return Err(Status::invalid_argument(
            "Missing agent_uuid for delete operation",
        ));
    }

    // Remove the agent from memory
    let agents_guard = manager.agents.write().await;
    if !agents_guard.contains_key(&agent_uuid) {
        eprintln!(
            "[ERROR] Agent deletion failed: Agent with UUID {} not found in memory",
            agent_uuid
        );
        return Err(Status::not_found(format!(
            "Agent with UUID {} not found",
            agent_uuid
        )));
    }

    // Remove the message queue
    manager.message_queues.remove(&agent_uuid);

    // First delete associated steps for the agent
    if let Err(e) = sqlx::query("DELETE FROM steps WHERE agent_id IN (SELECT id FROM agents WHERE global_uuid = $1)")
        .bind(&agent_uuid)
        .execute(&manager.db_pool)
        .await {
        eprintln!("[ERROR] Failed to delete agent's steps from database: {}", e);
        return Err(Status::internal("Failed to delete agent's steps from database"));
    }

    // Then delete the agent itself
    if let Err(e) = sqlx::query("DELETE FROM agents WHERE global_uuid = $1")
        .bind(&agent_uuid)
        .execute(&manager.db_pool)
        .await
    {
        eprintln!("[ERROR] Failed to delete agent from database: {}", e);
        return Err(Status::internal("Failed to delete agent from database"));
    }

    println!("[INFO] Agent successfully removed");
    Ok(GeneralResponse {
        success: true,
        message: format!("Agent {} deleted successfully", agent_uuid),
    })
}
