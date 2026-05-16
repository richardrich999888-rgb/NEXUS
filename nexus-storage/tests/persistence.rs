// NEXUS Storage: Persistence Tests
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use nexus_storage::{ProvenanceLog, AlgebraicIndex};
use tempfile::tempdir;

#[test]
fn test_provenance_log_persistence() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    
    let signing_key = generate_signing_key();
    let mut clock = VectorClock::new();
    
    let tensor = CausalTensor::new(
        b"Persistence test data".to_vec(),
        vec![],
        1,
        &mut clock,
        &signing_key,
    ).unwrap();
    
    let tensor_id = tensor.id;
    
    // 1. Open and write
    {
        let log = ProvenanceLog::open(path).expect("Failed to open log");
        log.append(&tensor).expect("Failed to append tensor");
        assert!(log.exists(&tensor_id).unwrap());
    }
    
    // 2. Re-open and read (persistence verify)
    {
        let log = ProvenanceLog::open(path).expect("Failed to re-open log");
        let restored = log.get(&tensor_id).expect("Failed to get tensor").unwrap();
        assert_eq!(restored.id, tensor_id);
        assert_eq!(restored.data, b"Persistence test data");
    }
}

#[test]
fn test_algebraic_index_queries() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    
    let index = AlgebraicIndex::open(path).expect("Failed to open index");
    let signing_key = generate_signing_key();
    
    // Create tensors for different nodes
    for node_id in 1..=3 {
        let mut clock = VectorClock::new();
        let tensor = CausalTensor::new(
            format!("Data from node {}", node_id).into_bytes(),
            vec![],
            node_id,
            &mut clock,
            &signing_key,
        ).unwrap();
        index.index_tensor(&tensor).expect("Failed to index");
    }
    
    // Query by node
    let node_1_tensors = index.get_by_node(1).unwrap();
    assert_eq!(node_1_tensors.len(), 1);
    
    let node_2_tensors = index.get_by_node(2).unwrap();
    assert_eq!(node_2_tensors.len(), 1);
    
    // Query by depth (all should be at depth 0 since they have no parents)
    let depth_0_tensors = index.get_by_depth(0).unwrap();
    assert_eq!(depth_0_tensors.len(), 3);
}
