use axum::{routing, Router};
use socketioxide::{extract::{Data, SocketRef}, SocketIo};
use serde_json::{json, Value};

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use portico_server::{Agent, Step, RuntimeSession, Job, JobStatus};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (layer, io) = SocketIo::new_layer();
    
    // Set up single WebSocket connection handler
    io.ns("/", |socket: SocketRef| {
        println!("Socket connected: {}", socket.id);

        // Handle postgres changes for UserJob table
        socket.on("postgres_changes", |socket: SocketRef, Data::<Value>(data)| async move {
            println!("Received UserJob changes: {:?}", data);
            
            if let Ok(msg) = serde_json::from_value::<RealtimeMessage>(data) {
                match msg.message_type {
                    MessageType::Insert => {
                        println!("New UserJob: {:?}", msg.new);
                        socket.broadcast().emit("userjob_created", msg.new).await.ok();
                    },
                    MessageType::Update => {
                        println!("UserJob updated: {:?}", msg.new);
                        socket.broadcast().emit("userjob_updated", msg.new).await.ok();
                    },
                    MessageType::Delete => {
                        println!("UserJob deleted: {:?}", msg.old.unwrap_or(msg.new));
                        socket.broadcast().emit("userjob_deleted", msg.old.unwrap_or(msg.new)).await.ok();
                    },
                    _ => println!("Unhandled message type: {:?}", msg.message_type),
                }
            }
        });

        // Subscribe to UserJob table
        let subscribe_msg = SubscribeMessage {
            message_type: "postgres_changes".to_string(),
            schema: "public".to_string(),
            table: "UserJob".to_string(),
            event_filter: "*".to_string(),
        };
        socket.emit("subscribe", subscribe_msg).ok();
    });

    // Set up basic HTTP server with WebSocket support
    let app = Router::new()
        .route("/", routing::get(|| async { "Realtime Server" }))
        .layer(layer);

    println!("Starting server on port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

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


