use axum::{routing::get, Router};
use socketioxide::{extract::{Data, SocketRef}, SocketIo};
use serde_json::{json, Value};

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use portico_server::{Agent, Step, RuntimeSession};

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

    // Create subscription message for Postgres changes
    let subscribe_msg = SubscribeMessage {
        message_type: "postgres_changes".to_string(),
        schema: "public".to_string(),
        table: "events".to_string(),
        event_filter: "*".to_string(),
    };

    socket.emit("subscribe", &subscribe_msg).ok();
}

async fn handle_insert(socket: SocketRef, new_record: Value) {
    println!("New record inserted: {:?}", new_record);
    socket.broadcast().emit("db_insert", &new_record).await.ok();
}

async fn handle_update(socket: SocketRef, new_record: Value, old_record: Option<Value>) {
    println!("Record updated from {:?} to {:?}", old_record, new_record);
    socket.broadcast().emit("db_update", &json!({
        "old": old_record,
        "new": new_record
    })).await.ok();
}

async fn handle_delete(socket: SocketRef, deleted_record: Value) {
    println!("Record deleted: {:?}", deleted_record);
    socket.broadcast().emit("db_delete", &deleted_record).await.ok();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);

    let app = Router::new()
        .route("/", get(|| async { "Realtime Server" }))
        .layer(layer);

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
