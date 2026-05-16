//! Node Affinity Tracking
//! 
//! Tracks data access patterns to compute "affinity" between nodes.
//! High affinity = nodes work on similar data = should sync more.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use crate::version_vector::VersionVector;

/// Data access pattern for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPattern {
    /// Documents/keys accessed recently
    pub accessed_keys: VecDeque<String>,
    /// Frequency of access per key
    pub access_frequency: HashMap<String, u64>,
    /// Time-weighted access score
    pub access_scores: HashMap<String, f64>,
    /// Pattern fingerprint (for quick comparison)
    pub fingerprint: u64,
    /// Maximum history size
    max_history: usize,
}

impl DataPattern {
    /// Create a new data pattern tracker
    pub fn new(max_history: usize) -> Self {
        Self {
            accessed_keys: VecDeque::with_capacity(max_history),
            access_frequency: HashMap::new(),
            access_scores: HashMap::new(),
            fingerprint: 0,
            max_history,
        }
    }

    /// Record an access to a key
    pub fn record_access(&mut self, key: &str) {
        // Add to recent history
        if self.accessed_keys.len() >= self.max_history {
            if let Some(old_key) = self.accessed_keys.pop_front() {
                // Decay old key's score
                if let Some(score) = self.access_scores.get_mut(&old_key) {
                    *score *= 0.9;
                }
            }
        }
        self.accessed_keys.push_back(key.to_string());
        
        // Update frequency
        *self.access_frequency.entry(key.to_string()).or_insert(0) += 1;
        
        // Update score (recent access = higher score)
        *self.access_scores.entry(key.to_string()).or_insert(0.0) += 1.0;
        
        // Decay all scores slightly
        for score in self.access_scores.values_mut() {
            *score *= 0.99;
        }
        
        // Update fingerprint
        self.update_fingerprint();
    }

    /// Update the fingerprint hash
    fn update_fingerprint(&mut self) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        
        // Hash top-10 most accessed keys
        let mut top_keys: Vec<_> = self.access_scores.iter().collect();
        top_keys.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        for (key, _) in top_keys.iter().take(10) {
            key.hash(&mut hasher);
        }
        
        self.fingerprint = hasher.finish();
    }

    /// Get top N accessed keys
    pub fn top_keys(&self, n: usize) -> Vec<(&String, f64)> {
        let mut sorted: Vec<_> = self.access_scores.iter()
            .map(|(k, &s)| (k, s))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted.into_iter().take(n).collect()
    }

    /// Compute similarity with another pattern (0.0 - 1.0)
    pub fn similarity(&self, other: &DataPattern) -> f64 {
        // Quick check: same fingerprint = likely very similar
        if self.fingerprint == other.fingerprint {
            return 1.0;
        }
        
        // Jaccard similarity on accessed keys
        let self_keys: HashSet<_> = self.access_scores.keys().collect();
        let other_keys: HashSet<_> = other.access_scores.keys().collect();
        
        let intersection = self_keys.intersection(&other_keys).count();
        let union = self_keys.union(&other_keys).count();
        
        if union == 0 {
            return 0.0;
        }
        
        let jaccard = intersection as f64 / union as f64;
        
        // Also consider access patterns (cosine similarity on scores)
        let cosine = self.cosine_similarity(other);
        
        // Combined similarity
        0.5 * jaccard + 0.5 * cosine
    }

    /// Cosine similarity of access scores
    fn cosine_similarity(&self, other: &DataPattern) -> f64 {
        let all_keys: HashSet<_> = self.access_scores.keys()
            .chain(other.access_scores.keys())
            .collect();
        
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for key in all_keys {
            let a = self.access_scores.get(key).unwrap_or(&0.0);
            let b = other.access_scores.get(key).unwrap_or(&0.0);
            
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

/// Affinity between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinity {
    /// Source node
    pub from_node: String,
    /// Target node
    pub to_node: String,
    /// Affinity score (0.0 - 1.0)
    pub score: f64,
    /// Last update timestamp
    pub last_updated: u64,
    /// Sync count with this node
    pub sync_count: u64,
    /// Success rate of syncs
    pub success_rate: f64,
}

impl NodeAffinity {
    pub fn new(from_node: String, to_node: String) -> Self {
        Self {
            from_node,
            to_node,
            score: 0.5, // Neutral starting point
            last_updated: Self::now(),
            sync_count: 0,
            success_rate: 1.0,
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Update affinity based on pattern similarity
    pub fn update(&mut self, similarity: f64, sync_success: bool) {
        // Exponential moving average
        self.score = 0.8 * self.score + 0.2 * similarity;
        self.last_updated = Self::now();
        self.sync_count += 1;
        
        // Update success rate
        let success_val = if sync_success { 1.0 } else { 0.0 };
        self.success_rate = 0.9 * self.success_rate + 0.1 * success_val;
    }

    /// Get effective affinity (combines score and success rate)
    pub fn effective_affinity(&self) -> f64 {
        self.score * self.success_rate
    }

    /// Check if affinity is stale
    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        Self::now() - self.last_updated > max_age_secs
    }
}

/// Affinity tracker for all nodes
#[derive(Debug, Clone)]
pub struct AffinityTracker {
    /// This node's ID
    node_id: String,
    /// This node's data pattern
    local_pattern: DataPattern,
    /// Known patterns from other nodes
    remote_patterns: HashMap<String, DataPattern>,
    /// Affinity scores to other nodes
    affinities: HashMap<String, NodeAffinity>,
    /// Version vector
    version: VersionVector,
    /// Minimum affinity to consider for routing
    min_affinity_threshold: f64,
}

impl AffinityTracker {
    /// Create a new affinity tracker
    pub fn new(node_id: String) -> Self {
        Self {
            node_id: node_id.clone(),
            local_pattern: DataPattern::new(1000),
            remote_patterns: HashMap::new(),
            affinities: HashMap::new(),
            version: VersionVector::new(),
            min_affinity_threshold: 0.3,
        }
    }

    /// Record local data access
    pub fn record_access(&mut self, key: &str) {
        self.local_pattern.record_access(key);
        self.version.increment(&self.node_id);
    }

    /// Update pattern from a remote node
    pub fn update_remote_pattern(&mut self, node_id: &str, pattern: DataPattern) {
        // Compute similarity
        let similarity = self.local_pattern.similarity(&pattern);
        
        // Update or create affinity
        let affinity = self.affinities
            .entry(node_id.to_string())
            .or_insert_with(|| NodeAffinity::new(self.node_id.clone(), node_id.to_string()));
        
        affinity.update(similarity, true);
        
        self.remote_patterns.insert(node_id.to_string(), pattern);
    }

    /// Record sync result with a node
    pub fn record_sync(&mut self, node_id: &str, success: bool, ops_synced: usize) {
        if let Some(affinity) = self.affinities.get_mut(node_id) {
            // Boost affinity if we synced useful data
            let usefulness = (ops_synced as f64 / 100.0).min(1.0);
            affinity.update(affinity.score + usefulness * 0.1, success);
        }
    }

    /// Get affinity with a node
    pub fn affinity(&self, node_id: &str) -> f64 {
        self.affinities
            .get(node_id)
            .map(|a| a.effective_affinity())
            .unwrap_or(0.5) // Default neutral affinity
    }

    /// Get nodes with high affinity (should sync with these)
    pub fn high_affinity_nodes(&self) -> Vec<(&String, f64)> {
        let mut result: Vec<_> = self.affinities
            .iter()
            .filter(|(_, a)| a.effective_affinity() >= self.min_affinity_threshold)
            .map(|(id, a)| (id, a.effective_affinity()))
            .collect();
        
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result
    }

    /// Get nodes ranked by sync priority
    pub fn sync_priority(&self) -> Vec<(String, f64)> {
        let mut priorities: Vec<_> = self.affinities
            .iter()
            .map(|(id, a)| {
                // Priority = affinity * recency_factor
                let staleness = NodeAffinity::now() - a.last_updated;
                let recency_factor = 1.0 / (1.0 + staleness as f64 / 3600.0); // Decay over hours
                (id.clone(), a.effective_affinity() * recency_factor)
            })
            .collect();
        
        priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        priorities
    }

    /// Get local pattern for sharing
    pub fn local_pattern(&self) -> &DataPattern {
        &self.local_pattern
    }

    /// Merge with another tracker (for distributed consensus)
    pub fn merge(&mut self, other: &AffinityTracker) {
        for (node_id, pattern) in &other.remote_patterns {
            if !self.remote_patterns.contains_key(node_id) {
                self.remote_patterns.insert(node_id.clone(), pattern.clone());
            }
        }
        
        for (node_id, other_affinity) in &other.affinities {
            if let Some(self_affinity) = self.affinities.get_mut(node_id) {
                // Average the scores
                self_affinity.score = (self_affinity.score + other_affinity.score) / 2.0;
            } else {
                self.affinities.insert(node_id.clone(), other_affinity.clone());
            }
        }
        
        self.version = self.version.merge(&other.version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_pattern() {
        let mut pattern = DataPattern::new(100);
        
        pattern.record_access("doc1");
        pattern.record_access("doc1");
        pattern.record_access("doc2");
        
        let top = pattern.top_keys(2);
        assert_eq!(top[0].0, "doc1"); // doc1 accessed more
    }

    #[test]
    fn test_pattern_similarity() {
        let mut p1 = DataPattern::new(100);
        let mut p2 = DataPattern::new(100);
        
        // Same access patterns
        p1.record_access("doc1");
        p1.record_access("doc2");
        p2.record_access("doc1");
        p2.record_access("doc2");
        
        let sim = p1.similarity(&p2);
        assert!(sim > 0.8); // Should be very similar
        
        // Different patterns
        let mut p3 = DataPattern::new(100);
        p3.record_access("doc99");
        p3.record_access("doc100");
        
        let sim2 = p1.similarity(&p3);
        assert!(sim2 < 0.3); // Should be very different
    }

    #[test]
    fn test_affinity_tracker() {
        let mut tracker = AffinityTracker::new("node1".to_string());
        
        // Record local accesses
        tracker.record_access("doc1");
        tracker.record_access("doc1");
        tracker.record_access("doc2");
        
        // Create similar remote pattern
        let mut remote_pattern = DataPattern::new(100);
        remote_pattern.record_access("doc1");
        remote_pattern.record_access("doc2");
        
        tracker.update_remote_pattern("node2", remote_pattern);
        
        // Should have high affinity
        let affinity = tracker.affinity("node2");
        assert!(affinity > 0.5);
    }

    #[test]
    fn test_high_affinity_nodes() {
        let mut tracker = AffinityTracker::new("node1".to_string());
        
        // Add some nodes with varying patterns
        for i in 0..5 {
            let mut pattern = DataPattern::new(100);
            pattern.record_access(&format!("doc{}", i));
            tracker.update_remote_pattern(&format!("node{}", i), pattern);
        }
        
        let high = tracker.high_affinity_nodes();
        assert!(!high.is_empty());
    }
}
