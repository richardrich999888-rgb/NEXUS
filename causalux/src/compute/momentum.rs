//! Causal Momentum Buffer
//! 
//! Implements momentum tracking for gradient descent with causal ordering.
//! This enables async SGD where momentum is tracked per-node and merged.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::version_vector::VersionVector;
use super::tensor::Tensor;

/// Causal Momentum - tracks momentum per layer with version vectors
/// 
/// Traditional momentum: v = β*v + (1-β)*gradient
/// Causal momentum: merges momentum states from multiple nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalMomentum {
    /// Momentum buffer per layer
    pub buffers: HashMap<String, MomentumState>,
    /// Version vector for tracking updates
    pub version: VersionVector,
    /// Node ID
    pub node_id: String,
    /// Momentum coefficient (β, typically 0.9)
    pub beta: f32,
}

/// State for a single momentum buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumState {
    /// Current momentum value
    pub velocity: Tensor,
    /// Number of updates applied
    pub update_count: u64,
    /// Last update timestamp
    pub timestamp: u64,
}

impl CausalMomentum {
    /// Create a new causal momentum tracker
    pub fn new(node_id: String, beta: f32) -> Self {
        Self {
            buffers: HashMap::new(),
            version: VersionVector::new(),
            node_id,
            beta,
        }
    }

    /// Update momentum with a new gradient (local update)
    pub fn update(&mut self, layer_id: &str, gradient: &Tensor) -> Tensor {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        let new_velocity = if let Some(state) = self.buffers.get(layer_id) {
            // v = β*v + (1-β)*gradient
            let scaled_v = state.velocity.scale(self.beta);
            let scaled_g = gradient.scale(1.0 - self.beta);
            scaled_v.add(&scaled_g)
        } else {
            // First gradient, just scale it
            gradient.scale(1.0 - self.beta)
        };

        let state = MomentumState {
            velocity: new_velocity.clone(),
            update_count: self.buffers.get(layer_id)
                .map(|s| s.update_count + 1)
                .unwrap_or(1),
            timestamp,
        };

        self.buffers.insert(layer_id.to_string(), state);
        self.version.increment(&self.node_id);
        
        new_velocity
    }

    /// Merge momentum from another node (CRDT merge)
    /// 
    /// Uses weighted average based on update counts
    pub fn merge(&mut self, other: &CausalMomentum) {
        for (layer_id, other_state) in &other.buffers {
            if let Some(self_state) = self.buffers.get_mut(layer_id) {
                // Weighted average based on update counts
                let total_updates = self_state.update_count + other_state.update_count;
                let self_weight = self_state.update_count as f32 / total_updates as f32;
                
                let merged_velocity = self_state.velocity
                    .weighted_average(&other_state.velocity, self_weight);
                
                self_state.velocity = merged_velocity;
                self_state.update_count = total_updates;
                self_state.timestamp = self_state.timestamp.max(other_state.timestamp);
            } else {
                // We don't have this layer, take other's state
                self.buffers.insert(layer_id.clone(), other_state.clone());
            }
        }
        
        // Merge version vectors
        self.version = self.version.merge(&other.version);
    }

    /// Get current momentum for a layer
    pub fn get(&self, layer_id: &str) -> Option<&Tensor> {
        self.buffers.get(layer_id).map(|s| &s.velocity)
    }

    /// Reset momentum (for learning rate warmup)
    pub fn reset(&mut self) {
        self.buffers.clear();
        self.version = VersionVector::new();
    }
}

/// Adam-style momentum with first and second moments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalAdam {
    /// First moment (mean)
    pub m: CausalMomentum,
    /// Second moment (variance)
    pub v: CausalMomentum,
    /// Beta1 for first moment
    pub beta1: f32,
    /// Beta2 for second moment
    pub beta2: f32,
    /// Epsilon for numerical stability
    pub epsilon: f32,
    /// Step count for bias correction
    pub step: u64,
}

impl CausalAdam {
    pub fn new(node_id: String) -> Self {
        Self {
            m: CausalMomentum::new(node_id.clone(), 0.9),
            v: CausalMomentum::new(node_id, 0.999),
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            step: 0,
        }
    }

    /// Compute Adam update for a gradient
    pub fn update(&mut self, layer_id: &str, gradient: &Tensor, learning_rate: f32) -> Tensor {
        self.step += 1;
        
        // Update first moment
        let m_t = self.m.update(layer_id, gradient);
        
        // Update second moment (using squared gradient)
        let g_squared = Tensor::from_data(
            gradient.data.iter().map(|x| x * x).collect(),
            gradient.shape.clone(),
        );
        let v_t = self.v.update(layer_id, &g_squared);
        
        // Bias correction
        let beta1_correction = 1.0 - self.beta1.powi(self.step as i32);
        let beta2_correction = 1.0 - self.beta2.powi(self.step as i32);
        
        let m_hat = m_t.scale(1.0 / beta1_correction);
        let v_hat = v_t.scale(1.0 / beta2_correction);
        
        // Adam update: m_hat / (sqrt(v_hat) + epsilon)
        let update: Vec<f32> = m_hat.data.iter()
            .zip(v_hat.data.iter())
            .map(|(m, v)| learning_rate * m / (v.sqrt() + self.epsilon))
            .collect();
        
        Tensor::from_data(update, gradient.shape.clone())
    }

    /// Merge Adam state from another node
    pub fn merge(&mut self, other: &CausalAdam) {
        self.m.merge(&other.m);
        self.v.merge(&other.v);
        self.step = self.step.max(other.step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::tensor::TensorShape;

    #[test]
    fn test_momentum_update() {
        let mut momentum = CausalMomentum::new("node1".to_string(), 0.9);
        let grad = Tensor::ones(TensorShape::new(vec![2, 2]));
        
        let v1 = momentum.update("layer1", &grad);
        assert!(v1.data.iter().all(|&x| (x - 0.1).abs() < 0.001)); // (1-0.9)*1 = 0.1
        
        let v2 = momentum.update("layer1", &grad);
        // v2 = 0.9*0.1 + 0.1*1 = 0.09 + 0.1 = 0.19
        assert!(v2.data.iter().all(|&x| (x - 0.19).abs() < 0.001));
    }

    #[test]
    fn test_momentum_merge() {
        let mut m1 = CausalMomentum::new("node1".to_string(), 0.9);
        let mut m2 = CausalMomentum::new("node2".to_string(), 0.9);
        
        let grad1 = Tensor::from_data(vec![1.0, 1.0], TensorShape::new(vec![2]));
        let grad2 = Tensor::from_data(vec![2.0, 2.0], TensorShape::new(vec![2]));
        
        m1.update("layer1", &grad1);
        m2.update("layer1", &grad2);
        
        m1.merge(&m2);
        
        // Merged velocity should be weighted average
        let v = m1.get("layer1").unwrap();
        // (0.1 * 0.5) + (0.2 * 0.5) = 0.15
        assert!(v.data.iter().all(|&x| (x - 0.15).abs() < 0.001));
    }

    #[test]
    fn test_adam_update() {
        let mut adam = CausalAdam::new("node1".to_string());
        let grad = Tensor::ones(TensorShape::new(vec![2, 2]));
        
        let update = adam.update("layer1", &grad, 0.001);
        
        // Adam update should be non-zero
        assert!(update.data.iter().all(|&x| x.abs() > 0.0));
    }
}
