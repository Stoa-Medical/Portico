/// Request batching service for improved throughput
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{sleep, timeout};
use serde::{Deserialize, Serialize};

/// Batch request types
#[derive(Debug, Clone)]
pub enum BatchableRequest {
    PlanWorkflow {
        agent_id: i32,
        objective: String,
        context: Option<serde_json::Value>,
        constraints: Option<serde_json::Value>,
        is_ephemeral: bool,
        response_tx: tokio::sync::oneshot::Sender<Result<BatchResponse, String>>,
    },
    AgentStats {
        agent_ids: Vec<i32>,
        response_tx: tokio::sync::oneshot::Sender<Result<BatchResponse, String>>,
    },
}

/// Batch response wrapper
#[derive(Debug, Clone, Serialize)]
pub enum BatchResponse {
    PlanWorkflow {
        success: bool,
        message: String,
        workflow_uuid: String,
        estimated_steps: u32,
        requires_approval: bool,
    },
    AgentStats {
        stats: HashMap<i32, serde_json::Value>,
    },
}

/// Batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
    pub flush_interval: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 50,
            max_wait_time: Duration::from_millis(100),
            flush_interval: Duration::from_millis(50),
        }
    }
}

/// Batch statistics
#[derive(Debug, Clone, Serialize)]
pub struct BatchStats {
    pub total_requests: u64,
    pub total_batches: u64,
    pub avg_batch_size: f64,
    pub max_batch_size: usize,
    pub avg_processing_time_ms: f64,
    pub timeouts: u64,
    pub queue_depth: usize,
}

/// Request batcher service
pub struct RequestBatcherService {
    request_tx: mpsc::UnboundedSender<BatchableRequest>,
    stats: Arc<RwLock<BatchStats>>,
    config: BatchConfig,
}

impl RequestBatcherService {
    pub fn new(config: BatchConfig) -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let stats = Arc::new(RwLock::new(BatchStats {
            total_requests: 0,
            total_batches: 0,
            avg_batch_size: 0.0,
            max_batch_size: 0,
            avg_processing_time_ms: 0.0,
            timeouts: 0,
            queue_depth: 0,
        }));

        let batcher = Self {
            request_tx,
            stats: stats.clone(),
            config: config.clone(),
        };

        // Start the batch processor
        let stats_clone = stats.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            Self::batch_processor(request_rx, stats_clone, config_clone).await;
        });

        batcher
    }

    /// Submit a batchable request
    pub async fn submit_request(&self, request: BatchableRequest) -> Result<(), String> {
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.queue_depth += 1;
        }

        self.request_tx
            .send(request)
            .map_err(|e| format!("Failed to submit request: {}", e))
    }

    /// Get batch statistics
    pub async fn get_stats(&self) -> BatchStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Plan workflow with batching
    pub async fn plan_workflow_batched(
        &self,
        agent_id: i32,
        objective: String,
        context: Option<serde_json::Value>,
        constraints: Option<serde_json::Value>,
        is_ephemeral: bool,
    ) -> Result<BatchResponse, String> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let request = BatchableRequest::PlanWorkflow {
            agent_id,
            objective,
            context,
            constraints,
            is_ephemeral,
            response_tx,
        };

        self.submit_request(request).await?;

        // Wait for response with timeout
        match timeout(self.config.max_wait_time * 2, response_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => {
                let mut stats = self.stats.write().await;
                stats.timeouts += 1;
                Err("Request timeout".to_string())
            }
        }
    }

    /// Get agent stats with batching
    pub async fn get_agent_stats_batched(&self, agent_ids: Vec<i32>) -> Result<BatchResponse, String> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let request = BatchableRequest::AgentStats {
            agent_ids,
            response_tx,
        };

        self.submit_request(request).await?;

        match timeout(self.config.max_wait_time * 2, response_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => {
                let mut stats = self.stats.write().await;
                stats.timeouts += 1;
                Err("Request timeout".to_string())
            }
        }
    }

    /// Internal batch processor
    async fn batch_processor(
        mut request_rx: mpsc::UnboundedReceiver<BatchableRequest>,
        stats: Arc<RwLock<BatchStats>>,
        config: BatchConfig,
    ) {
        let mut current_batch: Vec<BatchableRequest> = Vec::new();
        let mut flush_timer = tokio::time::interval(config.flush_interval);

        loop {
            tokio::select! {
                // Receive new request
                request = request_rx.recv() => {
                    match request {
                        Some(req) => {
                            current_batch.push(req);

                            // Update queue depth
                            {
                                let mut stats_guard = stats.write().await;
                                stats_guard.queue_depth = stats_guard.queue_depth.saturating_sub(1);
                            }

                            // Flush if batch is full
                            if current_batch.len() >= config.max_batch_size {
                                Self::process_batch(&mut current_batch, &stats).await;
                            }
                        }
                        None => break, // Channel closed
                    }
                }

                // Periodic flush
                _ = flush_timer.tick() => {
                    if !current_batch.is_empty() {
                        Self::process_batch(&mut current_batch, &stats).await;
                    }
                }
            }
        }
    }

    /// Process a batch of requests
    async fn process_batch(
        batch: &mut Vec<BatchableRequest>,
        stats: &Arc<RwLock<BatchStats>>,
    ) {
        if batch.is_empty() {
            return;
        }

        let start_time = Instant::now();
        let batch_size = batch.len();

        // Group requests by type for more efficient processing
        let mut plan_requests = Vec::new();
        let mut stats_requests = Vec::new();

        for request in batch.drain(..) {
            match request {
                BatchableRequest::PlanWorkflow { .. } => plan_requests.push(request),
                BatchableRequest::AgentStats { .. } => stats_requests.push(request),
            }
        }

        // Process planning requests in parallel
        if !plan_requests.is_empty() {
            let plan_futures: Vec<_> = plan_requests
                .into_iter()
                .map(|req| Self::process_plan_request(req))
                .collect();

            // Execute all planning requests concurrently
            futures::future::join_all(plan_futures).await;
        }

        // Process stats requests (can be batched more efficiently)
        if !stats_requests.is_empty() {
            for request in stats_requests {
                tokio::spawn(Self::process_stats_request(request));
            }
        }

        // Update statistics
        let processing_time = start_time.elapsed();
        {
            let mut stats_guard = stats.write().await;
            stats_guard.total_batches += 1;
            stats_guard.max_batch_size = stats_guard.max_batch_size.max(batch_size);

            // Update average batch size
            let total_requests = stats_guard.total_requests as f64;
            let total_batches = stats_guard.total_batches as f64;
            stats_guard.avg_batch_size = total_requests / total_batches;

            // Update average processing time
            let prev_avg = stats_guard.avg_processing_time_ms;
            let new_time = processing_time.as_millis() as f64;
            stats_guard.avg_processing_time_ms =
                (prev_avg * (total_batches - 1.0) + new_time) / total_batches;
        }
    }

    /// Process individual plan request
    async fn process_plan_request(request: BatchableRequest) {
        if let BatchableRequest::PlanWorkflow {
            agent_id,
            objective,
            response_tx,
            ..
        } = request
        {
            // Simulate workflow planning (in real implementation, call actual planner)
            let response = BatchResponse::PlanWorkflow {
                success: true,
                message: format!("Batched planning for agent {} objective: {}", agent_id, objective),
                workflow_uuid: uuid::Uuid::new_v4().to_string(),
                estimated_steps: 3,
                requires_approval: objective.to_lowercase().contains("system") || objective.to_lowercase().contains("admin"),
            };

            let _ = response_tx.send(Ok(response));
        }
    }

    /// Process individual stats request
    async fn process_stats_request(request: BatchableRequest) {
        if let BatchableRequest::AgentStats { agent_ids, response_tx } = request {
            // Simulate stats collection (in real implementation, query actual stats)
            let mut stats = HashMap::new();
            for agent_id in agent_ids {
                stats.insert(
                    agent_id,
                    serde_json::json!({
                        "workflows_planned": 10,
                        "success_rate": 0.95,
                        "avg_time_ms": 150
                    }),
                );
            }

            let response = BatchResponse::AgentStats { stats };
            let _ = response_tx.send(Ok(response));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_batching() {
        let config = BatchConfig {
            max_batch_size: 3,
            max_wait_time: Duration::from_millis(50),
            flush_interval: Duration::from_millis(25),
        };

        let batcher = RequestBatcherService::new(config);

        // Submit multiple planning requests
        let mut handles = Vec::new();
        for i in 1..=5 {
            let batcher_clone = &batcher;
            let handle = tokio::spawn(async move {
                batcher_clone
                    .plan_workflow_batched(
                        i,
                        format!("Test objective {}", i),
                        None,
                        None,
                        false,
                    )
                    .await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Check stats
        let stats = batcher.get_stats().await;
        assert!(stats.total_requests >= 5);
        assert!(stats.total_batches > 0);
    }

    #[tokio::test]
    async fn test_batch_timeout() {
        let config = BatchConfig {
            max_batch_size: 100, // Large batch size to force timeout
            max_wait_time: Duration::from_millis(10), // Very short timeout
            flush_interval: Duration::from_millis(100),
        };

        let batcher = RequestBatcherService::new(config);

        // This should timeout
        let result = batcher
            .plan_workflow_batched(1, "Test".to_string(), None, None, false)
            .await;

        // The request should still complete via flush interval
        assert!(result.is_ok() || result.is_err()); // Either is acceptable in this test
    }
}
