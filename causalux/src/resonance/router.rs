//! Resonant Router - Smart sync routing based on affinity
//! 
//! Routes sync requests to nodes with highest affinity,
//! reducing unnecessary sync operations and bandwidth.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use std::time::Duration;
use crate::version_vector::VersionVector;
use super::affinity::{AffinityTracker, DataPattern};

/// Routing decision for a sync operation
#[derive(Debug, Clone)]
pub enum RoutingDecision {
    /// Sync with specific nodes (ordered by priority)
    SyncWith(Vec<SyncRoute>),
    /// Broadcast to all known nodes
    Broadcast,
    /// No sync needed (up to date)
    SkipSync,
    /// Defer sync (wait for better conditions)
    Defer { reason: String, retry_after: Duration },
}

/// A sync route to a specific node
#[derive(Debug, Clone)]
pub struct SyncRoute {
    /// Target node ID
    pub node_id: String,
    /// Priority (higher = sync first)
    pub priority: f64,
    /// Estimated sync cost (bandwidth)
    pub estimated_cost: u64,
    /// Estimated benefit (operations to sync)
    pub estimated_benefit: u64,
    /// Route type
    pub route_type: RouteType,
}

/// Type of sync route
#[derive(Debug, Clone, PartialEq)]
pub enum RouteType {
    /// Direct P2P sync
    Direct,
    /// Via relay node
    Relay { via: String },
    /// Batch with other syncs
    Batched,
}

/// Resonant Router - makes smart sync decisions
#[derive(Debug, Clone)]
pub struct ResonantRouter {
    /// Affinity tracker
    affinity: AffinityTracker,
    /// Known nodes and their status
    nodes: HashMap<String, NodeStatus>,
    /// Sync history (for learning)
    sync_history: Vec<SyncRecord>,
    /// Maximum sync history
    max_history: usize,
    /// Minimum affinity to consider
    min_affinity: f64,
    /// Maximum concurrent syncs
    max_concurrent: usize,
    /// Current sync operations
    active_syncs: BTreeSet<String>,
}

/// Status of a known node
#[derive(Debug, Clone)]
pub struct NodeStatus {
    pub node_id: String,
    pub last_seen: u64,
    pub version: VersionVector,
    pub reachable: bool,
    pub latency_ms: u32,
    pub bandwidth_kbps: u32,
}

/// Record of a sync operation
#[derive(Debug, Clone)]
pub struct SyncRecord {
    pub node_id: String,
    pub timestamp: u64,
    pub success: bool,
    pub ops_synced: usize,
    pub duration_ms: u64,
    pub bytes_transferred: u64,
}

impl ResonantRouter {
    /// Create a new resonant router
    pub fn new(node_id: String) -> Self {
        Self {
            affinity: AffinityTracker::new(node_id),
            nodes: HashMap::new(),
            sync_history: Vec::new(),
            max_history: 1000,
            min_affinity: 0.3,
            max_concurrent: 5,
            active_syncs: BTreeSet::new(),
        }
    }

    /// Record data access (updates affinity patterns)
    pub fn record_access(&mut self, key: &str) {
        self.affinity.record_access(key);
    }

    /// Update node status
    pub fn update_node(&mut self, status: NodeStatus) {
        self.nodes.insert(status.node_id.clone(), status);
    }

    /// Update affinity with a node based on their pattern
    pub fn update_affinity(&mut self, node_id: &str, pattern: DataPattern) {
        self.affinity.update_remote_pattern(node_id, pattern);
    }

    /// Decide routing for a sync operation
    pub fn route(&self, required_keys: Option<&[String]>) -> RoutingDecision {
        // Get high-affinity nodes
        let high_affinity = self.affinity.high_affinity_nodes();
        
        if high_affinity.is_empty() {
            // No known nodes with good affinity
            return RoutingDecision::Broadcast;
        }
        
        // Filter by availability and capacity
        let mut routes: Vec<SyncRoute> = high_affinity
            .iter()
            .filter_map(|(node_id, affinity)| {
                let status = self.nodes.get(*node_id)?;
                if !status.reachable {
                    return None;
                }
                
                // Check if we can add more syncs
                if self.active_syncs.len() >= self.max_concurrent 
                   && !self.active_syncs.contains(*node_id) {
                    return None;
                }
                
                Some(SyncRoute {
                    node_id: (*node_id).clone(),
                    priority: *affinity,
                    estimated_cost: self.estimate_cost(status),
                    estimated_benefit: self.estimate_benefit(node_id, required_keys),
                    route_type: RouteType::Direct,
                })
            })
            .collect();
        
        // Sort by priority (considering cost/benefit)
        routes.sort_by(|a, b| {
            let score_a = a.priority * (a.estimated_benefit as f64) / (a.estimated_cost as f64 + 1.0);
            let score_b = b.priority * (b.estimated_benefit as f64) / (b.estimated_cost as f64 + 1.0);
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        if routes.is_empty() {
            RoutingDecision::Defer {
                reason: "No available nodes".to_string(),
                retry_after: Duration::from_secs(5),
            }
        } else {
            RoutingDecision::SyncWith(routes)
        }
    }

    /// Route for specific data (e.g., a document)
    pub fn route_for_data(&self, keys: &[String]) -> RoutingDecision {
        self.route(Some(keys))
    }

    /// Route for general sync (catch up)
    pub fn route_general(&self) -> RoutingDecision {
        self.route(None)
    }

    /// Record a sync operation result
    pub fn record_sync(&mut self, record: SyncRecord) {
        // Update affinity based on result
        self.affinity.record_sync(&record.node_id, record.success, record.ops_synced);
        
        // Store in history
        if self.sync_history.len() >= self.max_history {
            self.sync_history.remove(0);
        }
        self.sync_history.push(record.clone());
        
        // Update active syncs
        self.active_syncs.remove(&record.node_id);
    }

    /// Start a sync operation
    pub fn start_sync(&mut self, node_id: &str) -> bool {
        if self.active_syncs.len() >= self.max_concurrent {
            return false;
        }
        self.active_syncs.insert(node_id.to_string());
        true
    }

    /// Get sync statistics
    pub fn stats(&self) -> RouterStats {
        let total_syncs = self.sync_history.len();
        let successful = self.sync_history.iter().filter(|r| r.success).count();
        let total_ops: usize = self.sync_history.iter().map(|r| r.ops_synced).sum();
        let total_bytes: u64 = self.sync_history.iter().map(|r| r.bytes_transferred).sum();
        
        RouterStats {
            total_syncs,
            successful_syncs: successful,
            success_rate: if total_syncs > 0 { successful as f64 / total_syncs as f64 } else { 0.0 },
            total_ops_synced: total_ops,
            total_bytes_transferred: total_bytes,
            known_nodes: self.nodes.len(),
            high_affinity_nodes: self.affinity.high_affinity_nodes().len(),
            active_syncs: self.active_syncs.len(),
        }
    }

    fn estimate_cost(&self, status: &NodeStatus) -> u64 {
        // Cost based on latency and bandwidth
        let latency_factor = status.latency_ms as u64;
        let bandwidth_factor = 1000 / (status.bandwidth_kbps as u64 + 1);
        latency_factor + bandwidth_factor * 10
    }

    fn estimate_benefit(&self, node_id: &str, required_keys: Option<&[String]>) -> u64 {
        // Benefit based on version difference and affinity
        let affinity = self.affinity.affinity(node_id);
        let version_diff = self.nodes.get(node_id)
            .map(|s| s.version.total_operations())
            .unwrap_or(0) as u64;
        
        // Higher affinity = likely more useful data
        ((affinity * 100.0) as u64) + version_diff
    }

    /// Get affinity tracker (for pattern sharing)
    pub fn affinity_tracker(&self) -> &AffinityTracker {
        &self.affinity
    }

    /// Get local pattern for sharing with other nodes
    pub fn local_pattern(&self) -> &DataPattern {
        self.affinity.local_pattern()
    }
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    pub total_syncs: usize,
    pub successful_syncs: usize,
    pub success_rate: f64,
    pub total_ops_synced: usize,
    pub total_bytes_transferred: u64,
    pub known_nodes: usize,
    pub high_affinity_nodes: usize,
    pub active_syncs: usize,
}

/// Resonance Score - overall sync health
impl ResonantRouter {
    /// Compute resonance score (0.0 - 1.0)
    /// High score = network is well-synchronized
    pub fn resonance_score(&self) -> f64 {
        let stats = self.stats();
        
        if stats.total_syncs == 0 {
            return 0.5; // Unknown
        }
        
        // Factors: success rate, coverage, efficiency
        let success_factor = stats.success_rate;
        let coverage_factor = if stats.known_nodes > 0 {
            stats.high_affinity_nodes as f64 / stats.known_nodes as f64
        } else {
            0.0
        };
        let efficiency_factor = if stats.total_bytes_transferred > 0 {
            (stats.total_ops_synced as f64 / stats.total_bytes_transferred as f64 * 1000.0).min(1.0)
        } else {
            0.0
        };
        
        // Weighted average
        0.5 * success_factor + 0.3 * coverage_factor + 0.2 * efficiency_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = ResonantRouter::new("node1".to_string());
        assert!(router.stats().known_nodes == 0);
    }

    #[test]
    fn test_routing_decision() {
        let mut router = ResonantRouter::new("node1".to_string());
        
        // Add a known node
        router.update_node(NodeStatus {
            node_id: "node2".to_string(),
            last_seen: 0,
            version: VersionVector::new(),
            reachable: true,
            latency_ms: 50,
            bandwidth_kbps: 1000,
        });
        
        // Add affinity
        let mut pattern = DataPattern::new(100);
        pattern.record_access("doc1");
        router.update_affinity("node2", pattern);
        
        // Record some local access
        router.record_access("doc1");
        
        let decision = router.route_general();
        
        match decision {
            RoutingDecision::SyncWith(routes) => {
                assert!(!routes.is_empty());
                assert_eq!(routes[0].node_id, "node2");
            }
            _ => panic!("Expected SyncWith decision"),
        }
    }

    #[test]
    fn test_sync_recording() {
        let mut router = ResonantRouter::new("node1".to_string());
        
        router.record_sync(SyncRecord {
            node_id: "node2".to_string(),
            timestamp: 0,
            success: true,
            ops_synced: 100,
            duration_ms: 50,
            bytes_transferred: 10000,
        });
        
        let stats = router.stats();
        assert_eq!(stats.total_syncs, 1);
        assert_eq!(stats.successful_syncs, 1);
        assert_eq!(stats.total_ops_synced, 100);
    }

    #[test]
    fn test_resonance_score() {
        let mut router = ResonantRouter::new("node1".to_string());
        
        // Record successful syncs
        for _ in 0..10 {
            router.record_sync(SyncRecord {
                node_id: "node2".to_string(),
                timestamp: 0,
                success: true,
                ops_synced: 50,
                duration_ms: 100,
                bytes_transferred: 5000,
            });
        }
        
        let score = router.resonance_score();
        assert!(score > 0.4); // Should have decent score
    }
}
