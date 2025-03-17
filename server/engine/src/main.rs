use dotenvy::dotenv;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;
use sqlx::Pool;
use sqlx::postgres::Postgres;

// Import our model abstractions
use portico_engine::models::{Agent, RuntimeSession, Signal, Step};
use portico_engine::{IdFields, RunningStatus, TimestampFields};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Preps python interpreter (only needs to run once, though repeat calls are negligible)
    pyo3::prepare_freethreaded_python();

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
    let n_cores: u16 = std::thread::available_parallelism().unwrap().get().try_into().unwrap();
    let thread_multiplier: u16 = env::var("RUNTIME_THREAD_MULTIPLIER")
        .expect("RUNTIME_THREAD_MULTIPLIER needs to be specified")
        .parse()
        .expect("RUNTIME_THREAD_MULTIPLIER should be an integer");
    let thread_count = n_cores * thread_multiplier;

    println!(
        "Configuration loaded: PORT={}, thread_count={}",
        port, thread_count
    );

    // Initialize thread pool
    // - Initialize Tokio thread pool with dynamic configuration
    let rt: Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(thread_count.try_into().unwrap())
        .enable_all()
        .build()?;

    // - Get a handle to the runtime
    let rt_handle = rt.handle().clone();

    println!("Tokio runtime initialized with {} worker threads", thread_count);


    // Load state for Agents + Steps
    // - Connect to database
    let pool = Pool::<Postgres>::connect(&db_url);

    // - Construct single SQL query for pulling `Agents` and corresponding `Steps`
    // - Given response,

    // Start TCP/IP Listener from the bridge service
    // - Open specified port
    // - Wait until bridge service connects (init event received)

    // Start event loop -- respond to bridge messages.
    //   Event loop will run on different threads. So will need a locking mechanism to avoid race conditions
    //   Implement as an in-memory Message queue. Clone the `Agent` + `Step` state for each Thread (so "pure" function achieved)
    // - CREATE Signal with data: run requested Agent
    // - CREATE Agent/Step: make a new model
    // - UPDATE Agent/Step: update the in-memory object
    // - DELETE Agent/Step: drop the in-memory object

    // If get exit signal: clean up resources before exiting

    Ok(())
}
