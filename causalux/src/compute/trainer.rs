//! Distributed Trainer
//! 
//! High-level API for distributed training using CausalGradient CRDTs.
//! Enables async, fault-tolerant gradient descent across GPU mesh.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use std::time::{Duration, Instant};
use super::tensor::{Tensor, TensorShape};
use super::gradient_crdt::{CausalGradient, GradientError};
use super::momentum::CausalAdam;
use super::mesh::{GPUMesh, MeshConfig, GPUNode};
use super::shard::{ModelShard, ShardRegistry};

/// Configuration for distributed training
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Node ID
    pub node_id: String,
    /// Learning rate
    pub learning_rate: f32,
    /// Batch size
    pub batch_size: usize,
    /// Number of epochs
    pub epochs: usize,
    /// Gradient clip norm
    pub max_grad_norm: f32,
    /// Sync frequency (every N batches)
    pub sync_frequency: usize,
    /// Use Adam optimizer
    pub use_adam: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 10,
            max_grad_norm: 1.0,
            sync_frequency: 10,
            use_adam: true,
        }
    }
}

/// Training metrics
#[derive(Debug, Clone, Default)]
pub struct TrainingMetrics {
    /// Total batches processed
    pub batches: usize,
    /// Total gradient operations
    pub gradient_ops: usize,
    /// Total sync operations
    pub sync_ops: usize,
    /// Average gradient norm
    pub avg_grad_norm: f32,
    /// Training loss history
    pub loss_history: Vec<f32>,
    /// Time spent computing
    pub compute_time: Duration,
    /// Time spent syncing
    pub sync_time: Duration,
}

/// Distributed Trainer - coordinates async gradient descent
pub struct DistributedTrainer {
    /// Configuration
    config: TrainingConfig,
    /// GPU mesh for P2P coordination
    mesh: GPUMesh,
    /// Adam optimizer (if enabled)
    adam: Option<CausalAdam>,
    /// Training metrics
    metrics: TrainingMetrics,
    /// Current epoch
    current_epoch: usize,
    /// Current batch
    current_batch: usize,
}

impl DistributedTrainer {
    /// Create a new distributed trainer
    pub fn new(config: TrainingConfig) -> Self {
        let mesh_config = MeshConfig {
            node_id: config.node_id.clone(),
            ..Default::default()
        };
        
        let adam = if config.use_adam {
            Some(CausalAdam::new(config.node_id.clone()))
        } else {
            None
        };
        
        Self {
            config: config.clone(),
            mesh: GPUMesh::new(mesh_config),
            adam,
            metrics: TrainingMetrics::default(),
            current_epoch: 0,
            current_batch: 0,
        }
    }

    /// Initialize model layers
    pub fn init_model(&mut self, layers: Vec<(String, TensorShape)>) {
        for (layer_id, shape) in layers {
            self.mesh.gradients_mut().init_layer(&layer_id, shape);
        }
    }

    /// Add a peer to the training mesh
    pub fn add_peer(&mut self, peer_id: String, capacity: f32) {
        let peer = GPUNode::new(peer_id, capacity);
        self.mesh.add_peer(peer);
    }

    /// Perform one training step (forward + backward + update)
    /// 
    /// In a real implementation, this would:
    /// 1. Forward pass through model
    /// 2. Compute loss
    /// 3. Backward pass to compute gradients
    /// 4. Apply gradients using CRDT
    /// 
    /// Here we simulate with random gradients.
    pub fn train_step(&mut self, gradients: HashMap<String, Tensor>) -> Result<f32, GradientError> {
        let start = Instant::now();
        
        let mut total_grad_norm = 0.0;
        let mut grad_count = 0;
        
        for (layer_id, gradient) in gradients {
            // Clip gradient
            let clipped = gradient.clip_norm(self.config.max_grad_norm);
            total_grad_norm += clipped.norm();
            grad_count += 1;
            
            // Apply gradient (with Adam if enabled)
            if let Some(ref mut adam) = self.adam {
                let update = adam.update(&layer_id, &clipped, self.config.learning_rate);
                // Apply update to weights
                if let Some(weights) = self.mesh.gradients_mut().get_weights(&layer_id) {
                    let new_weights = weights.sub(&update);
                    self.mesh.gradients_mut().weights.insert(layer_id.clone(), new_weights);
                }
            } else {
                self.mesh.compute_gradient(&layer_id, clipped)?;
            }
        }
        
        self.current_batch += 1;
        self.metrics.batches += 1;
        self.metrics.gradient_ops += grad_count;
        self.metrics.compute_time += start.elapsed();
        
        if grad_count > 0 {
            self.metrics.avg_grad_norm = 
                (self.metrics.avg_grad_norm * (self.metrics.batches - 1) as f32 
                 + total_grad_norm / grad_count as f32) / self.metrics.batches as f32;
        }
        
        // Sync periodically
        if self.current_batch % self.config.sync_frequency == 0 {
            // In real impl, would sync with peers here
            self.metrics.sync_ops += 1;
        }
        
        // Simulated loss (decreasing over time)
        let loss = 1.0 / (self.metrics.batches as f32 + 1.0);
        self.metrics.loss_history.push(loss);
        
        Ok(loss)
    }

    /// Train for one epoch
    pub fn train_epoch(&mut self, data_size: usize) -> Result<f32, GradientError> {
        let batches = data_size / self.config.batch_size;
        let mut epoch_loss = 0.0;
        
        for _ in 0..batches {
            // Simulate gradients for each layer
            let mut gradients = HashMap::new();
            for layer_id in self.mesh.gradients().layer_ids() {
                if let Some(weights) = self.mesh.gradients().get_weights(&layer_id) {
                    let grad = Tensor::random(weights.shape.clone());
                    gradients.insert(layer_id, grad);
                }
            }
            
            let loss = self.train_step(gradients)?;
            epoch_loss += loss;
        }
        
        self.current_epoch += 1;
        self.current_batch = 0;
        
        Ok(epoch_loss / batches as f32)
    }

    /// Full training loop
    pub fn train(&mut self, data_size: usize) -> Result<Vec<f32>, GradientError> {
        let mut epoch_losses = Vec::new();
        
        for _ in 0..self.config.epochs {
            let loss = self.train_epoch(data_size)?;
            epoch_losses.push(loss);
        }
        
        Ok(epoch_losses)
    }

    /// Sync with another trainer (P2P)
    pub fn sync_with(&mut self, other: &mut DistributedTrainer) {
        let start = Instant::now();
        
        self.mesh.sync_with(&mut other.mesh);
        
        // Merge Adam state if both use it
        if let (Some(ref mut self_adam), Some(ref other_adam)) = (&mut self.adam, &other.adam) {
            self_adam.merge(other_adam);
        }
        
        self.metrics.sync_ops += 1;
        self.metrics.sync_time += start.elapsed();
    }

    /// Get current weights for a layer
    pub fn get_weights(&self, layer_id: &str) -> Option<&Tensor> {
        self.mesh.gradients().get_weights(layer_id)
    }

    /// Get training metrics
    pub fn metrics(&self) -> &TrainingMetrics {
        &self.metrics
    }

    /// Get mesh statistics
    pub fn mesh_stats(&self) -> super::mesh::MeshStats {
        self.mesh.stats()
    }

    /// Export model weights
    pub fn export_weights(&self) -> HashMap<String, Tensor> {
        self.mesh.gradients().weights.clone()
    }

    /// Import model weights
    pub fn import_weights(&mut self, weights: HashMap<String, Tensor>) {
        for (layer_id, tensor) in weights {
            self.mesh.gradients_mut().weights.insert(layer_id, tensor);
        }
    }
}

/// Federated learning coordinator
/// 
/// Coordinates multiple trainers for federated learning scenarios
pub struct FederatedCoordinator {
    /// Central gradient aggregator
    aggregator: CausalGradient,
    /// Round number
    round: usize,
    /// Participating nodes
    nodes: Vec<String>,
}

impl FederatedCoordinator {
    pub fn new(coordinator_id: String) -> Self {
        Self {
            aggregator: CausalGradient::new(coordinator_id, 0.001),
            round: 0,
            nodes: Vec::new(),
        }
    }

    /// Register a participating node
    pub fn register_node(&mut self, node_id: String) {
        self.nodes.push(node_id);
    }

    /// Aggregate gradients from multiple trainers
    pub fn aggregate(&mut self, trainers: &[&DistributedTrainer]) {
        for trainer in trainers {
            self.aggregator.merge(trainer.mesh.gradients());
        }
        self.round += 1;
    }

    /// Distribute aggregated weights to trainers
    pub fn distribute(&self, trainers: &mut [DistributedTrainer]) {
        for trainer in trainers {
            for (layer_id, weights) in &self.aggregator.weights {
                trainer.mesh.gradients_mut().weights.insert(layer_id.clone(), weights.clone());
            }
        }
    }

    /// Get current round
    pub fn round(&self) -> usize {
        self.round
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trainer_creation() {
        let config = TrainingConfig::default();
        let trainer = DistributedTrainer::new(config);
        
        assert_eq!(trainer.current_epoch, 0);
        assert_eq!(trainer.current_batch, 0);
    }

    #[test]
    fn test_model_initialization() {
        let config = TrainingConfig::default();
        let mut trainer = DistributedTrainer::new(config);
        
        trainer.init_model(vec![
            ("fc1".to_string(), TensorShape::new(vec![10, 10])),
            ("fc2".to_string(), TensorShape::new(vec![10, 5])),
        ]);
        
        assert!(trainer.get_weights("fc1").is_some());
        assert!(trainer.get_weights("fc2").is_some());
    }

    #[test]
    fn test_train_step() {
        let config = TrainingConfig::default();
        let mut trainer = DistributedTrainer::new(config);
        
        trainer.init_model(vec![
            ("fc1".to_string(), TensorShape::new(vec![4, 4])),
        ]);
        
        let mut gradients = HashMap::new();
        gradients.insert("fc1".to_string(), Tensor::random(TensorShape::new(vec![4, 4])));
        
        let loss = trainer.train_step(gradients);
        assert!(loss.is_ok());
        assert_eq!(trainer.metrics.batches, 1);
    }

    #[test]
    fn test_distributed_training() {
        let config1 = TrainingConfig {
            node_id: "node1".to_string(),
            epochs: 2,
            ..Default::default()
        };
        let config2 = TrainingConfig {
            node_id: "node2".to_string(),
            epochs: 2,
            ..Default::default()
        };
        
        let mut trainer1 = DistributedTrainer::new(config1);
        let mut trainer2 = DistributedTrainer::new(config2);
        
        // Same model architecture
        let layers = vec![
            ("fc1".to_string(), TensorShape::new(vec![4, 4])),
        ];
        trainer1.init_model(layers.clone());
        trainer2.init_model(layers);
        
        // Train independently
        let mut grads1 = HashMap::new();
        grads1.insert("fc1".to_string(), Tensor::random(TensorShape::new(vec![4, 4])));
        trainer1.train_step(grads1).unwrap();
        
        let mut grads2 = HashMap::new();
        grads2.insert("fc1".to_string(), Tensor::random(TensorShape::new(vec![4, 4])));
        trainer2.train_step(grads2).unwrap();
        
        // Sync
        trainer1.sync_with(&mut trainer2);
        
        // Both should have updated
        assert!(trainer1.metrics.sync_ops > 0);
    }

    #[test]
    fn test_federated_learning() {
        let mut coord = FederatedCoordinator::new("coordinator".to_string());
        
        let config1 = TrainingConfig {
            node_id: "node1".to_string(),
            ..Default::default()
        };
        let config2 = TrainingConfig {
            node_id: "node2".to_string(),
            ..Default::default()
        };
        
        let mut trainer1 = DistributedTrainer::new(config1);
        let mut trainer2 = DistributedTrainer::new(config2);
        
        coord.register_node("node1".to_string());
        coord.register_node("node2".to_string());
        
        // Initialize same model
        let layers = vec![("fc1".to_string(), TensorShape::new(vec![4]))];
        trainer1.init_model(layers.clone());
        trainer2.init_model(layers);
        
        // Train locally
        let mut grads = HashMap::new();
        grads.insert("fc1".to_string(), Tensor::ones(TensorShape::new(vec![4])));
        trainer1.train_step(grads.clone()).unwrap();
        trainer2.train_step(grads).unwrap();
        
        // Aggregate at coordinator
        coord.aggregate(&[&trainer1, &trainer2]);
        
        // Distribute back
        coord.distribute(&mut [trainer1, trainer2]);
        
        assert_eq!(coord.round(), 1);
    }
}
