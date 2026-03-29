use crate::handlers::{run, fyi, sync};
use crate::proto::{SignalRequest, SignalResponse, SignalType};
use crate::proto_struct_to_json;
use crate::SharedWorkflowMap;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tonic::Status;

/// Default lease duration for signal processing (5 minutes).
const DEFAULT_LEASE_DURATION_SECS: i64 = 300;

/// Maximum time to wait for in-flight signals during shutdown (30 seconds).
const SHUTDOWN_DRAIN_TIMEOUT_SECS: u64 = 30;

// Workflow manager handles message queuing and processing
pub struct WorkflowManager {
    pub workflows: SharedWorkflowMap,
    // Map from local ID (as string) to global UUID for quick lookups
    pub local_id_map: HashMap<String, String>,
    pub message_queues: HashMap<String, mpsc::Sender<SignalRequest>>,
    pub db_pool: PgPool,
    /// Cancellation token used to signal graceful shutdown to all worker tasks.
    pub cancel_token: CancellationToken,
    /// Counter tracking how many signals are currently being processed.
    pub in_flight_count: Arc<AtomicUsize>,
}

impl WorkflowManager {
    pub fn new(workflows: SharedWorkflowMap, db_pool: PgPool) -> Self {
        Self {
            workflows,
            local_id_map: HashMap::new(),
            message_queues: HashMap::new(),
            db_pool,
            cancel_token: CancellationToken::new(),
            in_flight_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    // Set up message queues for all existing workflows
    pub async fn init_workflow_queues(&mut self) -> Result<(), Status> {
        // Collect all workflow UUIDs and their local IDs first to avoid borrowing conflicts
        let workflow_data: Vec<(String, Option<i32>)> = {
            let workflows = self.workflows.read().await;
            println!(
                "[INFO] Initializing message queues for {} existing workflows",
                workflows.len()
            );
            workflows
                .iter()
                .map(|(uuid, workflow)| (uuid.clone(), workflow.identifiers.local_id))
                .collect()
        };

        // Populate the local_id_map
        for (workflow_uuid, maybe_local_id) in &workflow_data {
            if let Some(local_id) = maybe_local_id {
                self.local_id_map
                    .insert(local_id.to_string(), workflow_uuid.clone());
            }
        }

        // Create message queues for each workflow
        for (workflow_uuid, _) in workflow_data {
            self.setup_workflow_queue(workflow_uuid).await?;
        }

        Ok(())
    }

    /// Reconstruct in-memory queues from the database on startup.
    ///
    /// This method:
    /// 1. Reclaims expired leases by resetting dispatched signals whose lease has expired.
    /// 2. Queries all signals with status 'pending' or 'dispatched', ordered by created_at ASC.
    /// 3. For each signal, ensures the workflow's MPSC queue exists and re-enqueues the signal.
    pub async fn reconstruct_queues(&mut self) -> Result<(), Status> {
        println!("[INFO] Reconstructing queues from database...");

        // Step 1: Reclaim expired leases
        let reclaimed = sqlx::query(
            r#"
            UPDATE signals
            SET status = 'pending', leased_at = NULL, lease_expires_at = NULL, updated_at = NOW()
            WHERE status = 'dispatched'
              AND lease_expires_at < NOW()
            "#,
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to reclaim expired leases: {}", e)))?;

        println!(
            "[INFO] Reclaimed {} expired signal leases",
            reclaimed.rows_affected()
        );

        // Step 2: Query pending and dispatched signals ordered by creation time
        let rows = sqlx::query_as::<_, SignalRow>(
            r#"
            SELECT id, workflow_id, signal_type::text as signal_type, initial_data
            FROM signals
            WHERE status IN ('pending', 'dispatched')
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Failed to query recoverable signals: {}", e)))?;

        println!(
            "[INFO] Found {} signals to reconstruct into queues",
            rows.len()
        );

        // Step 3: Re-enqueue each signal
        let mut enqueued = 0u64;
        for row in &rows {
            let Some(workflow_id) = row.workflow_id else {
                println!(
                    "[WARN] Signal {} has no workflow_id, skipping reconstruction",
                    row.id
                );
                continue;
            };

            // Resolve workflow_id to global UUID
            let workflow_uuid = match self.local_id_map.get(&workflow_id.to_string()) {
                Some(uuid) => uuid.clone(),
                None => {
                    println!(
                        "[WARN] No workflow mapping for local_id {}, skipping signal {}",
                        workflow_id, row.id
                    );
                    continue;
                }
            };

            // Ensure the queue exists (it should from init_workflow_queues, but be safe)
            if !self.message_queues.contains_key(&workflow_uuid) {
                println!(
                    "[INFO] Creating queue for workflow {} during reconstruction",
                    workflow_uuid
                );
                self.setup_workflow_queue(workflow_uuid.clone()).await?;
            }

            // Build a minimal SignalRequest to re-enqueue
            let signal_request = SignalRequest {
                workflow_id: workflow_id,
                signal_type: match row.signal_type.as_str() {
                    "run" | "command" => SignalType::Run as i32,
                    "sync" => SignalType::Sync as i32,
                    "fyi" => SignalType::Fyi as i32,
                    _ => {
                        println!(
                            "[WARN] Unknown signal_type '{}' for signal {}, skipping",
                            row.signal_type, row.id
                        );
                        continue;
                    }
                },
                payload: None, // The worker will re-read from DB if needed; original data is persisted
            };

            if let Some(tx) = self.message_queues.get(&workflow_uuid) {
                if tx.send(signal_request).await.is_ok() {
                    enqueued += 1;
                } else {
                    println!(
                        "[ERROR] Failed to enqueue signal {} into workflow {} queue",
                        row.id, workflow_uuid
                    );
                }
            }
        }

        println!(
            "[INFO] Queue reconstruction complete: {}/{} signals enqueued",
            enqueued,
            rows.len()
        );

        Ok(())
    }

    // Set up a queue for a specific workflow
    pub async fn setup_workflow_queue(&mut self, workflow_uuid: String) -> Result<(), Status> {
        let (tx, mut rx) = mpsc::channel::<SignalRequest>(1024);

        // Store the sender for this workflow
        self.message_queues.insert(workflow_uuid.clone(), tx);

        // Clone values for the async task
        let workflow_map = Arc::clone(&self.workflows);
        let pool = self.db_pool.clone();
        let cancel_token = self.cancel_token.clone();
        let in_flight = Arc::clone(&self.in_flight_count);

        // Spawn a worker task for this workflow
        tokio::spawn(async move {
            println!("[INFO] Worker started for workflow: {}", workflow_uuid);

            loop {
                tokio::select! {
                    // Check for cancellation
                    _ = cancel_token.cancelled() => {
                        println!("[INFO] Worker for workflow {} received shutdown signal", workflow_uuid);
                        break;
                    }
                    // Process next signal from queue
                    signal_opt = rx.recv() => {
                        let Some(signal_request) = signal_opt else {
                            // Channel closed
                            break;
                        };

                        // Increment in-flight counter
                        in_flight.fetch_add(1, Ordering::SeqCst);

                        // Acquire lease in the database
                        let lease_acquired = WorkflowManager::acquire_signal_lease(
                            &pool,
                            signal_request.workflow_id,
                            DEFAULT_LEASE_DURATION_SECS,
                        )
                        .await;

                        if let Err(ref e) = lease_acquired {
                            println!(
                                "[WARN] Could not acquire lease for signal on workflow {}: {}",
                                workflow_uuid, e
                            );
                        }

                        // Process the signal
                        let result = WorkflowManager::process_workflow_signal(
                            Arc::clone(&workflow_map),
                            signal_request.clone(),
                            pool.clone(),
                        )
                        .await;

                        match result {
                            Ok(response) => {
                                // Mark signal completed in DB
                                if let Err(e) = WorkflowManager::complete_signal_lease(
                                    &pool,
                                    signal_request.workflow_id,
                                )
                                .await
                                {
                                    println!(
                                        "[WARN] Failed to mark signal completed for workflow {}: {}",
                                        workflow_uuid, e
                                    );
                                }
                                println!(
                                    "[SUCCESS] Processed signal for workflow {}: {}",
                                    workflow_uuid, response.message
                                );
                            }
                            Err(e) => {
                                // Mark signal failed in DB
                                if let Err(db_err) = WorkflowManager::fail_signal_lease(
                                    &pool,
                                    signal_request.workflow_id,
                                    &e.to_string(),
                                )
                                .await
                                {
                                    println!(
                                        "[WARN] Failed to mark signal as failed for workflow {}: {}",
                                        workflow_uuid, db_err
                                    );
                                }
                                println!(
                                    "[ERROR] Failed to process signal for workflow {}: {}",
                                    workflow_uuid, e
                                );
                            }
                        }

                        // Decrement in-flight counter
                        in_flight.fetch_sub(1, Ordering::SeqCst);
                    }
                }
            }

            println!("[INFO] Worker terminated for workflow: {}", workflow_uuid);
        });

        Ok(())
    }

    // ---- Lease management helpers (static, operate on PgPool) ----

    /// Acquire a lease on the oldest pending signal for a given workflow.
    /// Sets status = 'dispatched', leased_at = NOW(), lease_expires_at = NOW() + interval.
    async fn acquire_signal_lease(
        pool: &PgPool,
        workflow_id: i32,
        lease_duration_secs: i64,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE signals
            SET status = 'dispatched',
                leased_at = NOW(),
                lease_expires_at = NOW() + make_interval(secs => $2::double precision),
                updated_at = NOW()
            WHERE id = (
                SELECT id FROM signals
                WHERE workflow_id = $1 AND status = 'pending'
                ORDER BY created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            "#,
        )
        .bind(workflow_id)
        .bind(lease_duration_secs as f64)
        .execute(pool)
        .await
        .map_err(|e| format!("acquire_signal_lease: {}", e))?;

        Ok(())
    }

    /// Mark a dispatched signal as completed and clear the lease.
    async fn complete_signal_lease(pool: &PgPool, workflow_id: i32) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE signals
            SET status = 'completed',
                leased_at = NULL,
                lease_expires_at = NULL,
                updated_at = NOW()
            WHERE id = (
                SELECT id FROM signals
                WHERE workflow_id = $1 AND status = 'dispatched'
                ORDER BY created_at ASC
                LIMIT 1
            )
            "#,
        )
        .bind(workflow_id)
        .execute(pool)
        .await
        .map_err(|e| format!("complete_signal_lease: {}", e))?;

        Ok(())
    }

    /// Mark a dispatched signal as failed and clear the lease.
    async fn fail_signal_lease(
        pool: &PgPool,
        workflow_id: i32,
        error_msg: &str,
    ) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE signals
            SET status = 'failed',
                leased_at = NULL,
                lease_expires_at = NULL,
                error_message = $2,
                updated_at = NOW()
            WHERE id = (
                SELECT id FROM signals
                WHERE workflow_id = $1 AND status = 'dispatched'
                ORDER BY created_at ASC
                LIMIT 1
            )
            "#,
        )
        .bind(workflow_id)
        .bind(error_msg)
        .execute(pool)
        .await
        .map_err(|e| format!("fail_signal_lease: {}", e))?;

        Ok(())
    }

    /// Graceful shutdown.
    ///
    /// 1. Signals all worker tasks to stop accepting new work via the CancellationToken.
    /// 2. Waits up to 30 seconds for in-flight signals to finish processing.
    /// 3. Marks any remaining dispatched signals as 'pending' so they are picked up on restart.
    /// 4. Logs progress throughout.
    pub async fn shutdown(&mut self) {
        println!("[INFO] Initiating graceful shutdown...");

        // Step 1: Signal cancellation to all workers
        self.cancel_token.cancel();
        println!("[INFO] Shutdown signal sent to all worker tasks");

        // Step 2: Wait for in-flight signals to drain (up to 30 seconds)
        let deadline = tokio::time::Instant::now()
            + tokio::time::Duration::from_secs(SHUTDOWN_DRAIN_TIMEOUT_SECS);

        loop {
            let remaining = self.in_flight_count.load(Ordering::SeqCst);
            if remaining == 0 {
                println!("[INFO] All in-flight signals have completed");
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                println!(
                    "[WARN] Shutdown drain timeout reached with {} signals still in-flight",
                    remaining
                );
                break;
            }
            println!(
                "[INFO] Waiting for {} in-flight signal(s) to complete...",
                remaining
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        // Step 3: Drop all queue senders so worker loops exit
        let queue_count = self.message_queues.len();
        self.message_queues.clear();
        println!(
            "[INFO] Closed {} workflow queue channel(s)",
            queue_count
        );

        // Step 4: Mark remaining dispatched signals as pending in the database
        match sqlx::query(
            r#"
            UPDATE signals
            SET status = 'pending', leased_at = NULL, lease_expires_at = NULL, updated_at = NOW()
            WHERE status = 'dispatched'
            "#,
        )
        .execute(&self.db_pool)
        .await
        {
            Ok(result) => {
                println!(
                    "[INFO] Reset {} dispatched signals to pending for recovery on next startup",
                    result.rows_affected()
                );
            }
            Err(e) => {
                println!(
                    "[ERROR] Failed to reset dispatched signals during shutdown: {}",
                    e
                );
            }
        }

        println!("[INFO] Graceful shutdown complete");
    }

    // Queue a signal for a specific workflow
    pub async fn queue_signal(&self, signal_request: SignalRequest) -> Result<SignalResponse, Status> {
        // Map the workflow_id to global UUID
        let workflow_uuid = if signal_request.workflow_id != 0 {
            // Look up by local ID
            if let Some(uuid) = self.local_id_map.get(&signal_request.workflow_id.to_string()) {
                uuid.clone()
            } else {
                return Err(Status::not_found(format!(
                    "Workflow with local ID {} not found",
                    signal_request.workflow_id
                )));
            }
        } else {
            return Err(Status::invalid_argument("workflow_id is required"));
        };

        // Get the message queue for this workflow
        if let Some(tx) = self.message_queues.get(&workflow_uuid) {
            match tx.send(signal_request).await {
                Ok(_) => Ok(SignalResponse {
                    success: true,
                    message: "Signal queued successfully".to_string(),
                    runtime_session_uuid: "".to_string(), // Will be filled by worker
                    result_data: None,
                }),
                Err(_) => Err(Status::internal("Failed to queue signal")),
            }
        } else {
            Err(Status::not_found(format!(
                "No queue found for workflow: {}",
                workflow_uuid
            )))
        }
    }

    // Process a signal for a specific workflow
    async fn process_workflow_signal(
        workflow_map: SharedWorkflowMap,
        signal_request: SignalRequest,
        pool: PgPool,
    ) -> Result<SignalResponse, Status> {
        match signal_request.signal_type() {
            SignalType::Run => match signal_request.payload {
                Some(crate::proto::signal_request::Payload::RunData(ref run_data)) => {
                    let json_data = proto_struct_to_json(run_data);
                    run::handle_run_signal(workflow_map, signal_request.workflow_id, json_data, pool).await
                }
                _ => Err(Status::invalid_argument("Missing run_data for RUN signal"))
            },
            SignalType::Sync => match signal_request.payload {
                Some(crate::proto::signal_request::Payload::Sync(ref sync_payload)) => {
                    sync::handle_sync_signal(workflow_map, sync_payload.clone(), pool).await
                }
                _ => Err(Status::invalid_argument("Missing sync payload for SYNC signal"))
            },
            SignalType::Fyi => match signal_request.payload {
                Some(crate::proto::signal_request::Payload::FyiData(ref fyi_data)) => {
                    let json_data = proto_struct_to_json(fyi_data);
                    fyi::handle_fyi_signal(workflow_map, signal_request.workflow_id, json_data, pool).await
                }
                _ => Err(Status::invalid_argument("Missing fyi_data for FYI signal"))
            },
        }
    }

    // Add a new workflow to the system
    pub async fn add_workflow(&mut self, workflow_uuid: String) -> Result<(), Status> {
        // Set up a queue for the new workflow
        self.setup_workflow_queue(workflow_uuid).await?;
        println!("[INFO] Added new workflow queue");
        Ok(())
    }

    // Remove a workflow from the system
    pub async fn remove_workflow(&mut self, workflow_uuid: &str) -> Result<(), Status> {
        // Remove from workflows map
        {
            let mut workflows = self.workflows.write().await;
            workflows.remove(workflow_uuid);
        }

        // Remove local ID mapping if exists
        self.local_id_map.retain(|_k, v| v != workflow_uuid);

        // Remove message queue (dropping the sender will close the channel)
        self.message_queues.remove(workflow_uuid);

        println!("[INFO] Removed workflow: {}", workflow_uuid);
        Ok(())
    }

    // Get workflow count
    pub async fn workflow_count(&self) -> usize {
        let workflows = self.workflows.read().await;
        workflows.len()
    }
}

/// Lightweight row type for queue reconstruction queries.
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct SignalRow {
    id: i64,
    workflow_id: Option<i32>,
    signal_type: String,
    initial_data: Option<serde_json::Value>,
}
