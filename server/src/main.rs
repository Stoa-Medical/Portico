use axum::{routing, Router};
use socketioxide::{extract::{Data, SocketRef}, SocketIo};
use serde_json::Value;
use serde::{Deserialize, Serialize};

use portico_server::{
    MessageType, SubscribeMessage
};

#[derive(Debug, Serialize, Deserialize)]
struct RealtimeMessage {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub schema: String,
    pub table: String,
    pub new: Value,
    pub old: Option<Value>,
    pub errors: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (layer, io) = SocketIo::new_layer();
    
    // Set up single WebSocket connection handler
    io.ns("/", |socket: SocketRef| {
        println!("Socket connected: {}", socket.id);

        // Handle postgres changes for Signal table
        socket.on("postgres_changes", |socket: SocketRef, Data::<Value>(data)| async move {
            println!("Received Signal changes: {:?}", data);
            
            if let Ok(msg) = serde_json::from_value::<RealtimeMessage>(data) {
                match msg.message_type {
                    MessageType::Insert => {
                        println!("New Signal: {:?}", msg.new);
                        socket.broadcast().emit("Signal_created", msg.new).ok();
                    },
                    MessageType::Update => {
                        println!("Signal updated: {:?}", msg.new);
                        socket.broadcast().emit("Signal_updated", msg.new).ok();
                    },
                    MessageType::Delete => {
                        println!("Signal deleted: {:?}", msg.old.clone().unwrap_or(msg.new.clone()));
                        socket.broadcast().emit("Signal_deleted", msg.old.clone().unwrap_or(msg.new.clone())).ok();
                    },
                    _ => println!("Unhandled message type: {:?}", msg.message_type),
                }
            }
        });

        // Subscribe to Signal table
        let subscribe_msg = SubscribeMessage {
            message_type: String::from("postgres_changes"),
            schema: String::from("public"),
            table: String::from("Signal"),
            event_filter: String::from("*"),
        };
        socket.emit("subscribe", subscribe_msg).ok();
    });

    // Set up basic HTTP server with WebSocket support
    let app = Router::new()
        .route("/", routing::get(|| async { "Realtime Server" }))
        .layer(layer);

    println!("Starting server on port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
