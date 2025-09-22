/// Agent caching service for improved performance
use portico_database::models::agents::{AgentCapabilities, AgentPolicy};
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cached agent information
#[derive(Debug, Clone, Serialize)]
pub struct CachedAgent {
    pub agent_id: i32,
    pub name: String,
    pub capabilities: AgentCapabilities,
    pub policy: AgentPolicy,
    #[serde(skip)]
    pub last_updated: Instant,
    pub access_count: u64,
    #[serde(skip)]
    pub last_accessed: Instant,
}


/// Cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub total_entries: u64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub memory_usage_estimate: u64,
}

/// Agent caching service
pub struct AgentCacheService {
    cache: RwLock<HashMap<i32, CachedAgent>>,
    stats: RwLock<CacheStats>,
    max_entries: usize,
    ttl: Duration,
}

impl AgentCacheService {
    pub fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(CacheStats {
                total_entries: 0,
                hit_rate: 0.0,
                miss_rate: 0.0,
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                evictions: 0,
                memory_usage_estimate: 0,
            }),
            max_entries,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get agent from cache
    pub async fn get_agent(&self, agent_id: i32) -> Option<CachedAgent> {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        let mut cache = self.cache.write().await;

        if let Some(agent) = cache.get_mut(&agent_id) {
            // Check if entry is still valid
            if agent.last_updated.elapsed() < self.ttl {
                agent.access_count += 1;
                agent.last_accessed = Instant::now();
                stats.cache_hits += 1;
                stats.hit_rate = stats.cache_hits as f64 / stats.total_requests as f64;
                return Some(agent.clone());
            } else {
                // Entry expired, remove it
                cache.remove(&agent_id);
                stats.total_entries = cache.len() as u64;
            }
        }

        stats.cache_misses += 1;
        stats.miss_rate = stats.cache_misses as f64 / stats.total_requests as f64;
        None
    }

    /// Store agent in cache
    pub async fn store_agent(&self, agent: CachedAgent) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        // Check if we need to evict entries
        if cache.len() >= self.max_entries {
            self.evict_lru(&mut cache, &mut stats).await;
        }

        cache.insert(agent.agent_id, agent);
        stats.total_entries = cache.len() as u64;
        stats.memory_usage_estimate = self.estimate_memory_usage(&cache);
    }

    /// Update agent capabilities in cache
    pub async fn update_agent_capabilities(&self, agent_id: i32, capabilities: AgentCapabilities) {
        let mut cache = self.cache.write().await;

        if let Some(agent) = cache.get_mut(&agent_id) {
            agent.capabilities = capabilities;
            agent.last_updated = Instant::now();
        }
    }

    /// Update agent policy in cache
    pub async fn update_agent_policy(&self, agent_id: i32, policy: AgentPolicy) {
        let mut cache = self.cache.write().await;

        if let Some(agent) = cache.get_mut(&agent_id) {
            agent.policy = policy;
            agent.last_updated = Instant::now();
        }
    }

    /// Remove agent from cache
    pub async fn remove_agent(&self, agent_id: i32) -> bool {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        let removed = cache.remove(&agent_id).is_some();
        if removed {
            stats.total_entries = cache.len() as u64;
            stats.memory_usage_estimate = self.estimate_memory_usage(&cache);
        }
        removed
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;
        let _now = Instant::now();

        let initial_count = cache.len();
        cache.retain(|_, agent| agent.last_updated.elapsed() < self.ttl);
        let final_count = cache.len();

        stats.total_entries = final_count as u64;
        stats.evictions += (initial_count - final_count) as u64;
        stats.memory_usage_estimate = self.estimate_memory_usage(&cache);
    }

    /// Get most frequently accessed agents
    pub async fn get_popular_agents(&self, limit: usize) -> Vec<(i32, u64)> {
        let cache = self.cache.read().await;
        let mut agents: Vec<(i32, u64)> = cache
            .iter()
            .map(|(id, agent)| (*id, agent.access_count))
            .collect();

        agents.sort_by(|a, b| b.1.cmp(&a.1));
        agents.truncate(limit);
        agents
    }

    /// Preload agents based on predicted usage
    pub async fn preload_agents(&self, agent_ids: Vec<i32>) -> usize {
        // In a real implementation, this would fetch from database
        // For now, we'll just return how many would be preloaded
        let cache = self.cache.read().await;
        agent_ids.iter().filter(|id| !cache.contains_key(id)).count()
    }

    /// Evict least recently used entry
    async fn evict_lru(&self, cache: &mut HashMap<i32, CachedAgent>, stats: &mut CacheStats) {
        if let Some((agent_id, _)) = cache
            .iter()
            .min_by_key(|(_, agent)| agent.last_accessed)
            .map(|(id, agent)| (*id, agent.clone()))
        {
            cache.remove(&agent_id);
            stats.evictions += 1;
        }
    }

    /// Estimate memory usage of cache
    fn estimate_memory_usage(&self, cache: &HashMap<i32, CachedAgent>) -> u64 {
        // Rough estimation - in practice would be more sophisticated
        const ESTIMATED_ENTRY_SIZE: u64 = 1024; // 1KB per entry average
        cache.len() as u64 * ESTIMATED_ENTRY_SIZE
    }
}

impl Default for AgentCacheService {
    fn default() -> Self {
        Self::new(1000, 3600) // 1000 entries, 1 hour TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_cache_basic_operations() {
        let cache = AgentCacheService::new(10, 60); // Small cache for testing

        let agent = CachedAgent {
            agent_id: 1,
            name: "Test Agent".to_string(),
            capabilities: AgentCapabilities {
                tools: vec!["python".to_string()],
                models: vec!["gpt-4".to_string()],
                max_steps: 10,
                can_create_ephemeral: true,
            },
            policy: AgentPolicy {
                max_workflows_per_hour: 100,
                allowed_patterns: vec!["data_analysis".to_string()],
                security_constraints: HashMap::new(),
            },
            last_updated: Instant::now(),
            access_count: 0,
            last_accessed: Instant::now(),
        };

        // Store agent
        cache.store_agent(agent.clone()).await;

        // Retrieve agent
        let retrieved = cache.get_agent(1).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Agent");

        // Check stats
        let stats = cache.get_stats().await;
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = AgentCacheService::new(2, 60); // Very small cache

        // Add more agents than cache capacity
        for i in 1..=3 {
            let agent = CachedAgent {
                agent_id: i,
                name: format!("Agent {}", i),
                capabilities: AgentCapabilities {
                    tools: vec![],
                    models: vec![],
                    max_steps: 5,
                    can_create_ephemeral: false,
                },
                policy: AgentPolicy {
                    max_workflows_per_hour: 10,
                    allowed_patterns: vec![],
                    security_constraints: HashMap::new(),
                },
                last_updated: Instant::now(),
                access_count: 0,
                last_accessed: Instant::now(),
            };
            cache.store_agent(agent).await;
        }

        let stats = cache.get_stats().await;
        assert_eq!(stats.total_entries, 2); // Should be capped at max_entries
        assert!(stats.evictions > 0); // Should have evicted at least one
    }
}
