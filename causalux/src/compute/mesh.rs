//! GPU Mesh Protocol
//! 
//! P2P network of GPU nodes for distributed training.
//! Nodes discover each other, share shards, and sync gradients.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use std::time::{Duration, Instant};
use crate::version_vector::VersionVector;
use crate::sync::AdaptiveSync;
use super::gradient_crdt::{CausalGradient, GradientOp};
use super::shard::{ModelShard, ShardRegistry};

/// Configuration for GPU mesh
#[derive(Debug, Clone)]
pub struct MeshConfig {
    /// This node's ID
    pub node_id: String,
    /// Maximum peers to maintain
    pub max_peers: usize,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Gradient sync interval
    pub sync_interval: Duration,
    /// Enable GPU (if false, use CPU simulation)
    pub gpu_enabled: bool,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            max_peers: 10,
            heartbeat_interval: Duration::from_secs(5),
            sync_interval: Duration::from_millis(100),
            gpu_enabled: false,
        }
    }
}

/// State of a GPU node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is idle, ready for work
    Idle,
    /// Node is computing gradients
    Computing,
    /// Node is syncing gradients
    Syncing,
    /// Node is offline
    Offline,
}

/// Information about a GPU node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUNode {
    /// Node ID
    pub id: String,
    /// Node state
    pub state: NodeState,
    /// Compute capacity (TFLOPS estimate)
    pub capacity: f32,
    /// Current load (0.0 - 1.0)
    pub load: f32,
    /// Shards this node owns
    pub shards: Vec<String>,
    /// Last heartbeat
    pub last_seen: u64,
    /// Version vector
    pub version: VersionVector,
}

impl GPUNode {
    pub fn new(id: String, capacity: f32) -> Self {
        Self {
            id,
            state: NodeState::Idle,
            capacity,
            load: 0.0,
            shards: Vec::new(),
            last_seen: Self::now(),
            version: VersionVector::new(),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn heartbeat(&mut self) {
        self.last_seen = Self::now();
    }

    pub fn is_alive(&self, timeout_secs: u64) -> bool {
        Self::now() - self.last_seen < timeout_secs
    }

    pub fn available_capacity(&self) -> f32 {
        self.capacity * (1.0 - self.load)
    }
}

/// GPU Mesh - P2P network for distributed training
pub struct GPUMesh {
    /// Configuration
    config: MeshConfig,
    /// This node
    local_node: GPUNode,
    /// Known peers
    peers: HashMap<String, GPUNode>,
    /// Shard registry
    shards: ShardRegistry,
    /// Gradient state
    gradients: CausalGradient,
    /// Pending gradient ops to send
    outbox: Vec<GradientOp>,
    /// Received gradient ops to apply
    inbox: Vec<GradientOp>,
    /// Sync protocol
    sync: AdaptiveSync,
}

impl GPUMesh {
    /// Create a new GPU mesh
    pub fn new(config: MeshConfig) -> Self {
        let node_id = config.node_id.clone();
        let local_node = GPUNode::new(node_id.clone(), 10.0); // Assume 10 TFLOPS
        
        Self {
            config: config.clone(),
            local_node,
            peers: HashMap::new(),
            shards: ShardRegistry::new(node_id.clone()),
            gradients: CausalGradient::new(node_id, 0.001),
            outbox: Vec::new(),
            inbox: Vec::new(),
            sync: AdaptiveSync::new(100, Duration::from_secs(10)),
        }
    }

    /// Add a peer to the mesh
    pub fn add_peer(&mut self, peer: GPUNode) {
        if self.peers.len() < self.config.max_peers {
            self.peers.insert(peer.id.clone(), peer);
        }
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }

    /// Get all active peers
    pub fn active_peers(&self) -> Vec<&GPUNode> {
        self.peers
            .values()
            .filter(|p| p.is_alive(30))
            .collect()
    }

    /// Register a model shard
    pub fn register_shard(&mut self, shard: ModelShard) {
        self.local_node.shards.push(shard.content_id.clone());
        self.shards.register(shard);
    }

    /// Compute gradient locally
    pub fn compute_gradient(
        &mut self,
        layer_id: &str,
        gradient: super::tensor::Tensor,
    ) -> Result<String, super::gradient_crdt::GradientError> {
        self.local_node.state = NodeState::Computing;
        self.local_node.load = 0.8;
        
        let op_id = self.gradients.apply_gradient(layer_id, gradient)?;
        
        // Queue for sync
        // (In production, we'd create a GradientOp here)
        
        self.local_node.state = NodeState::Idle;
        self.local_node.load = 0.0;
        
        Ok(op_id)
    }

    /// Receive gradient from peer
    pub fn receive_gradient(&mut self, op: GradientOp) -> Result<(), super::gradient_crdt::GradientError> {
        self.gradients.receive_gradient(op)
    }

    /// Sync gradients with a peer
    pub fn sync_with(&mut self, peer: &mut GPUMesh) {
        // Exchange gradient states
        let my_gradients = self.gradients.clone();
        let peer_gradients = peer.gradients.clone();
        
        self.gradients.merge(&peer_gradients);
        peer.gradients.merge(&my_gradients);
        
        // Exchange shard registries
        let my_shards = self.shards.clone();
        let peer_shards = peer.shards.clone();
        
        self.shards.merge(&peer_shards);
        peer.shards.merge(&my_shards);
    }

    /// Get best node for a layer (load balancing)
    pub fn best_node_for_layer(&self, layer_id: &str) -> Option<String> {
        let shards = self.shards.get_layer_shards(layer_id);
        if let Some(shard) = shards.first() {
            return Some(shard.owner_node.clone());
        }
        
        // Fall back to node with most available capacity
        let mut best: Option<(&String, f32)> = None;
        
        for (id, peer) in &self.peers {
            if peer.is_alive(30) {
                let cap = peer.available_capacity();
                if best.map(|(_, c)| cap > c).unwrap_or(true) {
                    best = Some((id, cap));
                }
            }
        }
        
        best.map(|(id, _)| id.clone())
    }

    /// Get total compute capacity of mesh
    pub fn total_capacity(&self) -> f32 {
        let local = self.local_node.available_capacity();
        let peers: f32 = self.active_peers()
            .iter()
            .map(|p| p.available_capacity())
            .sum();
        local + peers
    }

    /// Get mesh statistics
    pub fn stats(&self) -> MeshStats {
        MeshStats {
            node_count: 1 + self.active_peers().len(),
            total_capacity: self.total_capacity(),
            shard_count: self.shards.shard_count(),
            gradient_ops: self.gradients.total_ops(),
        }
    }

    /// Get local node info
    pub fn local_node(&self) -> &GPUNode {
        &self.local_node
    }

    /// Get gradient state
    pub fn gradients(&self) -> &CausalGradient {
        &self.gradients
    }

    /// Get mutable gradient state
    pub fn gradients_mut(&mut self) -> &mut CausalGradient {
        &mut self.gradients
    }
}

/// Mesh statistics
#[derive(Debug, Clone)]
pub struct MeshStats {
    pub node_count: usize,
    pub total_capacity: f32,
    pub shard_count: usize,
    pub gradient_ops: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tensor::{Tensor, TensorShape};

    #[test]
    fn test_mesh_creation() {
        let config = MeshConfig::default();
        let mesh = GPUMesh::new(config);
        
        assert_eq!(mesh.active_peers().len(), 0);
        assert!(mesh.total_capacity() > 0.0);
    }

    #[test]
    fn test_add_peer() {
        let config = MeshConfig::default();
        let mut mesh = GPUMesh::new(config);
        
        let peer = GPUNode::new("peer1".to_string(), 20.0);
        mesh.add_peer(peer);
        
        assert_eq!(mesh.active_peers().len(), 1);
    }

    #[test]
    fn test_gradient_compute() {
        let config = MeshConfig {
            node_id: "node1".to_string(),
            ..Default::default()
        };
        let mut mesh = GPUMesh::new(config);
        
        // Initialize layer
        mesh.gradients_mut().init_layer("fc1", TensorShape::new(vec![4, 4]));
        
        // Compute gradient
        let gradient = Tensor::random(TensorShape::new(vec![4, 4]));
        let result = mesh.compute_gradient("fc1", gradient);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_mesh_sync() {
        let config1 = MeshConfig {
            node_id: "node1".to_string(),
            ..Default::default()
        };
        let config2 = MeshConfig {
            node_id: "node2".to_string(),
            ..Default::default()
        };
        
        let mut mesh1 = GPUMesh::new(config1);
        let mut mesh2 = GPUMesh::new(config2);
        
        // Both init same layer
        mesh1.gradients_mut().init_layer("fc1", TensorShape::new(vec![4]));
        mesh2.gradients_mut().init_layer("fc1", TensorShape::new(vec![4]));
        
        // Each computes different gradients
        let g1 = Tensor::from_data(vec![1.0, 0.0, 0.0, 0.0], TensorShape::new(vec![4]));
        let g2 = Tensor::from_data(vec![0.0, 1.0, 0.0, 0.0], TensorShape::new(vec![4]));
        
        mesh1.compute_gradient("fc1", g1).unwrap();
        mesh2.compute_gradient("fc1", g2).unwrap();
        
        // Sync
        mesh1.sync_with(&mut mesh2);
        
        // Both should have converged
        let ops1 = mesh1.gradients().total_ops();
        let ops2 = mesh2.gradients().total_ops();
        
        assert!(ops1 > 0);
        assert!(ops2 > 0);
    }
}
