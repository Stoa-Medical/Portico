use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;

use portico_database::models::Workflow;
use portico_database::DatabaseItem;
use portico_engine::RpcServer;

/// Shared state for the health check HTTP server.
#[derive(Clone)]
struct HealthState {
    draining: Arc<AtomicBool>,
}

/// GET / -- returns 200 when healthy, 503 when draining.
async fn health_check(State(state): State<HealthState>) -> impl IntoResponse {
    if state.draining.load(Ordering::SeqCst) {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "draining" })),
        )
    } else {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ok" })),
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Read Config
    dotenv().ok();

    let grpc_port: u16 = env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse()
        .expect("GRPC_PORT should be a number");

    let health_port: u16 = env::var("HEALTH_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("HEALTH_PORT should be a number");

    // Support both DATABASE_URL (preferred) and POSTGRES_DB_URI (backward compat)
    let db_url: String = env::var("DATABASE_URL")
        .or_else(|_| env::var("POSTGRES_DB_URI"))
        .expect("DATABASE_URL (or POSTGRES_DB_URI) needs to be specified");

    // Define the server address
    let addr = format!("0.0.0.0:{}", grpc_port).parse::<SocketAddr>()?;
    println!("Will try to start the gRPC server on {}", addr);

    println!("Trying to connect to the database...");
    // Connect to database (share pooled connection)
    let db_conn_pool = PgPoolOptions::new().connect(&db_url).await?;
    println!("Connected to the database successfully");

    // Pull corresponding `Workflows` and corresponding `Steps`
    let workflows: Vec<Workflow> = Workflow::try_db_select_all(&db_conn_pool)
        .await
        .expect("Failed to fetch workflows from database");

    println!(
        "Fetched workflows successfully, count: {}",
        workflows.len()
    );

    // Create a thread-safe workflow map
    let workflow_map: Arc<RwLock<HashMap<String, Workflow>>> = Arc::new(RwLock::new(
        workflows
            .into_iter()
            .map(|workflow| (workflow.identifiers.global_uuid.clone(), workflow))
            .collect(),
    ));

    // Create an instance of our gRPC service
    let bridge_service = RpcServer::new(workflow_map, db_conn_pool.clone());

    // Grab a reference to the workflow manager before consuming bridge_service
    let workflow_manager = bridge_service.workflow_manager();

    // Initialize workflow queues and reconstruct from the database before serving traffic
    {
        let mut manager = workflow_manager.lock().await;

        if let Err(e) = manager.init_workflow_queues().await {
            eprintln!("[ERROR] Failed to initialize workflow queues: {}", e);
        }

        println!("[INFO] Reconstructing signal queues from database...");
        if let Err(e) = manager.reconstruct_queues().await {
            eprintln!(
                "[ERROR] Failed to reconstruct queues: {}. Continuing startup anyway.",
                e
            );
        }
    }

    // ---- Health check HTTP server ----
    let draining = Arc::new(AtomicBool::new(false));
    let health_state = HealthState {
        draining: draining.clone(),
    };

    let health_app = Router::new()
        .route("/", get(health_check))
        .with_state(health_state);

    let health_addr: SocketAddr = format!("0.0.0.0:{}", health_port).parse()?;
    let health_listener = tokio::net::TcpListener::bind(health_addr).await?;
    println!("Health check server listening on {}", health_addr);

    let health_server = tokio::spawn(async move {
        if let Err(e) = axum::serve(health_listener, health_app).await {
            eprintln!("[ERROR] Health check server error: {}", e);
        }
    });

    // ---- SIGTERM / Ctrl-C handling ----
    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    // Start the gRPC server (consumes bridge_service)
    println!("Starting gRPC server with workflow queuing support...");
    let server_future = Server::builder()
        .add_service(bridge_service.with_server())
        .serve(addr);

    tokio::select! {
        result = server_future => {
            if let Err(e) = result {
                eprintln!("[ERROR] gRPC server exited with error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT (Ctrl-C), initiating graceful shutdown...");
        }
        _ = async {
            #[cfg(unix)]
            sigterm.recv().await;
            #[cfg(not(unix))]
            std::future::pending::<()>().await;
        } => {
            println!("Received SIGTERM, initiating graceful shutdown...");
        }
    }

    // ---- Graceful shutdown sequence ----
    println!("[INFO] Beginning shutdown drain...");

    // Mark as draining so health checks return 503
    draining.store(true, Ordering::SeqCst);

    // Perform graceful shutdown via the workflow manager
    {
        let mut manager = workflow_manager.lock().await;
        manager.shutdown().await;
    }

    // Abort the health server
    health_server.abort();

    println!("[INFO] Shutdown complete");
    Ok(())
}
