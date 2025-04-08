use dotenvy::dotenv;
use std::collections::HashMap;
use std::env;
use std::net::TcpListener;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use anyhow::{anyhow, Result};

use portico_engine::{read_json_message, BridgeMessage};
use portico_shared::models::{Agent, RuntimeSession};

#[tokio::main]
async fn main() -> Result<()> {
    // Read Config
    dotenv().ok();
    let port: u16 = env::var("TCPIP_PORT")
        .expect("TCPIP_PORT needs to be specified")
        .parse()
        .expect("TCPIP_PORT should be a number");
    let db_url: String = env::var("POSTGRES_DB_URI")
        .expect("POSTGRES_DB_URI needs to be specified")
        .parse()
        .unwrap();

    // Start TCP/IP Listener from the bridge service
    println!("Starting TCP listener on port {}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    println!("Waiting for bridge service to connect...");

    // Accept connection and wait for init message
    let mut stream = listener.accept()?.0;
    println!("Connection established, waiting for init message");
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;

    // Read and validate init message
    let init_message = read_json_message(&mut stream)?;
    if let BridgeMessage::ServerInit(true) = init_message {
        println!("Received init message from bridge service");
    } else {
        return Err(anyhow!("Expected init message with server-init: true, got something else"));
    }
    println!("Bridge service initialized successfully");

    // Connect to database (share pooled connection)
    let db_conn_pool = PgPoolOptions::new()
        .connect(&db_url)
        .await?;
    println!("Connected to the database successfully");

    // Pull corresponding `Agents` and corresponding `Steps`
    let agents: Vec<Agent> = Agent::try_db_select_all(&db_conn_pool)
        .await
        .expect("Failed to fetch agents from database");

    let agent_map: HashMap<&str, Agent> = agents
        .into_iter()
        .map(|agent| (&agent.identifiers.global_uuid, agent))
        .collect();

    // Start event loop -- wait and listen to `listener`
    // TODO: Can this be an async loop? E.g. incoming messages aren't blocked
    loop {
        // Accept new messages from the bridge service
        match read_json_message(&mut stream) {
            Ok(message) => {
                println!("Received message: {:?}", message);
                // Process message here
                // TODO: Implement message handling logic
            },
            Err(e) => {
                // Check if the error is due to connection being closed
                if e.to_string().contains("connection reset") ||
                   e.to_string().contains("broken pipe") ||
                   e.to_string().contains("connection refused") {
                    println!("Bridge service disconnected: {}", e);
                    break;
                }

                // Log other errors but continue the loop
                eprintln!("Error reading message: {}", e);
            }
        }
    }

    Ok(())
}
