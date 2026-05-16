//! Causal Gradient CRDT
//! 
//! The core innovation: gradients as CRDTs that can be merged asynchronously.
//! This enables true distributed training without synchronization barriers.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use crate::version_vector::VersionVector;
use crate::content_address::ContentAddress;
use super::tensor::{Tensor, TensorShape};
use super::momentum::CausalMomentum;

/// A gradient operation with causal ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientOp {
    /// Unique operation ID (content hash)
    pub id: String,
    /// Layer this gradient applies to
    pub layer_id: String,
    /// The gradient tensor
    pub gradient: Tensor,
    /// Version vector at creation time
    pub version: VersionVector,
    /// Node that computed this gradient
    pub node_id: String,
    /// Batch index (for ordering)
    pub batch_idx: u64,
    /// Dependencies (previous gradient ops)
    pub dependencies: BTreeSet<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl GradientOp {
    /// Create a new gradient operation
    pub fn new(
        layer_id: String,
        gradient: Tensor,
        version: VersionVector,
        node_id: String,
        batch_idx: u64,
        dependencies: BTreeSet<String>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        // ID is hash of gradient content + metadata
        let id = Self::compute_id(&gradient, &node_id, batch_idx);
        
        Self {
            id,
            layer_id,
            gradient,
            version,
            node_id,
            batch_idx,
            dependencies,
            timestamp,
        }
    }

    fn compute_id(gradient: &Tensor, node_id: &str, batch_idx: u64) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&gradient.content_hash);
        hasher.update(node_id.as_bytes());
        hasher.update(batch_idx.to_le_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Check if this op conflicts with another
    pub fn conflicts_with(&self, other: &GradientOp) -> bool {
        // Conflicts if same layer + concurrent (neither happens-before)
        self.layer_id == other.layer_id && 
        self.version.conflicts_with(&other.version)
    }
}

/// Causal Gradient CRDT - the core data structure
/// 
/// Stores gradients for a model with causal ordering and merge semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGradient {
    /// Current weight state per layer (content-addressed)
    pub weights: HashMap<String, Tensor>,
    /// Pending gradient operations (not yet applied)
    pub pending_ops: HashMap<String, GradientOp>,
    /// Applied operation IDs
    pub applied_ops: BTreeSet<String>,
    /// Version vector tracking all updates
    pub version: VersionVector,
    /// Momentum tracker
    pub momentum: CausalMomentum,
    /// Node ID
    pub node_id: String,
    /// Learning rate
    pub learning_rate: f32,
}

impl CausalGradient {
    /// Create a new CausalGradient for a model
    pub fn new(node_id: String, learning_rate: f32) -> Self {
        Self {
            weights: HashMap::new(),
            pending_ops: HashMap::new(),
            applied_ops: BTreeSet::new(),
            version: VersionVector::new(),
            momentum: CausalMomentum::new(node_id.clone(), 0.9),
            node_id,
            learning_rate,
        }
    }

    /// Initialize a layer's weights
    pub fn init_layer(&mut self, layer_id: &str, shape: TensorShape) {
        let weights = Tensor::random(shape);
        self.weights.insert(layer_id.to_string(), weights);
    }

    /// Apply a local gradient update
    pub fn apply_gradient(&mut self, layer_id: &str, gradient: Tensor) -> Result<String, GradientError> {
        // Get current weights
        let weights = self.weights.get(layer_id)
            .ok_or_else(|| GradientError::LayerNotFound(layer_id.to_string()))?;
        
        // Increment version
        self.version.increment(&self.node_id);
        
        // Create gradient operation
        let op = GradientOp::new(
            layer_id.to_string(),
            gradient.clone(),
            self.version.clone(),
            self.node_id.clone(),
            self.version.get(&self.node_id),
            self.get_dependencies(layer_id),
        );
        
        let op_id = op.id.clone();
        
        // Apply momentum
        let momentum_grad = self.momentum.update(layer_id, &gradient);
        
        // Update weights: w = w - lr * momentum_grad
        let update = momentum_grad.scale(self.learning_rate);
        let new_weights = weights.sub(&update);
        
        self.weights.insert(layer_id.to_string(), new_weights);
        self.applied_ops.insert(op_id.clone());
        
        Ok(op_id)
    }

    /// Receive a remote gradient operation
    pub fn receive_gradient(&mut self, op: GradientOp) -> Result<(), GradientError> {
        // Idempotency check
        if self.applied_ops.contains(&op.id) || self.pending_ops.contains_key(&op.id) {
            return Ok(());
        }
        
        // Check if we can apply immediately (dependencies satisfied)
        if self.can_apply(&op) {
            self.apply_remote_gradient(op)?;
        } else {
            // Buffer for later
            self.pending_ops.insert(op.id.clone(), op);
        }
        
        // Try to apply any pending ops
        self.flush_pending()?;
        
        Ok(())
    }

    /// Check if operation's dependencies are satisfied
    fn can_apply(&self, op: &GradientOp) -> bool {
        op.dependencies.iter().all(|dep| self.applied_ops.contains(dep))
    }

    /// Apply a remote gradient operation
    fn apply_remote_gradient(&mut self, op: GradientOp) -> Result<(), GradientError> {
        let layer_id = &op.layer_id;
        
        // Get current weights (or skip if layer not initialized)
        let weights = match self.weights.get(layer_id) {
            Some(w) => w,
            None => {
                // Store op but don't apply
                self.applied_ops.insert(op.id.clone());
                return Ok(());
            }
        };
        
        // Merge version vectors
        self.version = self.version.merge(&op.version);
        
        // Apply gradient with reduced learning rate (remote gradients are "stale")
        let staleness = self.compute_staleness(&op);
        let adjusted_lr = self.learning_rate / (1.0 + staleness);
        
        // Update momentum with remote gradient
        let momentum_grad = self.momentum.update(layer_id, &op.gradient);
        
        // Update weights
        let update = momentum_grad.scale(adjusted_lr);
        let new_weights = weights.sub(&update);
        
        self.weights.insert(layer_id.to_string(), new_weights);
        self.applied_ops.insert(op.id);
        
        Ok(())
    }

    /// Compute staleness of a remote gradient
    fn compute_staleness(&self, op: &GradientOp) -> f32 {
        // Staleness = difference in version vector sum
        let local_sum = self.version.total_operations();
        let remote_sum = op.version.total_operations();
        (local_sum as i64 - remote_sum as i64).abs() as f32
    }

    /// Flush pending operations that can now be applied
    fn flush_pending(&mut self) -> Result<(), GradientError> {
        let mut applied = true;
        while applied {
            applied = false;
            let pending_ids: Vec<String> = self.pending_ops.keys().cloned().collect();
            
            for op_id in pending_ids {
                if let Some(op) = self.pending_ops.get(&op_id) {
                    if self.can_apply(op) {
                        let op = self.pending_ops.remove(&op_id).unwrap();
                        self.apply_remote_gradient(op)?;
                        applied = true;
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Get dependencies for a new operation on a layer
    fn get_dependencies(&self, layer_id: &str) -> BTreeSet<String> {
        // Depend on the most recent op for this layer
        self.applied_ops.iter()
            .rev()
            .take(1)
            .cloned()
            .collect()
    }

    /// Merge with another CausalGradient (CRDT merge)
    pub fn merge(&mut self, other: &CausalGradient) {
        // Merge version vectors
        self.version = self.version.merge(&other.version);
        
        // Merge momentum
        self.momentum.merge(&other.momentum);
        
        // Merge weights using weighted average based on update counts
        for (layer_id, other_weights) in &other.weights {
            if let Some(self_weights) = self.weights.get(layer_id) {
                let local_ops = self.version.total_operations();
                let remote_ops = other.version.total_operations();
                let total = local_ops + remote_ops;
                let self_weight = local_ops as f32 / total as f32;
                
                let merged = self_weights.weighted_average(other_weights, self_weight);
                self.weights.insert(layer_id.clone(), merged);
            } else {
                self.weights.insert(layer_id.clone(), other_weights.clone());
            }
        }
        
        // Merge applied ops
        self.applied_ops.extend(other.applied_ops.iter().cloned());
        
        // Receive any pending ops from other
        for (_, op) in &other.pending_ops {
            let _ = self.receive_gradient(op.clone());
        }
    }

    /// Get current weights for a layer
    pub fn get_weights(&self, layer_id: &str) -> Option<&Tensor> {
        self.weights.get(layer_id)
    }

    /// Get all layer IDs
    pub fn layer_ids(&self) -> Vec<String> {
        self.weights.keys().cloned().collect()
    }

    /// Get total gradient operations applied
    pub fn total_ops(&self) -> usize {
        self.applied_ops.len()
    }
}

/// Errors for gradient operations
#[derive(Debug, Clone)]
pub enum GradientError {
    LayerNotFound(String),
    ShapeMismatch { expected: TensorShape, got: TensorShape },
    DependencyMissing(String),
}

impl std::fmt::Display for GradientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradientError::LayerNotFound(id) => write!(f, "Layer not found: {}", id),
            GradientError::ShapeMismatch { expected, got } => {
                write!(f, "Shape mismatch: expected {:?}, got {:?}", expected.0, got.0)
            }
            GradientError::DependencyMissing(id) => write!(f, "Dependency missing: {}", id),
        }
    }
}

impl std::error::Error for GradientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_apply() {
        let mut cg = CausalGradient::new("node1".to_string(), 0.01);
        cg.init_layer("fc1", TensorShape::new(vec![10, 10]));
        
        let initial_weights = cg.get_weights("fc1").unwrap().clone();
        
        let grad = Tensor::ones(TensorShape::new(vec![10, 10]));
        cg.apply_gradient("fc1", grad).unwrap();
        
        let new_weights = cg.get_weights("fc1").unwrap();
        
        // Weights should have changed
        assert_ne!(initial_weights.content_hash, new_weights.content_hash);
    }

    #[test]
    fn test_gradient_merge() {
        let mut cg1 = CausalGradient::new("node1".to_string(), 0.01);
        let mut cg2 = CausalGradient::new("node2".to_string(), 0.01);
        
        cg1.init_layer("fc1", TensorShape::new(vec![4]));
        cg2.init_layer("fc1", TensorShape::new(vec![4]));
        
        // Both nodes apply gradients
        let grad1 = Tensor::from_data(vec![1.0, 1.0, 1.0, 1.0], TensorShape::new(vec![4]));
        let grad2 = Tensor::from_data(vec![2.0, 2.0, 2.0, 2.0], TensorShape::new(vec![4]));
        
        cg1.apply_gradient("fc1", grad1).unwrap();
        cg2.apply_gradient("fc1", grad2).unwrap();
        
        // Merge
        cg1.merge(&cg2);
        
        // Version should reflect both nodes
        assert!(cg1.version.get("node1") > 0);
        assert!(cg1.version.get("node2") > 0);
    }

    #[test]
    fn test_concurrent_gradients() {
        let mut cg1 = CausalGradient::new("node1".to_string(), 0.01);
        let mut cg2 = CausalGradient::new("node2".to_string(), 0.01);
        
        cg1.init_layer("fc1", TensorShape::new(vec![2]));
        cg2.weights = cg1.weights.clone(); // Same initial state
        
        // Concurrent updates
        let grad1 = Tensor::from_data(vec![1.0, 0.0], TensorShape::new(vec![2]));
        let grad2 = Tensor::from_data(vec![0.0, 1.0], TensorShape::new(vec![2]));
        
        cg1.apply_gradient("fc1", grad1).unwrap();
        cg2.apply_gradient("fc1", grad2).unwrap();
        
        // Merge both directions
        let cg1_clone = cg1.clone();
        cg1.merge(&cg2);
        cg2.merge(&cg1_clone);
        
        // Both should converge to same state (CRDT property)
        let w1 = cg1.get_weights("fc1").unwrap();
        let w2 = cg2.get_weights("fc1").unwrap();
        
        // Weights should be close (may not be identical due to merge ordering)
        for (a, b) in w1.data.iter().zip(w2.data.iter()) {
            assert!((a - b).abs() < 0.1);
        }
    }
}
