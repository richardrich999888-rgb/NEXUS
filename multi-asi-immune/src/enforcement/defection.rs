//! Defection detection and handling.

use crate::identity::keypair::AsiId;
use crate::protocol::message::AccusationEvidence;
use serde::{Serialize, Deserialize};

/// Types of detectable defection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefectionType {
    /// Node stopped responding (missed heartbeats).
    Unresponsive,
    /// Node sent contradictory messages.
    Contradictory,
    /// Node violated a mutual constraint.
    ConstraintViolation,
    /// Node sent messages with invalid signatures.
    InvalidSignatures,
    /// Node provided false threat reports.
    FalseThreatReports,
    /// Node attempted to forge identity.
    IdentityForgery,
}

impl DefectionType {
    /// Returns severity of this defection type [0, 1].
    pub fn severity(&self) -> f64 {
        match self {
            DefectionType::Unresponsive => 0.3,
            DefectionType::Contradictory => 0.7,
            DefectionType::ConstraintViolation => 0.6,
            DefectionType::InvalidSignatures => 0.9,
            DefectionType::FalseThreatReports => 0.5,
            DefectionType::IdentityForgery => 1.0,
        }
    }
}

/// Record of a detected defection.
#[derive(Debug, Clone)]
pub struct DefectionRecord {
    /// The defecting node.
    pub node: AsiId,
    /// Type of defection.
    pub defection_type: DefectionType,
    /// Evidence of defection.
    pub evidence: AccusationEvidence,
    /// When detected.
    pub detected_at: u64,
    /// Who detected it.
    pub detected_by: AsiId,
}

/// Tracks defection history for nodes.
#[derive(Debug, Default)]
pub struct DefectionTracker {
    /// Defection records by accused node.
    records: std::collections::HashMap<AsiId, Vec<DefectionRecord>>,
    /// Threshold for isolation.
    isolation_threshold: f64,
}

impl DefectionTracker {
    /// Creates a new tracker with default isolation threshold.
    pub fn new() -> Self {
        Self {
            records: std::collections::HashMap::new(),
            isolation_threshold: 1.5, // Cumulative severity
        }
    }
    
    /// Creates a tracker with custom isolation threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            records: std::collections::HashMap::new(),
            isolation_threshold: threshold,
        }
    }
    
    /// Records a defection.
    pub fn record(&mut self, record: DefectionRecord) {
        self.records
            .entry(record.node)
            .or_default()
            .push(record);
    }
    
    /// Returns cumulative severity for a node.
    pub fn cumulative_severity(&self, node: AsiId) -> f64 {
        self.records
            .get(&node)
            .map(|records| {
                records.iter().map(|r| r.defection_type.severity()).sum()
            })
            .unwrap_or(0.0)
    }
    
    /// Returns true if node should be isolated.
    pub fn should_isolate(&self, node: AsiId) -> bool {
        self.cumulative_severity(node) >= self.isolation_threshold
    }
    
    /// Returns all defection records for a node.
    pub fn get_records(&self, node: AsiId) -> &[DefectionRecord] {
        self.records.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    /// Returns count of defections for a node.
    pub fn defection_count(&self, node: AsiId) -> usize {
        self.records.get(&node).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_id(n: u8) -> AsiId {
        AsiId([n; 32])
    }
    
    #[test]
    fn test_isolation_threshold() {
        let mut tracker = DefectionTracker::with_threshold(1.0);
        
        let bad_node = make_id(1);
        let detector = make_id(2);
        
        // Record minor defection
        tracker.record(DefectionRecord {
            node: bad_node,
            defection_type: DefectionType::Unresponsive,
            evidence: AccusationEvidence::MissedHeartbeats {
                expected_count: 5,
                received_count: 0,
            },
            detected_at: 100,
            detected_by: detector,
        });
        
        assert!(!tracker.should_isolate(bad_node));
        
        // Record severe defection
        tracker.record(DefectionRecord {
            node: bad_node,
            defection_type: DefectionType::Contradictory,
            evidence: AccusationEvidence::Contradiction {
                message1: vec![1],
                message2: vec![2],
            },
            detected_at: 200,
            detected_by: detector,
        });
        
        // Now should be isolated
        assert!(tracker.should_isolate(bad_node));
    }
}
