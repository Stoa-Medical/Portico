use axum::{routing::get, Router};
use socketioxide::{extract::{Data, SocketRef}, SocketIo};
use serde_json::{json, Value};

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use portico_server::{Agent, Step, RuntimeSession, Job, JobStatus};

// TODO: Actually add application-specific logic for each of the cases
fn on_connect(socket: SocketRef) {
    println!("Socket connected: {}", socket.id);

    socket.on("postgres_changes", |socket: SocketRef, Data::<Value>(data)| async move {
        println!("Received postgres changes: {:?}", data);
        
        if let Ok(msg) = serde_json::from_value::<RealtimeMessage>(data) {
            match msg.message_type {
                MessageType::Insert => handle_insert(socket, msg.new).await,
                MessageType::Update => handle_update(socket, msg.new, msg.old).await,
                MessageType::Delete => handle_delete(socket, msg.old.unwrap_or(msg.new)).await,
                MessageType::Error => {
                    if let Some(errors) = msg.errors {
                        println!("Error received: {:?}", errors);
                    }
                },
                _ => println!("Unhandled message type: {:?}", msg.message_type),
            }
        }
    });

    // Handle job creation requests
    socket.on("create_job", |socket: SocketRef, Data::<Value>(data)| async move {
        if let Ok(job) = serde_json::from_value::<Job>(data) {
            socket.broadcast().emit("job_created", job).await.ok();
        }
    });

    // Subscribe to both events and jobs tables
    let subscriptions = vec![
        SubscribeMessage {
            message_type: "postgres_changes".to_string(),
            schema: "public".to_string(),
            table: "events".to_string(),
            event_filter: "*".to_string(),
        },
        SubscribeMessage {
            message_type: "postgres_changes".to_string(),
            schema: "public".to_string(),
            table: "jobs".to_string(),
            event_filter: "*".to_string(),
        }
    ];

    socket.emit("subscribe", &subscribe_msg).ok();
}

async fn handle_insert(socket: SocketRef, new_record: Value) {
    println!("New record inserted: {:?}", new_record);
    socket.broadcast().emit("db_insert", &new_record).await.ok();
}

async fn handle_job_update(socket: SocketRef, new_record: Value) {
    // Parse job status update
    if let Ok(job) = serde_json::from_value::<Job>(new_record.clone()) {
        // Broadcast specific job status events
        let status_event = match job.status {
            JobStatus::Running => "job_started",
            JobStatus::Completed => "job_completed",
            JobStatus::Failed => "job_failed",
            JobStatus::Cancelled => "job_cancelled",
            _ => "job_updated",
        };

        // Emit both generic and specific events
        socket.broadcast().emit(status_event, &job).await.ok();
        socket.broadcast().emit("job_updated", &new_record).await.ok();
    }
}

async fn handle_update(socket: SocketRef, new_record: Value, old_record: Option<Value>) {
    println!("Record updated from {:?} to {:?}", old_record, new_record);
    socket.broadcast().emit("db_update", &json!({
        "old": old_record,
        "new": new_record
    })).await.ok();
}

async fn handle_job_status(socket: SocketRef, job_id: String, status: JobStatus) {
    socket.broadcast().emit("job_status", json!({
        "job_id": job_id,
        "status": status,
        "timestamp": chrono::Utc::now()
    })).await.ok();
}


async fn handle_delete(socket: SocketRef, deleted_record: Value) {
    println!("Record deleted: {:?}", deleted_record);
    socket.broadcast().emit("db_delete", &deleted_record).await.ok();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Set up job management routes
    let app = Router::new()
        .route("/", get(|| async { "Realtime Server" }))
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/cancel", post(cancel_job))
        .layer(layer);
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    // Job management handlers
    async fn list_jobs() -> impl axum::response::IntoResponse {
        // TODO: Implement job listing from database
        axum::Json(json!({"message": "Not implemented"}))
    }

    async fn get_job(axum::extract::Path(id): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
        // TODO: Implement job retrieval from database
        axum::Json(json!({"message": "Not implemented", "job_id": id}))
    }

    println!("Starting server on port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}



// ============ Supabase Realtime things =============
#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeMessage {
    #[serde(rename = "type")]
    message_type: MessageType,
    schema: String,
    table: String,
    commit_timestamp: String,
    #[serde(rename = "eventType")]
    event_type: MessageType,
    new: Value,
    old: Option<Value>,
    errors: Option<ErrorPayload>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MessageType {
    Insert,
    Update,
    Delete,
    Broadcast,
    Presence,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeMessage {
    #[serde(rename = "type")]
    message_type: String,  // "postgres_changes"
    schema: String,
    table: String,
    #[serde(rename = "filter")]
    event_filter: String,  // "*" or specific event type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    message: String,
    code: Option<String>,
    details: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastMessage {
    #[serde(rename = "type")]
    message_type: String,
    event: String,
    payload: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PresenceState {
    #[serde(rename = "type")]
    message_type: String,
    presence_ref: String,
    joins: HashMap<String, Value>,
    leaves: HashMap<String, Value>,
}
