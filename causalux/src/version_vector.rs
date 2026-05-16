// Version Vector implementation for causal tracking

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Version Vector for tracking causality across distributed nodes.
/// 
/// A version vector is a map from node IDs to logical clocks, enabling
/// detection of causal relationships and conflicts between operations.
/// 
/// # Examples
/// 
/// ```
/// use causalux_v2::VersionVector;
/// 
/// let mut v1 = VersionVector::new();
/// v1.increment("node_a");
/// v1.increment("node_a");
/// 
/// let mut v2 = VersionVector::new();
/// v2.increment("node_a");
/// v2.increment("node_a");
/// v2.increment("node_b");
/// 
/// assert!(v1.happens_before(&v2));
/// assert!(!v1.conflicts_with(&v2));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VersionVector {
    /// Map of node_id -> operation_count
    pub versions: BTreeMap<String, u64>,
}

impl VersionVector {
    /// Create a new empty version vector
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the count for a given node and return the new count
    pub fn increment(&mut self, node_id: &str) -> u64 {
        let counter = self.versions.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Check if self happened-before other (self ≤ other)
    /// 
    /// Returns true if all entries in self are less than or equal to
    /// the corresponding entries in other.
    pub fn happens_before(&self, other: &VersionVector) -> bool {
        self.versions.iter().all(|(node, count)| {
            other.versions.get(node).map_or(false, |other_count| count <= other_count)
        })
    }

    /// Check if vectors conflict (neither dominates the other)
    /// 
    /// Returns true when concurrent operations from different nodes
    /// exist, requiring conflict resolution.
    pub fn conflicts_with(&self, other: &VersionVector) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }

    /// Merge two version vectors (take max per node)
    /// 
    /// Creates a new version vector that represents the union of
    /// causality from both input vectors.
    pub fn merge(&self, other: &VersionVector) -> VersionVector {
        let mut merged = self.versions.clone();
        for (node, count) in &other.versions {
            merged
                .entry(node.clone())
                .and_modify(|c| *c = (*c).max(*count))
                .or_insert(*count);
        }
        VersionVector { versions: merged }
    }

    /// Get the count for a specific node (0 if not present)
    pub fn get(&self, node_id: &str) -> u64 {
        self.versions.get(node_id).copied().unwrap_or(0)
    }

    /// Get the total number of operations across all nodes
    pub fn total_operations(&self) -> u64 {
        self.versions.values().sum()
    }

    /// Check if this vector is empty
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut vv = VersionVector::new();
        assert_eq!(vv.increment("node1"), 1);
        assert_eq!(vv.increment("node1"), 2);
        assert_eq!(vv.increment("node2"), 1);
        assert_eq!(vv.get("node1"), 2);
        assert_eq!(vv.get("node2"), 1);
    }

    #[test]
    fn test_happens_before() {
        let mut v1 = VersionVector::new();
        v1.increment("A");
        v1.increment("B");

        let mut v2 = VersionVector::new();
        v2.increment("A");
        v2.increment("A");
        v2.increment("B");

        assert!(v1.happens_before(&v2));
        assert!(!v2.happens_before(&v1));
    }

    #[test]
    fn test_conflict_detection() {
        let mut v1 = VersionVector::new();
        v1.increment("A");
        v1.increment("A");
        v1.increment("B");

        let mut v2 = VersionVector::new();
        v2.increment("A");
        v2.increment("B");
        v2.increment("B");

        assert!(v1.conflicts_with(&v2));
        assert!(v2.conflicts_with(&v1));
    }

    #[test]
    fn test_merge() {
        let mut v1 = VersionVector::new();
        v1.increment("A");
        v1.increment("B");

        let mut v2 = VersionVector::new();
        v2.increment("A");
        v2.increment("A");
        v2.increment("C");

        let merged = v1.merge(&v2);
        assert_eq!(merged.get("A"), 2);
        assert_eq!(merged.get("B"), 1);
        assert_eq!(merged.get("C"), 1);
    }

    #[test]
    fn test_total_operations() {
        let mut vv = VersionVector::new();
        vv.increment("A");
        vv.increment("A");
        vv.increment("B");
        
        assert_eq!(vv.total_operations(), 3);
    }
}
