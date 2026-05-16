// Conflict resolution policies and resolution engine

use crate::causal_op::CausalOp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conflict resolution policy.
/// 
/// Determines how to resolve conflicts when operations have
/// concurrent version vectors (neither happens-before the other).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// Use wall clock timestamp (may lose data)
    LastWriterWins,
    
    /// Use node priority (configured per-node)
    HighestPriority,
    
    /// Prompt user for resolution (returns both options)
    ManualResolution,
    
    /// Merge semantically if possible (CRDT-aware)
    SemanticMerge,
}

/// Result of conflict resolution
#[derive(Clone, Debug)]
pub enum ConflictResolution {
    /// No conflict detected
    NoConflict,
    
    /// Automatically resolved to winner
    Winner(String),
    
    /// Manual resolution required
    Manual {
        option_a: CausalOp,
        option_b: CausalOp,
    },
    
    /// Semantic merge possible (for CRDTs)
    Merged {
        result: serde_json::Value,
    },
}

/// Conflict resolver with configurable policies.
pub struct ConflictResolver {
    /// Default policy for all operations
    default_policy: ConflictPolicy,
    
    /// Per-operation-type policies (override default)
    operation_policies: HashMap<String, ConflictPolicy>,
    
    /// Node priorities (higher = wins conflicts)
    node_priorities: HashMap<String, u8>,
}

impl ConflictResolver {
    /// Create a new resolver with default policy
    pub fn new(default_policy: ConflictPolicy) -> Self {
        Self {
            default_policy,
            operation_policies: HashMap::new(),
            node_priorities: HashMap::new(),
        }
    }

    /// Set priority for a node
    pub fn set_node_priority(&mut self, node_id: String, priority: u8) {
        self.node_priorities.insert(node_id, priority);
    }

    /// Set policy for specific operation type
    pub fn set_operation_policy(&mut self, operation: String, policy: ConflictPolicy) {
        self.operation_policies.insert(operation, policy);
    }

    /// Get policy for an operation type
    fn get_policy(&self, operation: &str) -> &ConflictPolicy {
        self.operation_policies
            .get(operation)
            .unwrap_or(&self.default_policy)
    }

    /// Resolve conflict between two operations
    pub fn resolve(&self, op1: &CausalOp, op2: &CausalOp) -> ConflictResolution {
        // Check if they actually conflict
        if !op1.conflicts_with(op2) {
            return ConflictResolution::NoConflict;
        }

        let policy = self.get_policy(&op1.operation);

        match policy {
            ConflictPolicy::LastWriterWins => {
                self.resolve_lww(op1, op2)
            }
            ConflictPolicy::HighestPriority => {
                self.resolve_priority(op1, op2)
            }
            ConflictPolicy::ManualResolution => {
                ConflictResolution::Manual {
                    option_a: op1.clone(),
                    option_b: op2.clone(),
                }
            }
            ConflictPolicy::SemanticMerge => {
                self.resolve_semantic(op1, op2)
            }
        }
    }

    /// Last-writer-wins resolution
    fn resolve_lww(&self, op1: &CausalOp, op2: &CausalOp) -> ConflictResolution {
        if op1.wall_clock > op2.wall_clock {
            ConflictResolution::Winner(op1.id.clone())
        } else if op2.wall_clock > op1.wall_clock {
            ConflictResolution::Winner(op2.id.clone())
        } else {
            // Tie-breaker: compare operation IDs (deterministic)
            if op1.id > op2.id {
                ConflictResolution::Winner(op1.id.clone())
            } else {
                ConflictResolution::Winner(op2.id.clone())
            }
        }
    }

    /// Priority-based resolution
    fn resolve_priority(&self, op1: &CausalOp, op2: &CausalOp) -> ConflictResolution {
        let p1 = self.node_priorities.get(&op1.node_id).copied().unwrap_or(0);
        let p2 = self.node_priorities.get(&op2.node_id).copied().unwrap_or(0);

        if p1 > p2 {
            ConflictResolution::Winner(op1.id.clone())
        } else if p2 > p1 {
            ConflictResolution::Winner(op2.id.clone())
        } else {
            // Same priority: fall back to LWW
            self.resolve_lww(op1, op2)
        }
    }

    /// Semantic merge resolution (CRDT-aware)
    fn resolve_semantic(&self, op1: &CausalOp, op2: &CausalOp) -> ConflictResolution {
        // Check if operations can be merged semantically
        if op1.operation == op2.operation {
            match op1.operation.as_str() {
                "increment" | "decrement" => {
                    // Counter operations: add values
                    let v1 = op1.input.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                    let v2 = op2.input.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                    ConflictResolution::Merged {
                        result: serde_json::json!({ "value": v1 + v2 }),
                    }
                }
                "add_to_set" => {
                    // Set add: union of elements
                    let set1: Vec<serde_json::Value> = op1.input
                        .get("elements")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let set2: Vec<serde_json::Value> = op2.input
                        .get("elements")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    
                    let mut merged: Vec<serde_json::Value> = set1;
                    for elem in set2 {
                        if !merged.contains(&elem) {
                            merged.push(elem);
                        }
                    }
                    
                    ConflictResolution::Merged {
                        result: serde_json::json!({ "elements": merged }),
                    }
                }
                _ => {
                    // Unknown operation type: fall back to manual
                    ConflictResolution::Manual {
                        option_a: op1.clone(),
                        option_b: op2.clone(),
                    }
                }
            }
        } else {
            // Different operation types: manual resolution
            ConflictResolution::Manual {
                option_a: op1.clone(),
                option_b: op2.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VersionVector;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use std::collections::BTreeSet;

    fn create_test_keypair() -> SigningKey {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    fn create_conflicting_ops(keypair: &SigningKey) -> (CausalOp, CausalOp) {
        let mut vv1 = VersionVector::new();
        vv1.increment("node1");

        let mut vv2 = VersionVector::new();
        vv2.increment("node2");

        let op1 = CausalOp::new(
            "edit".to_string(),
            serde_json::json!({"value": "A"}),
            BTreeSet::new(),
            vv1,
            "node1".to_string(),
            keypair,
        );

        // Add small delay to ensure different wall clocks
        std::thread::sleep(std::time::Duration::from_millis(100));

        let op2 = CausalOp::new(
            "edit".to_string(),
            serde_json::json!({"value": "B"}),
            BTreeSet::new(),
            vv2,
            "node2".to_string(),
            keypair,
        );

        (op1, op2)
    }

    #[test]
    fn test_no_conflict() {
        let keypair = create_test_keypair();
        let resolver = ConflictResolver::new(ConflictPolicy::LastWriterWins);

        let mut vv1 = VersionVector::new();
        vv1.increment("node1");

        let op1 = CausalOp::new(
            "op1".to_string(),
            serde_json::json!({}),
            BTreeSet::new(),
            vv1.clone(),
            "node1".to_string(),
            &keypair,
        );

        let mut vv2 = vv1.clone();
        vv2.increment("node1");

        let mut deps = BTreeSet::new();
        deps.insert(op1.id.clone());

        let op2 = CausalOp::new(
            "op2".to_string(),
            serde_json::json!({}),
            deps,
            vv2,
            "node1".to_string(),
            &keypair,
        );

        match resolver.resolve(&op1, &op2) {
            ConflictResolution::NoConflict => {}
            _ => panic!("Expected no conflict"),
        }
    }

    #[test]
    fn test_lww_resolution() {
        let keypair = create_test_keypair();
        let resolver = ConflictResolver::new(ConflictPolicy::LastWriterWins);
        let (op1, op2) = create_conflicting_ops(&keypair);

        match resolver.resolve(&op1, &op2) {
            ConflictResolution::Winner(id) => {
                // op2 was created later, should win
                assert_eq!(id, op2.id);
            }
            _ => panic!("Expected winner resolution"),
        }
    }

    #[test]
    fn test_priority_resolution() {
        let keypair = create_test_keypair();
        let mut resolver = ConflictResolver::new(ConflictPolicy::HighestPriority);
        resolver.set_node_priority("node1".to_string(), 10);
        resolver.set_node_priority("node2".to_string(), 5);

        let (op1, op2) = create_conflicting_ops(&keypair);

        match resolver.resolve(&op1, &op2) {
            ConflictResolution::Winner(id) => {
                // node1 has higher priority
                assert_eq!(id, op1.id);
            }
            _ => panic!("Expected winner resolution"),
        }
    }

    #[test]
    fn test_manual_resolution() {
        let keypair = create_test_keypair();
        let resolver = ConflictResolver::new(ConflictPolicy::ManualResolution);
        let (op1, op2) = create_conflicting_ops(&keypair);

        match resolver.resolve(&op1, &op2) {
            ConflictResolution::Manual { option_a, option_b } => {
                assert_eq!(option_a.id, op1.id);
                assert_eq!(option_b.id, op2.id);
            }
            _ => panic!("Expected manual resolution"),
        }
    }

    #[test]
    fn test_semantic_merge_counter() {
        let keypair = create_test_keypair();
        let resolver = ConflictResolver::new(ConflictPolicy::SemanticMerge);

        let mut vv1 = VersionVector::new();
        vv1.increment("node1");

        let mut vv2 = VersionVector::new();
        vv2.increment("node2");

        let op1 = CausalOp::new(
            "increment".to_string(),
            serde_json::json!({"value": 5}),
            BTreeSet::new(),
            vv1,
            "node1".to_string(),
            &keypair,
        );

        let op2 = CausalOp::new(
            "increment".to_string(),
            serde_json::json!({"value": 3}),
            BTreeSet::new(),
            vv2,
            "node2".to_string(),
            &keypair,
        );

        match resolver.resolve(&op1, &op2) {
            ConflictResolution::Merged { result } => {
                assert_eq!(result.get("value").unwrap().as_i64().unwrap(), 8);
            }
            _ => panic!("Expected merged resolution"),
        }
    }
}
