use crate::handlers::{run, fyi, sync};
use crate::proto::{SignalRequest, SignalResponse, SignalType};
use crate::proto_struct_to_json;
use crate::SharedAgentMap;
use portico_shared::{DatabaseItem, RunningStatus, RuntimeSession};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::Status;
use uuid;

// Agent manager handles message queuing and processing
pub struct AgentManager {
    pub agents: SharedAgentMap,
    // Map from local ID (as string) to global UUID for quick lookups
    pub local_id_map: HashMap<String, String>,
    pub message_queues: HashMap<String, mpsc::Sender<SignalRequest>>,
    pub db_pool: PgPool,
}

impl AgentManager {
    pub fn new(agents: SharedAgentMap, db_pool: PgPool) -> Self {
        Self {
            agents,
            local_id_map: HashMap::new(),
            message_queues: HashMap::new(),
            db_pool,
        }
    }

    // Set up message queues for all existing agents
    pub async fn init_agent_queues(&mut self) -> Result<(), Status> {
        // Collect all agent UUIDs and their local IDs first to avoid borrowing conflicts
        let agent_data: Vec<(String, Option<i32>)> = {
            let agents = self.agents.read().await;
            println!(
                "[INFO] Initializing message queues for {} existing agents",
                agents.len()
            );
            agents
                .iter()
                .map(|(uuid, agent)| (uuid.clone(), agent.identifiers.local_id))
                .collect()
        };

        // Populate the local_id_map
        for (agent_uuid, maybe_local_id) in &agent_data {
            if let Some(local_id) = maybe_local_id {
                let local_id_str = local_id.to_string();
                println!("[INFO] Mapping local ID {} to UUID {}", local_id_str, agent_uuid);
                self.local_id_map.insert(local_id_str, agent_uuid.clone());
            }
        }

        // Set up queues for all agents
        for (agent_uuid, _) in agent_data {
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

        println!(
            "[INFO] Processing signal: type={:?}, signal_id={}",
            signal.signal_type(),
            signal.signal_id
        );

        match signal.signal_type() {
            SignalType::Run => {
                // Direct handling of RUN signals
                run::handle_run(self, signal.clone(), runtime_session_uuid).await
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
                    "[INFO] Agent {} worker processing signal: signal_id={}, type={:?}",
                    agent_uuid,
                    signal.signal_id,
                    signal.signal_type()
                );

                if let SignalType::Run = signal.signal_type() {
                    if let Some(crate::proto::signal_request::Payload::RunData(run_data)) =
                        &signal.payload
                    {
                        // Process the run data - expecting a "data" field in the wrapper
                        if let Some(data_field) = run_data.fields.get("data") {
                            if let Some(value) = &data_field.kind {
                                if let prost_types::value::Kind::StructValue(data_struct) = value {
                                    let run_data_json = proto_struct_to_json(data_struct);
                                    let agents_guard = agents.read().await;

                                    if let Some(agent) = agents_guard.get(&agent_uuid) {
                                        println!(
                                            "[INFO] Running agent {} with data from signal {}",
                                            agent_uuid,
                                            signal.signal_id
                                        );

                                        // Call agent.run() which creates a RuntimeSession internally
                                        match agent.run(run_data_json.clone()).await {
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
                                                eprintln!(
                                                    "[ERROR] Agent execution failed: {}",
                                                    e
                                                );

                                                // Create a failed session
                                                println!("[INFO] Creating and saving failed RuntimeSession");

                                                // Extract steps from the agent
                                                let steps = agent.steps.clone();

                                                // Create a new RuntimeSession with failed status
                                                let mut failed_session = RuntimeSession::new(
                                                    run_data_json,
                                                    steps,
                                                );

                                                // Set the status to Cancelled
                                                failed_session.status = RunningStatus::Cancelled;

                                                // Set the last result to include the error message
                                                failed_session.last_successful_result = Some(json!({
                                                    "error": e.to_string(),
                                                    "signal_uuid": signal.signal_id,
                                                    "agent_uuid": agent_uuid
                                                }));

                                                // Try to save the failed session
                                                if let Err(db_err) = failed_session
                                                    .try_db_create(&db_pool)
                                                    .await
                                                {
                                                    eprintln!("[ERROR] Failed to save error session: {}", db_err);
                                                } else {
                                                    println!(
                                                        "[INFO] Failed session saved successfully with UUID: {}",
                                                        failed_session.identifiers.global_uuid
                                                    );
                                                }
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
            }

            println!("[INFO] Worker for agent {} shutting down", agent_uuid);
        });

        Ok(())
    }
}
