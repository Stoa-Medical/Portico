use crate::core::agent_manager::AgentManager;
use crate::proto::GeneralResponse;
use sqlx;
use tonic::Status;

// Delete agent operation handler
pub async fn handle_delete_agent(
    manager: &mut AgentManager,
    agent_id: i32,
) -> Result<GeneralResponse, Status> {
    if agent_id == 0 {
        return Err(Status::invalid_argument(
            "Missing agent_id for delete operation",
        ));
    }

    // For now, we'll skip the in-memory removal since we're using IDs directly
    // In a production system, you'd want to look up the agent by ID and then remove it
    println!("[INFO] Removing agent with ID {} from database", agent_id);

    // First delete associated steps for the agent
    if let Err(e) = sqlx::query("DELETE FROM steps WHERE agent_id = $1")
        .bind(agent_id)
        .execute(&manager.db_pool)
        .await {
        eprintln!("[ERROR] Failed to delete agent's steps from database: {}", e);
        return Err(Status::internal("Failed to delete agent's steps from database"));
    }

    // Then delete the agent itself
    if let Err(e) = sqlx::query("DELETE FROM agents WHERE id = $1")
        .bind(agent_id)
        .execute(&manager.db_pool)
        .await
    {
        eprintln!("[ERROR] Failed to delete agent from database: {}", e);
        return Err(Status::internal("Failed to delete agent from database"));
    }

    println!("[INFO] Agent successfully removed");
    Ok(GeneralResponse {
        success: true,
        message: format!("Agent with ID {} deleted successfully", agent_id),
    })
}
