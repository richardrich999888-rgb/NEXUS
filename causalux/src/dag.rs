// Causal DAG v2.0 with version vectors and GC

use crate::causal_op::CausalOp;
use crate::conflict::{ConflictPolicy, ConflictResolver, ConflictResolution};
use crate::snapshot::{Snapshot, SnapshotManager};
use crate::version_vector::VersionVector;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Error types for DAG operations
#[derive(Debug, Clone)]
pub enum DagError {
    MissingDependency(String),
    ConflictRejected { winner_id: String },
    ManualResolutionRequired { op_a: String, op_b: String },
    InvalidSignature,
    CausalityViolation,
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::MissingDependency(id) => write!(f, "Missing dependency: {}", id),
            DagError::ConflictRejected { winner_id } => {
                write!(f, "Conflict resolved: {} won", winner_id)
            }
            DagError::ManualResolutionRequired { op_a, op_b } => {
                write!(f, "Manual resolution needed: {} vs {}", op_a, op_b)
            }
            DagError::InvalidSignature => write!(f, "Invalid operation signature"),
            DagError::CausalityViolation => write!(f, "Causality violation detected"),
        }
    }
}

impl std::error::Error for DagError {}

/// Causal DAG v2.0 with integrated snapshot management and conflict resolution.
pub struct CausalDAG {
    /// Operations by ID
    operations: BTreeMap<String, CausalOp>,
    
    /// Reverse dependency index (op_id -> dependent op_ids)
    dependents: BTreeMap<String, BTreeSet<String>>,
    
    /// Root operations (no dependencies)
    roots: BTreeSet<String>,
    
    /// Snapshot manager for GC
    snapshot_manager: SnapshotManager,
    
    /// Conflict resolver
    conflict_resolver: ConflictResolver,
    
    /// This node's ID
    node_id: String,
    
    /// Current version vector
    version_vector: VersionVector,
    
    /// Application state (simplified - would be actual CRDT state in production)
    state: serde_json::Value,
}

impl CausalDAG {
    /// Create a new Causal DAG
    /// 
    /// # Arguments
    /// 
    /// * `node_id` - Unique identifier for this node
    /// * `snapshot_interval` - Operations per snapshot (default: 10,000)
    /// * `conflict_policy` - Default conflict resolution policy
    pub fn new(
        node_id: String,
        snapshot_interval: usize,
        conflict_policy: ConflictPolicy,
    ) -> Self {
        Self {
            operations: BTreeMap::new(),
            dependents: BTreeMap::new(),
            roots: BTreeSet::new(),
            snapshot_manager: SnapshotManager::new(100, snapshot_interval),
            conflict_resolver: ConflictResolver::new(conflict_policy),
            node_id,
            version_vector: VersionVector::new(),
            state: serde_json::json!({}),
        }
    }

    /// Insert an operation into the DAG
    pub fn insert(&mut self, op: CausalOp) -> Result<(), DagError> {
        // 1. Idempotence check
        if self.operations.contains_key(&op.id) {
            return Ok(());
        }

        // 2. Validate dependencies exist
        for dep_id in &op.dependencies {
            if !self.operations.contains_key(dep_id) {
                return Err(DagError::MissingDependency(dep_id.clone()));
            }
        }

        // 3. Check for conflicts with existing operations
        let mut merged_results = Vec::new();
        for existing_op in self.operations.values() {
            if op.conflicts_with(existing_op) {
                match self.conflict_resolver.resolve(&op, existing_op) {
                    ConflictResolution::NoConflict => {}
                    ConflictResolution::Winner(winner_id) => {
                        if winner_id != op.id {
                            return Err(DagError::ConflictRejected { winner_id });
                        }
                    }
                    ConflictResolution::Manual { option_a, option_b } => {
                        return Err(DagError::ManualResolutionRequired {
                            op_a: option_a.id,
                            op_b: option_b.id,
                        });
                    }
                    ConflictResolution::Merged { result } => {
                        merged_results.push(result.clone());
                    }
                }
            }
        }
        
        // Apply merged results after iteration
        for result in merged_results {
            self.apply_merged_result(&op, result);
        }

        // 4. Update reverse dependency index
        for dep_id in &op.dependencies {
            self.dependents
                .entry(dep_id.clone())
                .or_insert_with(BTreeSet::new)
                .insert(op.id.clone());
        }

        // 5. Track roots
        if op.dependencies.is_empty() {
            self.roots.insert(op.id.clone());
        }

        // 6. Merge version vectors
        self.version_vector = self.version_vector.merge(&op.version_vector);

        // 7. Apply operation to state
        self.apply_operation(&op);

        // 8. Insert operation
        self.operations.insert(op.id.clone(), op);

        // 9. Update snapshot manager
        self.snapshot_manager.increment_operation_count();

        // 10. Check if we should snapshot
        if self.snapshot_manager.should_snapshot() {
            self.create_snapshot();
        }

        // 11. Garbage collect old operations
        self.garbage_collect();

        Ok(())
    }

    /// Apply an operation to the state
    fn apply_operation(&mut self, op: &CausalOp) {
        // Simplified state updates (production would have proper CRDT logic)
        match op.operation.as_str() {
            "set" => {
                if let Some(key) = op.input.get("key").and_then(|k| k.as_str()) {
                    if let Some(value) = op.input.get("value") {
                        self.state[key] = value.clone();
                    }
                }
            }
            "increment" => {
                if let Some(key) = op.input.get("key").and_then(|k| k.as_str()) {
                    let delta = op.input.get("value").and_then(|v| v.as_i64()).unwrap_or(1);
                    let current = self.state.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
                    self.state[key] = serde_json::json!(current + delta);
                }
            }
            _ => {
                // Log unknown operation types
            }
        }
    }

    /// Apply merged conflict result
    fn apply_merged_result(&mut self, _op: &CausalOp, result: serde_json::Value) {
        // Merge the result into current state
        if let Some(obj) = result.as_object() {
            for (key, value) in obj {
                self.state[key] = value.clone();
            }
        }
    }

    /// Create a snapshot of current state
    fn create_snapshot(&mut self) {
        let merkle_root = self.compute_merkle_root();
        self.snapshot_manager.create_snapshot(
            self.state.clone(),
            merkle_root,
            self.version_vector.clone(),
        );
    }

    /// Garbage collect old operations
    fn garbage_collect(&mut self) {
        if let Some(threshold) = self.snapshot_manager.get_trimable_threshold() {
            // Count operations and remove old ones
            let to_remove: Vec<String> = self.operations
                .iter()
                .filter(|(_, op)| op.lamport_clock < threshold)
                .map(|(id, _)| id.clone())
                .collect();

            for id in to_remove {
                self.operations.remove(&id);
                self.roots.remove(&id);
                self.dependents.remove(&id);
            }
        }
    }

    /// Compute Merkle root of operation DAG
    pub fn compute_merkle_root(&self) -> String {
        let ordered_ops = self.causal_order();
        let mut hasher = Sha256::new();
        for op_id in ordered_ops {
            hasher.update(op_id.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Get operations in causal (topological) order
    pub fn causal_order(&self) -> Vec<String> {
        let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();

        for (op_id, op) in &self.operations {
            in_degree.insert(op_id.clone(), op.dependencies.len());
        }

        let mut queue: VecDeque<String> = self.roots.iter().cloned().collect();
        let mut result = Vec::new();

        while let Some(op_id) = queue.pop_front() {
            result.push(op_id.clone());

            if let Some(deps) = self.dependents.get(&op_id) {
                for dep_id in deps {
                    if let Some(degree) = in_degree.get_mut(dep_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Get current version vector
    pub fn get_version_vector(&self) -> &VersionVector {
        &self.version_vector
    }

    /// Get latest snapshot
    pub fn get_latest_snapshot(&self) -> Option<&Snapshot> {
        self.snapshot_manager.get_latest()
    }

    /// Get snapshot IDs for sync negotiation
    pub fn get_snapshot_ids(&self) -> Vec<String> {
        self.snapshot_manager.get_snapshot_ids()
    }

    /// Find common snapshot with peer
    pub fn find_common_snapshot(&self, peer_ids: &[String]) -> Option<&Snapshot> {
        self.snapshot_manager.find_common_snapshot(peer_ids)
    }

    /// Get operation by ID
    pub fn get_operation(&self, id: &str) -> Option<&CausalOp> {
        self.operations.get(id)
    }

    /// Get all operations after a certain Lamport clock
    pub fn get_operations_after(&self, after_lamport: u64) -> Vec<&CausalOp> {
        self.operations
            .values()
            .filter(|op| op.lamport_clock > after_lamport)
            .collect()
    }

    /// Get operation count
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    /// Get current state
    pub fn get_state(&self) -> &serde_json::Value {
        &self.state
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Set conflict resolver node priority
    pub fn set_node_priority(&mut self, node_id: String, priority: u8) {
        self.conflict_resolver.set_node_priority(node_id, priority);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn create_test_keypair() -> SigningKey {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn test_dag_creation() {
        let dag = CausalDAG::new(
            "node1".to_string(),
            100,
            ConflictPolicy::LastWriterWins,
        );

        assert_eq!(dag.operation_count(), 0);
        assert_eq!(dag.node_id(), "node1");
    }

    #[test]
    fn test_insert_operation() {
        let keypair = create_test_keypair();
        let mut dag = CausalDAG::new(
            "node1".to_string(),
            100,
            ConflictPolicy::LastWriterWins,
        );

        let mut vv = VersionVector::new();
        vv.increment("node1");

        let op = CausalOp::new(
            "set".to_string(),
            serde_json::json!({"key": "counter", "value": 42}),
            BTreeSet::new(),
            vv,
            "node1".to_string(),
            &keypair,
        );

        dag.insert(op.clone()).unwrap();

        assert_eq!(dag.operation_count(), 1);
        assert!(dag.get_operation(&op.id).is_some());
        assert_eq!(dag.get_state()["counter"], serde_json::json!(42));
    }

    #[test]
    fn test_causal_ordering() {
        let keypair = create_test_keypair();
        let mut dag = CausalDAG::new(
            "node1".to_string(),
            100,
            ConflictPolicy::LastWriterWins,
        );

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

        dag.insert(op1.clone()).unwrap();

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

        dag.insert(op2.clone()).unwrap();

        let order = dag.causal_order();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], op1.id);
        assert_eq!(order[1], op2.id);
    }

    #[test]
    fn test_missing_dependency() {
        let keypair = create_test_keypair();
        let mut dag = CausalDAG::new(
            "node1".to_string(),
            100,
            ConflictPolicy::LastWriterWins,
        );

        let mut vv = VersionVector::new();
        vv.increment("node1");

        let mut deps = BTreeSet::new();
        deps.insert("nonexistent".to_string());

        let op = CausalOp::new(
            "test".to_string(),
            serde_json::json!({}),
            deps,
            vv,
            "node1".to_string(),
            &keypair,
        );

        match dag.insert(op) {
            Err(DagError::MissingDependency(id)) => {
                assert_eq!(id, "nonexistent");
            }
            _ => panic!("Expected missing dependency error"),
        }
    }

    #[test]
    fn test_idempotent_insert() {
        let keypair = create_test_keypair();
        let mut dag = CausalDAG::new(
            "node1".to_string(),
            100,
            ConflictPolicy::LastWriterWins,
        );

        let mut vv = VersionVector::new();
        vv.increment("node1");

        let op = CausalOp::new(
            "test".to_string(),
            serde_json::json!({}),
            BTreeSet::new(),
            vv,
            "node1".to_string(),
            &keypair,
        );

        dag.insert(op.clone()).unwrap();
        dag.insert(op.clone()).unwrap();  // Should succeed (idempotent)

        assert_eq!(dag.operation_count(), 1);
    }

    #[test]
    fn test_snapshot_creation() {
        let keypair = create_test_keypair();
        let mut dag = CausalDAG::new(
            "node1".to_string(),
            10,  // Snapshot every 10 operations
            ConflictPolicy::LastWriterWins,
        );

        // Insert 10 operations to trigger snapshot
        for i in 0..10 {
            let mut vv = VersionVector::new();
            for _ in 0..=i {
                vv.increment("node1");
            }

            let op = CausalOp::new(
                format!("op{}", i),
                serde_json::json!({}),
                BTreeSet::new(),
                vv,
                "node1".to_string(),
                &keypair,
            );

            dag.insert(op).unwrap();
        }

        assert!(dag.get_latest_snapshot().is_some());
    }

    #[test]
    fn test_increment_operation() {
        let keypair = create_test_keypair();
        let mut dag = CausalDAG::new(
            "node1".to_string(),
            100,
            ConflictPolicy::LastWriterWins,
        );

        // First increment
        let mut vv1 = VersionVector::new();
        vv1.increment("node1");

        let op1 = CausalOp::new(
            "increment".to_string(),
            serde_json::json!({"key": "counter", "value": 5}),
            BTreeSet::new(),
            vv1.clone(),
            "node1".to_string(),
            &keypair,
        );

        dag.insert(op1.clone()).unwrap();

        // Second increment
        let mut vv2 = vv1.clone();
        vv2.increment("node1");

        let mut deps = BTreeSet::new();
        deps.insert(op1.id);

        let op2 = CausalOp::new(
            "increment".to_string(),
            serde_json::json!({"key": "counter", "value": 3}),
            deps,
            vv2,
            "node1".to_string(),
            &keypair,
        );

        dag.insert(op2).unwrap();

        // Counter should be 8 (5 + 3)
        assert_eq!(dag.get_state()["counter"], serde_json::json!(8));
    }
}
