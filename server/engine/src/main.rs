use anyhow::Result;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;

use portico_engine::RpcServer;
use portico_shared::models::Workflow;
use portico_shared::DatabaseItem;

#[tokio::main]
async fn main() -> Result<()> {
    // Read Config
    dotenv().ok();
    let grpc_port: u16 = env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse()
        .expect("GRPC_PORT should be a number");
    let db_url: String = env::var("POSTGRES_DB_URI")
        .expect("POSTGRES_DB_URI needs to be specified")
        .parse()
        .unwrap();

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

    println!("Fetched workflows successfully, count: {}", workflows.len());

    // Create a thread-safe workflow map
    let workflow_map: Arc<RwLock<HashMap<String, Workflow>>> = Arc::new(RwLock::new(
        workflows
            .into_iter()
            .map(|workflow| (workflow.identifiers.global_uuid.clone(), workflow))
            .collect(),
    ));

    // Create an instance of our gRPC service
    let bridge_service = RpcServer::new(workflow_map, db_conn_pool);

    // Start the gRPC server
    println!("Starting gRPC server with workflow queuing support...");
    Server::builder()
        .add_service(bridge_service.with_server())
        .serve(addr)
        .await?;

    Ok(())
}
