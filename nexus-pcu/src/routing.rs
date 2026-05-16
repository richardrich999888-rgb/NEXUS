// NEXUS Routing: Code-to-Data Routing
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Key innovation: PCU routes TO where data lives, not the other way around.
// This eliminates data transfer costs and enables true edge computing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{NodeId, pcu::PCU, content_hash::ContentHash};

// ============================================================================
// NODE INFO - Metadata about a node
// ============================================================================

/// Information about a node in the mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node identifier
    pub id: NodeId,
    
    /// Node address (e.g., "192.168.1.1:8080")
    pub address: String,
    
    /// Region this node is in
    pub region: String,
    
    /// Capabilities this node has (e.g., "gpu", "arm64")
    pub capabilities: Vec<String>,
    
    /// Current load (0.0 - 1.0)
    pub load: f32,
    
    /// Network latency to this node in ms (from requester's perspective)
    pub latency_ms: u32,
    
    /// Available memory in bytes
    pub available_memory: u64,
}

impl NodeInfo {
    pub fn new(id: NodeId, address: impl Into<String>) -> Self {
        NodeInfo {
            id,
            address: address.into(),
            region: "local".to_string(),
            capabilities: Vec::new(),
            load: 0.0,
            latency_ms: 0,
            available_memory: 1024 * 1024 * 1024, // 1GB default
        }
    }

    /// Check if node has all required capabilities
    pub fn has_capabilities(&self, required: &[String]) -> bool {
        required.iter().all(|cap| self.capabilities.contains(cap))
    }

    /// Check if node has capacity for PCU
    pub fn has_capacity(&self, pcu: &PCU) -> bool {
        self.load < 0.9 && 
        self.available_memory > pcu.constraints.max_memory_bytes
    }
}

// ============================================================================
// DATA LOCATOR - Tracks where content lives
// ============================================================================

/// Tracks which nodes have which content
#[derive(Debug, Clone, Default)]
pub struct DataLocator {
    /// Map from content hash to nodes that have it
    content_map: HashMap<ContentHash, Vec<NodeId>>,
    
    /// Map from node ID to node info
    node_info: HashMap<NodeId, NodeInfo>,
}

impl DataLocator {
    /// Create empty locator
    pub fn new() -> Self {
        DataLocator {
            content_map: HashMap::new(),
            node_info: HashMap::new(),
        }
    }

    /// Register a node
    pub fn register_node(&mut self, info: NodeInfo) {
        self.node_info.insert(info.id, info);
    }

    /// Record that a node has some content
    pub fn record_content(&mut self, hash: ContentHash, node: NodeId) {
        self.content_map
            .entry(hash)
            .or_insert_with(Vec::new)
            .push(node);
    }

    /// Find nodes that have a piece of content
    pub fn locate(&self, hash: &ContentHash) -> Vec<NodeId> {
        self.content_map.get(hash).cloned().unwrap_or_default()
    }

    /// Find best node for PCU execution
    /// 
    /// Strategy:
    /// 1. Find nodes that have ALL required inputs
    /// 2. Filter by required capabilities
    /// 3. Filter by capacity
    /// 4. Sort by load * latency (prefer low-load, low-latency)
    pub fn route(&self, pcu: &PCU) -> Option<NodeId> {
        // If no inputs, run on any available node
        if pcu.inputs.is_empty() {
            return self.node_info.values()
                .filter(|n| n.has_capabilities(&pcu.constraints.required_capabilities))
                .filter(|n| n.has_capacity(pcu))
                .min_by(|a, b| {
                    let score_a = (a.load * 100.0) as u32 + a.latency_ms;
                    let score_b = (b.load * 100.0) as u32 + b.latency_ms;
                    score_a.cmp(&score_b)
                })
                .map(|n| n.id);
        }

        // Find nodes that have ALL inputs
        let mut candidates: Vec<NodeId> = Vec::new();
        
        for (i, input) in pcu.inputs.iter().enumerate() {
            let nodes = self.locate(input);
            if i == 0 {
                candidates = nodes;
            } else {
                candidates.retain(|n| nodes.contains(n));
            }
        }

        // Filter by requirements and sort
        candidates.iter()
            .filter_map(|id| self.node_info.get(id))
            .filter(|n| n.has_capabilities(&pcu.constraints.required_capabilities))
            .filter(|n| n.has_capacity(pcu))
            .min_by(|a, b| {
                let score_a = (a.load * 100.0) as u32 + a.latency_ms;
                let score_b = (b.load * 100.0) as u32 + b.latency_ms;
                score_a.cmp(&score_b)
            })
            .map(|n| n.id)
    }

    /// Find the node with most inputs for a PCU (partial execution)
    pub fn route_best_effort(&self, pcu: &PCU) -> (NodeId, Vec<ContentHash>) {
        if pcu.inputs.is_empty() {
            // Run locally if no inputs needed
            return (NodeId::local(), Vec::new());
        }

        // Count inputs per node
        let mut node_inputs: HashMap<NodeId, Vec<ContentHash>> = HashMap::new();
        
        for input in &pcu.inputs {
            for node in self.locate(input) {
                node_inputs.entry(node)
                    .or_insert_with(Vec::new)
                    .push(*input);
            }
        }

        // Find node with most inputs
        node_inputs.into_iter()
            .max_by_key(|(_, inputs)| inputs.len())
            .unwrap_or((NodeId::local(), Vec::new()))
    }

    /// Get number of registered nodes
    pub fn node_count(&self) -> usize {
        self.node_info.len()
    }

    /// Get total content items tracked
    pub fn content_count(&self) -> usize {
        self.content_map.len()
    }
}

// ============================================================================
// ROUTING DECISION - Result of routing
// ============================================================================

/// Result of routing a PCU
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Target node for execution
    pub target: NodeId,
    
    /// Inputs available at target
    pub available_inputs: Vec<ContentHash>,
    
    /// Inputs that need to be transferred
    pub missing_inputs: Vec<ContentHash>,
    
    /// Estimated network cost (bytes to transfer)
    pub transfer_cost: u64,
}

impl RoutingDecision {
    /// Check if all inputs are local (no transfer needed)
    pub fn is_local(&self) -> bool {
        self.missing_inputs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{IdentityContext, PrincipalId, CapabilitySet};
    use crate::pcu::WasmModule;

    fn create_test_pcu(inputs: Vec<ContentHash>) -> PCU {
        let code = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d]);
        let identity = IdentityContext::new(PrincipalId::generate(), CapabilitySet::default());
        PCU::new(code, inputs, vec![], identity)
    }

    #[test]
    fn test_route_no_inputs() {
        let mut locator = DataLocator::new();
        
        let node1 = NodeInfo::new(NodeId::new([1; 32]), "127.0.0.1:8080");
        locator.register_node(node1);
        
        let pcu = create_test_pcu(vec![]);
        let target = locator.route(&pcu);
        
        assert_eq!(target, Some(NodeId::new([1; 32])));
    }

    #[test]
    fn test_route_to_data() {
        let mut locator = DataLocator::new();
        
        let node1 = NodeInfo::new(NodeId::new([1; 32]), "node1:8080");
        let node2 = NodeInfo::new(NodeId::new([2; 32]), "node2:8080");
        locator.register_node(node1);
        locator.register_node(node2);
        
        let input1 = ContentHash::compute(b"data1");
        let input2 = ContentHash::compute(b"data2");
        
        // Node 2 has both inputs
        locator.record_content(input1, NodeId::new([2; 32]));
        locator.record_content(input2, NodeId::new([2; 32]));
        
        // Node 1 only has input1
        locator.record_content(input1, NodeId::new([1; 32]));
        
        let pcu = create_test_pcu(vec![input1, input2]);
        let target = locator.route(&pcu);
        
        // Should route to node 2 (has all inputs)
        assert_eq!(target, Some(NodeId::new([2; 32])));
    }

    #[test]
    fn test_route_best_effort() {
        let mut locator = DataLocator::new();
        
        let node1 = NodeInfo::new(NodeId::new([1; 32]), "node1");
        let node2 = NodeInfo::new(NodeId::new([2; 32]), "node2");
        locator.register_node(node1);
        locator.register_node(node2);
        
        let input1 = ContentHash::compute(b"data1");
        let input2 = ContentHash::compute(b"data2");
        let input3 = ContentHash::compute(b"data3");
        
        // Node 1 has 2 inputs, node 2 has 1
        locator.record_content(input1, NodeId::new([1; 32]));
        locator.record_content(input2, NodeId::new([1; 32]));
        locator.record_content(input3, NodeId::new([2; 32]));
        
        let pcu = create_test_pcu(vec![input1, input2, input3]);
        let (target, available) = locator.route_best_effort(&pcu);
        
        // Should prefer node 1 (has more inputs)
        assert_eq!(target, NodeId::new([1; 32]));
        assert_eq!(available.len(), 2);
    }

    #[test]
    fn test_node_capacity_check() {
        let mut node = NodeInfo::new(NodeId::new([1; 32]), "node1");
        node.load = 0.5;
        node.available_memory = 512 * 1024 * 1024; // 512MB
        
        let mut pcu = create_test_pcu(vec![]);
        pcu.constraints.max_memory_bytes = 256 * 1024 * 1024; // 256MB
        
        assert!(node.has_capacity(&pcu));
        
        pcu.constraints.max_memory_bytes = 1024 * 1024 * 1024; // 1GB
        assert!(!node.has_capacity(&pcu));
    }
}
