use dotenvy::dotenv;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use sqlx::postgres::PgPoolOptions;
use anyhow::Result;

use portico_engine::read_json_message;
use portico_shared::models::{Agent, RuntimeSession, Signal, Step};
use portico_shared::{IdFields, RunningStatus, TimestampFields};

#[tokio::main]
async fn main() -> Result<()> {
    // Read Config
    // - Load environment variables
    dotenv().ok();

    // - Read environment configuration
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
    if init_message.get("server-init").is_some() {
        println!("Received init message from bridge service");
    } else {
        return Err(anyhow::anyhow!("Expected init message, got something else"));
    }

    println!("Bridge service initialized successfully");


    // Connect to database
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await?;

    println!("Connected to the database");

    // Pull corresponding `Agents` and corresponding `Steps`
    let agents: Vec<Agent> = Agent::try_db_select_all(&pool);
    let steps: Vec<Step> = Step::try_db_select_all(&pool);

    // Start event loop -- respond to bridge messages.
    // - CREATE Signal with data: run requested Agent
    // - CREATE Agent/Step: make a new model
    // - UPDATE Agent/Step: update the in-memory object
    // - DELETE Agent/Step: drop the in-memory object
    // Event loop --
    //   Event loop will run on different threads. So will need a locking mechanism to avoid race conditions
    //   Implement as an in-memory Message queue. Clone the `Agent` + `Step` state for each Thread (so "pure" function achieved)



    // If get exit signal: clean up resources before exiting

    Ok(())
}
