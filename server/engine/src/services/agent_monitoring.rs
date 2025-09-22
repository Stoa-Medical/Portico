/// Agent statistics and monitoring service for tracking performance and usage
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Comprehensive agent statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub agent_id: i32,
    pub agent_name: String,
    pub total_workflows_planned: u64,
    pub successful_workflows: u64,
    pub failed_workflows: u64,
    pub workflows_requiring_approval: u64,
    pub avg_planning_time_ms: f64,
    pub last_activity: Option<String>, // ISO timestamp
    pub capabilities_utilization: HashMap<String, u64>,
    pub error_categories: HashMap<String, u64>,
}

/// Individual workflow planning metrics
#[derive(Debug, Clone)]
pub struct PlanningMetrics {
    pub workflow_uuid: String,
    pub agent_id: i32,
    pub objective: String,
    pub planning_duration: Duration,
    pub success: bool,
    pub estimated_steps: u32,
    pub requires_approval: bool,
    pub error_type: Option<String>,
    pub timestamp: Instant,
}

/// System-wide monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_agents: u64,
    pub active_agents_last_hour: u64,
    pub total_workflows_planned: u64,
    pub success_rate_percent: f64,
    pub avg_planning_time_ms: f64,
    pub peak_concurrent_planning: u64,
    pub most_active_agent_id: Option<i32>,
    pub most_used_tools: Vec<(String, u64)>,
    pub approval_rate_percent: f64,
}

/// Agent monitoring service
pub struct AgentMonitoringService {
    agent_stats: RwLock<HashMap<i32, AgentStats>>,
    recent_planning_metrics: RwLock<Vec<PlanningMetrics>>,
    system_counters: RwLock<SystemCounters>,
}

#[derive(Debug, Default)]
struct SystemCounters {
    total_planning_requests: u64,
    successful_planning: u64,
    failed_planning: u64,
    total_planning_time_ms: u64,
    peak_concurrent: u64,
    current_concurrent: u64,
    agents_seen: std::collections::HashSet<i32>,
}

impl AgentMonitoringService {
    pub fn new() -> Self {
        Self {
            agent_stats: RwLock::new(HashMap::new()),
            recent_planning_metrics: RwLock::new(Vec::new()),
            system_counters: RwLock::new(SystemCounters::default()),
        }
    }

    /// Record a workflow planning attempt
    pub async fn record_planning_attempt(
        &self,
        agent_id: i32,
        agent_name: String,
        objective: String,
        planning_duration: Duration,
        success: bool,
        estimated_steps: u32,
        requires_approval: bool,
        error_type: Option<String>,
        tools_used: Vec<String>,
    ) {
        let workflow_uuid = Uuid::new_v4().to_string();
        let planning_time_ms = planning_duration.as_millis() as u64;

        // Record individual metrics
        {
            let mut metrics = self.recent_planning_metrics.write().await;
            metrics.push(PlanningMetrics {
                workflow_uuid,
                agent_id,
                objective,
                planning_duration,
                success,
                estimated_steps,
                requires_approval,
                error_type: error_type.clone(),
                timestamp: Instant::now(),
            });

            // Keep only recent metrics (last 1000 entries)
            let excess = metrics.len().saturating_sub(1000);
            if excess > 0 {
                metrics.drain(..excess);
            }
        }

        // Update agent-specific stats
        {
            let mut agent_stats = self.agent_stats.write().await;
            let stats = agent_stats.entry(agent_id).or_insert_with(|| AgentStats {
                agent_id,
                agent_name: agent_name.clone(),
                total_workflows_planned: 0,
                successful_workflows: 0,
                failed_workflows: 0,
                workflows_requiring_approval: 0,
                avg_planning_time_ms: 0.0,
                last_activity: None,
                capabilities_utilization: HashMap::new(),
                error_categories: HashMap::new(),
            });

            stats.total_workflows_planned += 1;
            if success {
                stats.successful_workflows += 1;
            } else {
                stats.failed_workflows += 1;
                if let Some(error) = &error_type {
                    *stats.error_categories.entry(error.clone()).or_insert(0) += 1;
                }
            }

            if requires_approval {
                stats.workflows_requiring_approval += 1;
            }

            // Update average planning time
            let total_time = stats.avg_planning_time_ms * (stats.total_workflows_planned - 1) as f64;
            stats.avg_planning_time_ms = (total_time + planning_time_ms as f64) / stats.total_workflows_planned as f64;

            // Update last activity
            stats.last_activity = Some(chrono::Utc::now().to_rfc3339());

            // Track tool utilization
            for tool in tools_used {
                *stats.capabilities_utilization.entry(tool).or_insert(0) += 1;
            }
        }

        // Update system counters
        {
            let mut counters = self.system_counters.write().await;
            counters.total_planning_requests += 1;
            if success {
                counters.successful_planning += 1;
            } else {
                counters.failed_planning += 1;
            }
            counters.total_planning_time_ms += planning_time_ms;
            counters.agents_seen.insert(agent_id);
        }
    }

    /// Get statistics for a specific agent
    pub async fn get_agent_stats(&self, agent_id: i32) -> Option<AgentStats> {
        let agent_stats = self.agent_stats.read().await;
        agent_stats.get(&agent_id).cloned()
    }

    /// Get statistics for all agents
    pub async fn get_all_agent_stats(&self) -> HashMap<i32, AgentStats> {
        let agent_stats = self.agent_stats.read().await;
        agent_stats.clone()
    }

    /// Get system-wide statistics
    pub async fn get_system_stats(&self) -> SystemStats {
        let counters = self.system_counters.read().await;
        let agent_stats = self.agent_stats.read().await;

        // Calculate success rate
        let success_rate = if counters.total_planning_requests > 0 {
            (counters.successful_planning as f64 / counters.total_planning_requests as f64) * 100.0
        } else {
            0.0
        };

        // Calculate average planning time
        let avg_planning_time = if counters.total_planning_requests > 0 {
            counters.total_planning_time_ms as f64 / counters.total_planning_requests as f64
        } else {
            0.0
        };

        // Find most active agent
        let most_active_agent_id = agent_stats
            .iter()
            .max_by_key(|(_, stats)| stats.total_workflows_planned)
            .map(|(id, _)| *id);

        // Collect most used tools across all agents
        let mut tool_usage: HashMap<String, u64> = HashMap::new();
        for (_, stats) in agent_stats.iter() {
            for (tool, count) in &stats.capabilities_utilization {
                *tool_usage.entry(tool.clone()).or_insert(0) += count;
            }
        }

        let mut most_used_tools: Vec<(String, u64)> = tool_usage.into_iter().collect();
        most_used_tools.sort_by(|a, b| b.1.cmp(&a.1));
        most_used_tools.truncate(10); // Top 10 tools

        // Calculate approval rate
        let total_approvals: u64 = agent_stats.values().map(|s| s.workflows_requiring_approval).sum();
        let approval_rate = if counters.total_planning_requests > 0 {
            (total_approvals as f64 / counters.total_planning_requests as f64) * 100.0
        } else {
            0.0
        };

        SystemStats {
            total_agents: agent_stats.len() as u64,
            active_agents_last_hour: self.count_active_agents_last_hour().await,
            total_workflows_planned: counters.total_planning_requests,
            success_rate_percent: success_rate,
            avg_planning_time_ms: avg_planning_time,
            peak_concurrent_planning: counters.peak_concurrent,
            most_active_agent_id,
            most_used_tools,
            approval_rate_percent: approval_rate,
        }
    }

    /// Count agents that were active in the last hour
    async fn count_active_agents_last_hour(&self) -> u64 {
        let metrics = self.recent_planning_metrics.read().await;
        let one_hour_ago = Instant::now() - Duration::from_secs(3600);

        let active_agents: std::collections::HashSet<i32> = metrics
            .iter()
            .filter(|m| m.timestamp > one_hour_ago)
            .map(|m| m.agent_id)
            .collect();

        active_agents.len() as u64
    }

    /// Get recent planning failures for debugging
    pub async fn get_recent_failures(&self, limit: usize) -> Vec<PlanningMetrics> {
        let metrics = self.recent_planning_metrics.read().await;
        metrics
            .iter()
            .filter(|m| !m.success)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get performance trends over time
    pub async fn get_performance_trends(&self) -> PerformanceTrends {
        let metrics = self.recent_planning_metrics.read().await;
        let now = Instant::now();

        // Group metrics by time buckets (15-minute intervals)
        let mut buckets: HashMap<u64, Vec<&PlanningMetrics>> = HashMap::new();
        for metric in metrics.iter() {
            let minutes_ago = now.duration_since(metric.timestamp).as_secs() / 60;
            let bucket = minutes_ago / 15; // 15-minute buckets
            buckets.entry(bucket).or_default().push(metric);
        }

        let mut trends = Vec::new();
        for (bucket, bucket_metrics) in buckets {
            let success_count = bucket_metrics.iter().filter(|m| m.success).count();
            let avg_time = bucket_metrics.iter()
                .map(|m| m.planning_duration.as_millis() as f64)
                .sum::<f64>() / bucket_metrics.len() as f64;

            trends.push(TrendPoint {
                time_bucket: bucket * 15, // minutes ago
                success_rate: (success_count as f64 / bucket_metrics.len() as f64) * 100.0,
                avg_planning_time_ms: avg_time,
                request_count: bucket_metrics.len() as u64,
            });
        }

        // Sort by time (most recent first)
        trends.sort_by(|a, b| a.time_bucket.cmp(&b.time_bucket));

        PerformanceTrends { trends }
    }

    /// Clean up old metrics to prevent memory growth
    pub async fn cleanup_old_metrics(&self) {
        let mut metrics = self.recent_planning_metrics.write().await;
        let cutoff = Instant::now() - Duration::from_secs(3600 * 24); // Keep 24 hours
        metrics.retain(|m| m.timestamp > cutoff);
    }
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceTrends {
    pub trends: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrendPoint {
    pub time_bucket: u64, // minutes ago
    pub success_rate: f64,
    pub avg_planning_time_ms: f64,
    pub request_count: u64,
}

impl Default for AgentMonitoringService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_agent_monitoring() {
        let monitor = AgentMonitoringService::new();

        // Record some planning attempts
        monitor.record_planning_attempt(
            1,
            "Test Agent".to_string(),
            "Test objective".to_string(),
            Duration::from_millis(150),
            true,
            3,
            false,
            None,
            vec!["python".to_string()],
        ).await;

        monitor.record_planning_attempt(
            1,
            "Test Agent".to_string(),
            "Another objective".to_string(),
            Duration::from_millis(250),
            false,
            0,
            false,
            Some("validation_error".to_string()),
            vec!["webscrape".to_string()],
        ).await;

        // Get agent stats
        let stats = monitor.get_agent_stats(1).await.unwrap();
        assert_eq!(stats.total_workflows_planned, 2);
        assert_eq!(stats.successful_workflows, 1);
        assert_eq!(stats.failed_workflows, 1);

        // Get system stats
        let system_stats = monitor.get_system_stats().await;
        assert_eq!(system_stats.total_workflows_planned, 2);
        assert_eq!(system_stats.success_rate_percent, 50.0);
    }
}
