use std::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use thiserror::Error;
use serde_json::Value;

use crate::DataSource;
use super::agents::Agent;

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Job timed out after {0:?}")]
    Timeout(Duration),

    #[error("Job failed after {attempts} attempts: {message}")]
    ExecutionFailed {
        attempts: u32,
        message: String,
    },

    #[error("Job was cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Delay between retries
    #[serde(with = "humantime_serde")]
    pub retry_delay: Duration,
    /// Whether to use exponential backoff
    pub use_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay: Duration::from_secs(5),
            use_backoff: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job
    pub id: u64,
    /// When the job was created
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    /// Optional user identifier who created the job
    pub user_id: Option<u64>,
    /// Description of the job's goal
    pub description: String,
    /// ID of the agent to run
    pub agent_id: String,
    /// Input data for the agent
    pub input: DataSource,
    /// Current status of the job
    pub status: JobStatus,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Maximum runtime duration
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// When the job completed (successfully or not)
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if job failed
    pub error_message: Option<String>,
}

impl Job {
    /// Creates a new job builder with required fields
    pub fn builder(
        description: String,
        agent_id: String,
        input: DataSource,
    ) -> Self {
        Job {
            id: rand::random(),
            created_at: Utc::now(),
            user_id: None,
            description,
            agent_id,
            input,
            status: JobStatus::Pending,
            retry_config: RetryConfig::default(),
            timeout: Duration::from_secs(300),
            completed_at: None,
            error_message: None,
        }
    }

    /// Updates job status and timestamps
    pub fn update_status(&mut self, status: JobStatus, error: Option<String>) {
        self.status = status;
        match status {
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
                self.completed_at = Some(Utc::now());
                self.error_message = error;
            }
            _ => {}
        }
    }

    pub fn update_status_from_result(&mut self, result: &Result<Value, anyhow::Error>) {
        match result {
            Ok(_) => self.update_status(JobStatus::Completed, None),
            Err(e) => self.update_status(JobStatus::Failed, Some(e.to_string())),
        }
    }

    /// Executes the job with retry logic
    pub async fn execute(&mut self, agent: &mut Agent) -> Result<serde_json::Value, JobError> {
        self.update_status(JobStatus::Running, None);

        let mut attempts = 0;
        let mut delay = self.retry_config.retry_delay;

        while attempts < self.retry_config.max_attempts {
            match tokio::time::timeout(
                self.timeout,
                agent.run(self.input.clone(), Some(self))
            ).await {
                Ok(Ok(result)) => {
                    self.update_status(JobStatus::Completed, None);
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    attempts += 1;

                    if attempts >= self.retry_config.max_attempts {
                        let message = e.to_string();
                        self.update_status(JobStatus::Failed, Some(message.clone()));
                        return Err(JobError::ExecutionFailed {
                            attempts,
                            message,
                        });
                    }

                    if self.retry_config.use_backoff {
                        delay *= 2;  // Exponential backoff
                    }
                    tokio::time::sleep(delay).await;
                }
                Err(_) => {
                    self.update_status(JobStatus::Failed, Some("Job timed out".to_string()));
                    return Err(JobError::Timeout(self.timeout));
                }
            }
        }

        unreachable!("Loop should return before reaching here")
    }
}
