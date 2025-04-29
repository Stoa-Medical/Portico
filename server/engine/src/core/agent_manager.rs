use crate::handlers::{command, fyi, sync};
use crate::proto::signal_request;
use crate::proto::{CommandOperation, EntityType, SignalRequest, SignalResponse, SignalType};
use crate::proto_struct_to_json;
use crate::SharedAgentMap;
use portico_shared::DatabaseItem;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::Status;
use uuid;

// Agent manager handles message queuing and processing
pub struct AgentManager {
    pub agents: SharedAgentMap,
    pub message_queues: HashMap<String, mpsc::Sender<SignalRequest>>,
    pub db_pool: PgPool,
}

impl AgentManager {
    pub fn new(agents: SharedAgentMap, db_pool: PgPool) -> Self {
        Self {
            agents,
            message_queues: HashMap::new(),
            db_pool,
        }
    }

    // Set up message queues for all existing agents
    pub async fn init_agent_queues(&mut self) -> Result<(), Status> {
        // Collect all agent UUIDs first to avoid borrowing conflicts
        let agent_uuids: Vec<String> = {
            let agents = self.agents.read().await;
            println!(
                "[INFO] Initializing message queues for {} existing agents",
                agents.len()
            );
            agents.keys().cloned().collect()
        };

        for agent_uuid in agent_uuids {
            if let Err(e) = self.setup_agent_queue(agent_uuid.clone()).await {
                eprintln!(
                    "[ERROR] Failed to initialize queue for agent {}: {}",
                    agent_uuid, e
                );
            }
        }

        Ok(())
    }

    // Process a new SignalRequest coming from gRPC
    pub async fn process_signal(
        &mut self,
        signal: SignalRequest,
    ) -> Result<SignalResponse, Status> {
        let runtime_session_uuid = uuid::Uuid::new_v4().to_string();

        match signal.signal_type() {
            SignalType::Command => {
                if let Some(signal_request::Payload::Command(cmd)) = &signal.payload {
                    match cmd.operation() {
                        CommandOperation::Create => {
                            command::handle_create(self, cmd, runtime_session_uuid).await
                        }
                        CommandOperation::Update => {
                            command::handle_update(self, cmd, runtime_session_uuid).await
                        }
                        CommandOperation::Delete => {
                            command::handle_delete(self, cmd, runtime_session_uuid).await
                        }
                        CommandOperation::Run => {
                            command::handle_run(self, signal.clone(), runtime_session_uuid).await
                        }
                    }
                } else {
                    Err(Status::invalid_argument(
                        "Missing command payload for COMMAND signal type",
                    ))
                }
            }
            SignalType::Sync => sync::handle_sync(self, &signal, runtime_session_uuid).await,
            SignalType::Fyi => fyi::handle_fyi(self, &signal, runtime_session_uuid).await,
        }
    }

    // Set up processing for a specific agent
    pub async fn setup_agent_queue(&mut self, agent_uuid: String) -> Result<(), Status> {
        // Check if queue already exists
        if self.message_queues.contains_key(&agent_uuid) {
            return Ok(());
        }

        println!("[INFO] Setting up message queue for agent {}", agent_uuid);

        // Create a channel for this agent
        let (tx, mut rx) = mpsc::channel::<SignalRequest>(32);
        self.message_queues.insert(agent_uuid.clone(), tx);

        // Clone shared resources for the worker task
        let agents = Arc::clone(&self.agents);
        let db_pool = self.db_pool.clone();

        // Spawn a dedicated worker for this agent
        tokio::spawn(async move {
            println!("[INFO] Started worker for agent {}", agent_uuid);

            while let Some(signal) = rx.recv().await {
                println!(
                    "[INFO] Agent {} worker processing signal type {:?}",
                    agent_uuid,
                    signal.signal_type()
                );

                if let SignalType::Command = signal.signal_type() {
                    if let Some(crate::proto::signal_request::Payload::Command(cmd)) =
                        &signal.payload
                    {
                        if cmd.entity_type() == EntityType::Agent
                            && cmd.operation() == CommandOperation::Run
                        {
                            if let Some(data) = &cmd.data {
                                let run_data = proto_struct_to_json(data);
                                let agents_guard = agents.read().await;

                                if let Some(agent) = agents_guard.get(&agent_uuid) {
                                    println!("[INFO] Running agent {} with data", agent_uuid);

                                    // Call agent.run() which creates a RuntimeSession internally
                                    match agent.run(run_data).await {
                                        Ok(session) => {
                                            println!(
                                                "[INFO] Agent execution successful, saving session"
                                            );

                                            // Save the session to the database using the DatabaseItem trait
                                            if let Err(e) = session.try_db_create(&db_pool).await {
                                                eprintln!("[ERROR] Failed to save session: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[ERROR] Agent execution failed: {}", e);
                                        }
                                    }
                                } else {
                                    eprintln!("[ERROR] Agent {} not found in map", agent_uuid);
                                }
                            }
                        }
                    }
                }
            }

            println!("[INFO] Worker for agent {} shutting down", agent_uuid);
        });

        Ok(())
    }
}
