use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// Garbage collection policies for ephemeral workflows and agent resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionPolicy {
    /// How long to keep completed ephemeral workflows (in hours)
    pub ephemeral_retention_hours: i64,
    /// How long to keep failed workflows before cleanup (in hours)
    pub failed_retention_hours: i64,
    /// Maximum number of completed workflows to keep per agent
    pub max_completed_per_agent: Option<i64>,
    /// Whether to clean up orphaned steps
    pub cleanup_orphaned_steps: bool,
    /// Whether to clean up old runtime sessions
    pub cleanup_old_sessions: bool,
    /// How long to keep runtime session logs (in days)
    pub session_log_retention_days: i64,
    /// Custom cleanup rules
    pub custom_rules: Value,
}

impl Default for GarbageCollectionPolicy {
    fn default() -> Self {
        Self {
            ephemeral_retention_hours: 1,
            failed_retention_hours: 24,
            max_completed_per_agent: Some(100),
            cleanup_orphaned_steps: true,
            cleanup_old_sessions: true,
            session_log_retention_days: 7,
            custom_rules: Value::Null,
        }
    }
}

/// Garbage collection manager for agent-created resources
pub struct GarbageCollector {
    pool: PgPool,
    policy: GarbageCollectionPolicy,
}

/// Statistics from garbage collection operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcStatistics {
    pub ephemeral_workflows_cleaned: u64,
    pub failed_workflows_cleaned: u64,
    pub orphaned_steps_cleaned: u64,
    pub old_sessions_cleaned: u64,
    pub total_space_freed_bytes: Option<u64>,
    pub cleanup_duration_ms: u64,
    pub errors_encountered: Vec<String>,
}

impl GarbageCollector {
    pub fn new(pool: PgPool, policy: GarbageCollectionPolicy) -> Self {
        Self { pool, policy }
    }

    /// Run a complete garbage collection cycle
    pub async fn run_full_cleanup(&self) -> Result<GcStatistics> {
        let start_time = Utc::now();
        let mut stats = GcStatistics {
            ephemeral_workflows_cleaned: 0,
            failed_workflows_cleaned: 0,
            orphaned_steps_cleaned: 0,
            old_sessions_cleaned: 0,
            total_space_freed_bytes: None,
            cleanup_duration_ms: 0,
            errors_encountered: Vec::new(),
        };

        // Clean up ephemeral workflows
        match self.cleanup_ephemeral_workflows().await {
            Ok(count) => stats.ephemeral_workflows_cleaned = count,
            Err(e) => stats.errors_encountered.push(format!("Ephemeral cleanup failed: {}", e)),
        }

        // Clean up failed workflows
        match self.cleanup_failed_workflows().await {
            Ok(count) => stats.failed_workflows_cleaned = count,
            Err(e) => stats.errors_encountered.push(format!("Failed workflow cleanup failed: {}", e)),
        }

        // Clean up orphaned steps
        if self.policy.cleanup_orphaned_steps {
            match self.cleanup_orphaned_steps().await {
                Ok(count) => stats.orphaned_steps_cleaned = count,
                Err(e) => stats.errors_encountered.push(format!("Orphaned steps cleanup failed: {}", e)),
            }
        }

        // Clean up old runtime sessions
        if self.policy.cleanup_old_sessions {
            match self.cleanup_old_runtime_sessions().await {
                Ok(count) => stats.old_sessions_cleaned = count,
                Err(e) => stats.errors_encountered.push(format!("Session cleanup failed: {}", e)),
            }
        }

        // Apply agent-specific cleanup rules
        if let Err(e) = self.apply_agent_specific_cleanup(&mut stats).await {
            stats.errors_encountered.push(format!("Agent-specific cleanup failed: {}", e));
        }

        let end_time = Utc::now();
        stats.cleanup_duration_ms = (end_time - start_time).num_milliseconds() as u64;

        Ok(stats)
    }

    /// Clean up completed ephemeral workflows
    async fn cleanup_ephemeral_workflows(&self) -> Result<u64> {
        let cutoff_time = Utc::now() - Duration::hours(self.policy.ephemeral_retention_hours);

        // Find ephemeral workflows that are completed and old enough
        let workflows_to_delete = sqlx::query!(
            r#"
            SELECT w.id, w.global_uuid
            FROM workflows w
            JOIN runtime_sessions rs ON rs.workflow_id = w.id
            WHERE w.is_ephemeral = true
            AND rs.rts_status = 'completed'
            AND rs.updated_at < $1
            "#,
            cutoff_time
        )
        .fetch_all(&self.pool)
        .await?;

        let mut deleted_count = 0u64;

        for workflow in workflows_to_delete {
            if let Err(e) = self.delete_workflow_and_dependencies(workflow.id).await {
                eprintln!("Failed to delete ephemeral workflow {}: {}", workflow.id, e);
            } else {
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Clean up failed workflows after retention period
    async fn cleanup_failed_workflows(&self) -> Result<u64> {
        let cutoff_time = Utc::now() - Duration::hours(self.policy.failed_retention_hours);

        // Find workflows with failed runtime sessions
        let workflows_to_delete = sqlx::query!(
            r#"
            SELECT DISTINCT w.id
            FROM workflows w
            JOIN runtime_sessions rs ON rs.workflow_id = w.id
            WHERE rs.rts_status = 'cancelled'
            AND rs.updated_at < $1
            AND NOT EXISTS (
                SELECT 1 FROM runtime_sessions rs2
                WHERE rs2.workflow_id = w.id
                AND rs2.rts_status IN ('running', 'waiting')
            )
            "#,
            cutoff_time
        )
        .fetch_all(&self.pool)
        .await?;

        let mut deleted_count = 0u64;

        for workflow in workflows_to_delete {
            if let Err(e) = self.delete_workflow_and_dependencies(workflow.id).await {
                eprintln!("Failed to delete failed workflow {}: {}", workflow.id, e);
            } else {
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Clean up orphaned steps that don't belong to any workflow
    async fn cleanup_orphaned_steps(&self) -> Result<u64> {
        let deleted_count = sqlx::query!(
            r#"
            DELETE FROM steps
            WHERE workflow_id NOT IN (SELECT id FROM workflows)
            "#
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(deleted_count)
    }

    /// Clean up old runtime sessions
    async fn cleanup_old_runtime_sessions(&self) -> Result<u64> {
        let cutoff_time = Utc::now() - Duration::days(self.policy.session_log_retention_days);

        let deleted_count = sqlx::query!(
            r#"
            DELETE FROM runtime_sessions
            WHERE rts_status = 'completed'
            AND updated_at < $1
            "#,
            cutoff_time
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(deleted_count)
    }

    /// Delete a workflow and all its dependencies
    async fn delete_workflow_and_dependencies(&self, workflow_id: i32) -> Result<()> {
        // Delete in dependency order: signals -> runtime_sessions -> steps -> workflow

        // Delete signals
        sqlx::query!(
            "DELETE FROM signals WHERE workflow_id = $1",
            workflow_id
        )
        .execute(&self.pool)
        .await?;

        // Delete runtime sessions
        sqlx::query!(
            "DELETE FROM runtime_sessions WHERE workflow_id = $1",
            workflow_id
        )
        .execute(&self.pool)
        .await?;

        // Delete steps
        sqlx::query!(
            "DELETE FROM steps WHERE workflow_id = $1",
            workflow_id
        )
        .execute(&self.pool)
        .await?;

        // Delete workflow
        sqlx::query!(
            "DELETE FROM workflows WHERE id = $1",
            workflow_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Apply agent-specific cleanup rules
    async fn apply_agent_specific_cleanup(&self, stats: &mut GcStatistics) -> Result<()> {
        if let Some(max_per_agent) = self.policy.max_completed_per_agent {
            let agents_with_excess = sqlx::query!(
                r#"
                SELECT
                    w.created_by_agent_id,
                    COUNT(*) as workflow_count
                FROM workflows w
                JOIN runtime_sessions rs ON rs.workflow_id = w.id
                WHERE w.created_by_agent_id IS NOT NULL
                AND rs.rts_status = 'completed'
                GROUP BY w.created_by_agent_id
                HAVING COUNT(*) > $1
                "#,
                max_per_agent
            )
            .fetch_all(&self.pool)
            .await?;

            for agent_data in agents_with_excess {
                if let Some(agent_id) = agent_data.created_by_agent_id {
                    let excess_count = agent_data.workflow_count - max_per_agent;

                    // Delete oldest completed workflows for this agent
                    let workflows_to_delete = sqlx::query!(
                        r#"
                        SELECT w.id
                        FROM workflows w
                        JOIN runtime_sessions rs ON rs.workflow_id = w.id
                        WHERE w.created_by_agent_id = $1
                        AND rs.rts_status = 'completed'
                        ORDER BY rs.updated_at ASC
                        LIMIT $2
                        "#,
                        agent_id,
                        excess_count
                    )
                    .fetch_all(&self.pool)
                    .await?;

                    for workflow in workflows_to_delete {
                        if let Err(e) = self.delete_workflow_and_dependencies(workflow.id).await {
                            stats.errors_encountered.push(
                                format!("Failed to delete excess workflow {} for agent {}: {}",
                                       workflow.id, agent_id, e)
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get garbage collection statistics without running cleanup
    pub async fn get_cleanup_candidates(&self) -> Result<CleanupCandidates> {
        let ephemeral_cutoff = Utc::now() - Duration::hours(self.policy.ephemeral_retention_hours);
        let failed_cutoff = Utc::now() - Duration::hours(self.policy.failed_retention_hours);
        let session_cutoff = Utc::now() - Duration::days(self.policy.session_log_retention_days);

        let ephemeral_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM workflows w
            JOIN runtime_sessions rs ON rs.workflow_id = w.id
            WHERE w.is_ephemeral = true
            AND rs.rts_status = 'completed'
            AND rs.updated_at < $1
            "#,
            ephemeral_cutoff
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let failed_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT w.id)
            FROM workflows w
            JOIN runtime_sessions rs ON rs.workflow_id = w.id
            WHERE rs.rts_status = 'cancelled'
            AND rs.updated_at < $1
            "#,
            failed_cutoff
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let orphaned_steps_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM steps WHERE workflow_id NOT IN (SELECT id FROM workflows)"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let old_sessions_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM runtime_sessions WHERE rts_status = 'completed' AND updated_at < $1",
            session_cutoff
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(CleanupCandidates {
            ephemeral_workflows: ephemeral_count,
            failed_workflows: failed_count,
            orphaned_steps: orphaned_steps_count,
            old_sessions: old_sessions_count,
        })
    }

    /// Update garbage collection policy
    pub fn update_policy(&mut self, new_policy: GarbageCollectionPolicy) {
        self.policy = new_policy;
    }

    /// Get current policy
    pub fn get_policy(&self) -> &GarbageCollectionPolicy {
        &self.policy
    }
}

/// Summary of items that would be cleaned up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupCandidates {
    pub ephemeral_workflows: i64,
    pub failed_workflows: i64,
    pub orphaned_steps: i64,
    pub old_sessions: i64,
}

impl CleanupCandidates {
    pub fn total_items(&self) -> i64 {
        self.ephemeral_workflows + self.failed_workflows + self.orphaned_steps + self.old_sessions
    }

    pub fn is_empty(&self) -> bool {
        self.total_items() == 0
    }
}

/// Periodic garbage collection scheduler
pub struct GcScheduler {
    collector: GarbageCollector,
    interval_hours: u64,
}

impl GcScheduler {
    pub fn new(collector: GarbageCollector, interval_hours: u64) -> Self {
        Self {
            collector,
            interval_hours,
        }
    }

    /// Run garbage collection on a schedule
    pub async fn start_scheduled_cleanup(&self) -> Result<()> {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(self.interval_hours * 3600));

        loop {
            interval.tick().await;

            match self.collector.run_full_cleanup().await {
                Ok(stats) => {
                    println!("[GC] Cleanup completed: {}", self.format_stats(&stats));

                    if !stats.errors_encountered.is_empty() {
                        for error in &stats.errors_encountered {
                            eprintln!("[GC ERROR] {}", error);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("[GC ERROR] Cleanup failed: {}", e);
                }
            }
        }
    }

    fn format_stats(&self, stats: &GcStatistics) -> String {
        format!(
            "ephemeral: {}, failed: {}, orphaned: {}, sessions: {}, duration: {}ms",
            stats.ephemeral_workflows_cleaned,
            stats.failed_workflows_cleaned,
            stats.orphaned_steps_cleaned,
            stats.old_sessions_cleaned,
            stats.cleanup_duration_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = GarbageCollectionPolicy::default();
        assert_eq!(policy.ephemeral_retention_hours, 1);
        assert_eq!(policy.failed_retention_hours, 24);
        assert_eq!(policy.max_completed_per_agent, Some(100));
        assert!(policy.cleanup_orphaned_steps);
        assert!(policy.cleanup_old_sessions);
    }

    #[test]
    fn test_cleanup_candidates_total() {
        let candidates = CleanupCandidates {
            ephemeral_workflows: 5,
            failed_workflows: 3,
            orphaned_steps: 10,
            old_sessions: 2,
        };

        assert_eq!(candidates.total_items(), 20);
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_empty_cleanup_candidates() {
        let candidates = CleanupCandidates {
            ephemeral_workflows: 0,
            failed_workflows: 0,
            orphaned_steps: 0,
            old_sessions: 0,
        };

        assert_eq!(candidates.total_items(), 0);
        assert!(candidates.is_empty());
    }
}
