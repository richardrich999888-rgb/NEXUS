//! Model Sharding with Content Addressing
//! 
//! Shards model weights across nodes using content addressing.
//! Each shard is identified by its content hash, enabling deduplication
//! and efficient sync.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use crate::content_address::ContentAddress;
use crate::version_vector::VersionVector;
use super::tensor::{Tensor, TensorShape};

/// A shard of model weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelShard {
    /// Content address (hash-based ID)
    pub content_id: String,
    /// Layer name
    pub layer_id: String,
    /// Shard index within layer (for large layers split across nodes)
    pub shard_idx: usize,
    /// Total shards for this layer
    pub total_shards: usize,
    /// Weight tensor
    pub weights: Tensor,
    /// Version when this shard was created
    pub version: VersionVector,
    /// Node that owns this shard
    pub owner_node: String,
    /// Dependencies (layers that must be computed before this one)
    pub dependencies: BTreeSet<String>,
}

impl ModelShard {
    /// Create a new model shard
    pub fn new(
        layer_id: String,
        shard_idx: usize,
        total_shards: usize,
        weights: Tensor,
        owner_node: String,
        dependencies: BTreeSet<String>,
    ) -> Self {
        let content_id = Self::compute_content_id(&layer_id, shard_idx, &weights);
        Self {
            content_id,
            layer_id,
            shard_idx,
            total_shards,
            weights,
            version: VersionVector::new(),
            owner_node,
            dependencies,
        }
    }

    fn compute_content_id(layer_id: &str, shard_idx: usize, weights: &Tensor) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(layer_id.as_bytes());
        hasher.update(shard_idx.to_le_bytes());
        hasher.update(&weights.content_hash);
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Update weights and recompute content ID
    pub fn update_weights(&mut self, new_weights: Tensor) {
        self.content_id = Self::compute_content_id(&self.layer_id, self.shard_idx, &new_weights);
        self.weights = new_weights;
        self.version.increment(&self.owner_node);
    }
}

/// Registry for model shards across the network
#[derive(Debug, Clone)]
pub struct ShardRegistry {
    /// Shards by content ID
    shards: HashMap<String, ModelShard>,
    /// Layer to shard mapping
    layer_shards: HashMap<String, Vec<String>>,
    /// Node to shard mapping
    node_shards: HashMap<String, Vec<String>>,
    /// This node's ID
    node_id: String,
    /// Version vector
    version: VersionVector,
}

impl ShardRegistry {
    /// Create a new shard registry
    pub fn new(node_id: String) -> Self {
        Self {
            shards: HashMap::new(),
            layer_shards: HashMap::new(),
            node_shards: HashMap::new(),
            node_id,
            version: VersionVector::new(),
        }
    }

    /// Register a shard
    pub fn register(&mut self, shard: ModelShard) {
        let content_id = shard.content_id.clone();
        let layer_id = shard.layer_id.clone();
        let owner_node = shard.owner_node.clone();

        // Add to layer mapping
        self.layer_shards
            .entry(layer_id)
            .or_insert_with(Vec::new)
            .push(content_id.clone());

        // Add to node mapping
        self.node_shards
            .entry(owner_node)
            .or_insert_with(Vec::new)
            .push(content_id.clone());

        // Store shard
        self.shards.insert(content_id, shard);
        self.version.increment(&self.node_id);
    }

    /// Get a shard by content ID
    pub fn get(&self, content_id: &str) -> Option<&ModelShard> {
        self.shards.get(content_id)
    }

    /// Get all shards for a layer
    pub fn get_layer_shards(&self, layer_id: &str) -> Vec<&ModelShard> {
        self.layer_shards
            .get(layer_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.shards.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all shards owned by a node
    pub fn get_node_shards(&self, node_id: &str) -> Vec<&ModelShard> {
        self.node_shards
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.shards.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get shards this node owns
    pub fn my_shards(&self) -> Vec<&ModelShard> {
        self.get_node_shards(&self.node_id)
    }

    /// Shard a model across nodes
    pub fn shard_model(
        &mut self,
        layers: Vec<(String, Tensor, BTreeSet<String>)>,
        nodes: &[String],
    ) -> Vec<ModelShard> {
        let mut result = Vec::new();
        
        for (i, (layer_id, weights, deps)) in layers.into_iter().enumerate() {
            // Round-robin assignment to nodes
            let owner_node = nodes[i % nodes.len()].clone();
            
            let shard = ModelShard::new(
                layer_id,
                0,      // Single shard per layer for now
                1,      // Total shards
                weights,
                owner_node,
                deps,
            );
            
            self.register(shard.clone());
            result.push(shard);
        }
        
        result
    }

    /// Get execution order based on dependencies (topological sort)
    pub fn execution_order(&self) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for shard in self.shards.values() {
            in_degree.insert(shard.layer_id.clone(), shard.dependencies.len());
            for dep in &shard.dependencies {
                dependents
                    .entry(dep.clone())
                    .or_insert_with(Vec::new)
                    .push(shard.layer_id.clone());
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut order = Vec::new();

        while let Some(layer_id) = queue.pop() {
            order.push(layer_id.clone());
            
            if let Some(deps) = dependents.get(&layer_id) {
                for dep in deps {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        order
    }

    /// Merge with another registry
    pub fn merge(&mut self, other: &ShardRegistry) {
        for (content_id, shard) in &other.shards {
            if !self.shards.contains_key(content_id) {
                self.register(shard.clone());
            } else {
                // Merge: keep newer version
                let existing = self.shards.get_mut(content_id).unwrap();
                if shard.version.total_operations() > existing.version.total_operations() {
                    *existing = shard.clone();
                }
            }
        }
        
        self.version = self.version.merge(&other.version);
    }

    /// Get layer count
    pub fn layer_count(&self) -> usize {
        self.layer_shards.len()
    }

    /// Get total shard count
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_shard_creation() {
        let weights = Tensor::random(TensorShape::new(vec![10, 10]));
        let shard = ModelShard::new(
            "fc1".to_string(),
            0,
            1,
            weights,
            "node1".to_string(),
            BTreeSet::new(),
        );
        
        assert_eq!(shard.layer_id, "fc1");
        assert_eq!(shard.shard_idx, 0);
        assert!(!shard.content_id.is_empty());
    }

    #[test]
    fn test_shard_registry() {
        let mut registry = ShardRegistry::new("node1".to_string());
        
        let shard = ModelShard::new(
            "fc1".to_string(),
            0,
            1,
            Tensor::random(TensorShape::new(vec![10, 10])),
            "node1".to_string(),
            BTreeSet::new(),
        );
        
        let content_id = shard.content_id.clone();
        registry.register(shard);
        
        assert!(registry.get(&content_id).is_some());
        assert_eq!(registry.shard_count(), 1);
    }

    #[test]
    fn test_execution_order() {
        let mut registry = ShardRegistry::new("node1".to_string());
        
        // Create layers with dependencies: fc1 -> fc2 -> fc3
        let mut deps2 = BTreeSet::new();
        deps2.insert("fc1".to_string());
        
        let mut deps3 = BTreeSet::new();
        deps3.insert("fc2".to_string());
        
        let layers = vec![
            ("fc1".to_string(), Tensor::random(TensorShape::new(vec![4])), BTreeSet::new()),
            ("fc2".to_string(), Tensor::random(TensorShape::new(vec![4])), deps2),
            ("fc3".to_string(), Tensor::random(TensorShape::new(vec![4])), deps3),
        ];
        
        registry.shard_model(layers, &["node1".to_string()]);
        
        let order = registry.execution_order();
        assert_eq!(order, vec!["fc1", "fc2", "fc3"]);
    }
}
