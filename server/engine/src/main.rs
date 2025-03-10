use dotenv::dotenv;
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
use uuid::Uuid;

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

    // Initialize thread pool

    // Load state for Agents + Steps
    // - Connect to database
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
