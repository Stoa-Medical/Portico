use anyhow::Result;
use portico_database::OutboxEvent;
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;

use crate::services::fhir_client::MedplumClient;

/// Background outbox publisher that polls for pending outbox events
/// and forwards them to the Medplum FHIR server.
///
/// Implements the transactional outbox pattern: events are written to the
/// database in the same transaction as domain changes, then this publisher
/// picks them up asynchronously and delivers them.
pub struct OutboxPublisher {
    pool: PgPool,
    medplum_client: MedplumClient,
    /// Base interval between polls (increases with exponential backoff on failure)
    poll_interval: Duration,
    /// Maximum backoff duration
    max_backoff: Duration,
}

impl OutboxPublisher {
    pub fn new(pool: PgPool, medplum_client: MedplumClient) -> Self {
        Self {
            pool,
            medplum_client,
            poll_interval: Duration::from_secs(5),
            max_backoff: Duration::from_secs(300), // 5 minutes max
        }
    }

    /// Run the outbox publisher loop. Designed to be spawned as a background
    /// task via `tokio::spawn`.
    ///
    /// Polls for unpublished outbox events, sends each to Medplum, and marks
    /// them as published. On failure, applies exponential backoff before retrying.
    pub async fn run(&mut self) -> Result<()> {
        println!("[INFO] Outbox publisher started");

        let mut consecutive_failures: u32 = 0;

        loop {
            // Authenticate before each poll cycle (token may have expired)
            if let Err(e) = self.medplum_client.authenticate().await {
                eprintln!("[ERROR] Outbox publisher failed to authenticate: {}", e);
                consecutive_failures += 1;
                let backoff = self.calculate_backoff(consecutive_failures);
                tokio::time::sleep(backoff).await;
                continue;
            }

            match self.process_pending_events().await {
                Ok(count) => {
                    if count > 0 {
                        println!("[INFO] Outbox publisher processed {} events", count);
                    }
                    consecutive_failures = 0;
                    tokio::time::sleep(self.poll_interval).await;
                }
                Err(e) => {
                    eprintln!("[ERROR] Outbox publisher error: {}", e);
                    consecutive_failures += 1;
                    let backoff = self.calculate_backoff(consecutive_failures);
                    eprintln!(
                        "[INFO] Outbox publisher backing off for {:.1}s (failure #{})",
                        backoff.as_secs_f64(),
                        consecutive_failures
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    /// Process all pending (unpublished) outbox events.
    /// Returns the number of events successfully processed.
    async fn process_pending_events(&self) -> Result<usize> {
        let events = OutboxEvent::select_unpublished(&self.pool).await?;
        let mut processed = 0;

        for event in &events {
            match self.publish_event(event).await {
                Ok(()) => {
                    if let Some(id) = event.id {
                        OutboxEvent::mark_published(&self.pool, id).await?;
                    }
                    processed += 1;
                }
                Err(e) => {
                    eprintln!(
                        "[ERROR] Failed to publish outbox event {:?} (type={}): {}",
                        event.id, event.event_type, e
                    );
                    // Continue processing remaining events; failed ones will be
                    // retried on the next poll cycle.
                }
            }
        }

        Ok(processed)
    }

    /// Publish a single outbox event to Medplum based on its event type.
    async fn publish_event(&self, event: &OutboxEvent) -> Result<()> {
        let resource_type = &event.aggregate_type;
        let payload = &event.payload;

        match event.event_type.as_str() {
            "create" => {
                self.medplum_client
                    .create_resource(resource_type, payload)
                    .await?;
            }
            "transaction" => {
                self.medplum_client.transaction(payload).await?;
            }
            other => {
                println!(
                    "[WARN] Outbox publisher: unknown event type '{}', skipping",
                    other
                );
            }
        }

        Ok(())
    }

    /// Calculate exponential backoff duration based on consecutive failure count.
    fn calculate_backoff(&self, failures: u32) -> Duration {
        let base_secs = self.poll_interval.as_secs_f64();
        let backoff_secs = base_secs * 2.0_f64.powi(failures.min(10) as i32);
        let capped = Duration::from_secs_f64(backoff_secs.min(self.max_backoff.as_secs_f64()));
        capped
    }
}
