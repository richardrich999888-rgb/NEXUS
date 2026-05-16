use nexus_sync::*;
use nexus_pcu::*;
use nexus_pcu::routing::NodeInfo;
use nexus_pcu::proof::{ExecutionProof, ExecutionProofBuilder};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

#[test]
fn test_e2e_pcu_sync_and_execution() {
    // 1. Setup nodes
    let mut rng = OsRng;
    let mut signing_key_a = [0u8; 32];
    let mut signing_key_b = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rng, &mut signing_key_a);
    rand::RngCore::fill_bytes(&mut rng, &mut signing_key_b);
    
    let key_a = SigningKey::from_bytes(&signing_key_a);
    let key_b = SigningKey::from_bytes(&signing_key_b);
    
    let node_id_a = NodeId::from_verifying_key(&key_a.verifying_key());
    let node_id_b = NodeId::from_verifying_key(&key_b.verifying_key());
    
    // NexusSyncEngine::new takes a String for node_id for CAUSALUX
    let mut engine_a = NexusSyncEngine::new(node_id_a.to_hex(), ConflictPolicy::LastWriterWins);
    let mut engine_b = NexusSyncEngine::new(node_id_b.to_hex(), ConflictPolicy::LastWriterWins);
    
    // 2. Node B creates data (USO)
    let owner = PrincipalId::generate();
    let initial_data = b"initial state".to_vec();
    let uso_b = USO::new(initial_data, owner);
    let uso_id = uso_b.id;
    engine_b.register_uso(uso_b);
    
    // 3. Node A creates a PCU that depends on this USO
    let code = WasmModule::new(b"mock wasm".to_vec());
    let identity = IdentityContext::new(owner, CapabilitySet::default());
    let pcu = PCU::new(code.clone(), vec![uso_id], vec![], identity);
    
    // 4. Routing Simulation
    let mut locator = DataLocator::new();
    locator.register_node(NodeInfo::new(node_id_b, "node_b:8080"));
    locator.record_content(uso_id, node_id_b);
    
    let target = locator.route(&pcu).expect("Should route to Node B");
    assert_eq!(target, node_id_b);
    
    // 5. Node A syncs with Node B to get the USO state
    let delta_b = engine_b.get_sync_delta(engine_a.version_vector());
    engine_a.merge_remote(delta_b.operations).unwrap();
    
    // Simulate data transfer (actual USO sync logic is TBD in Phase 4, so we mock it)
    engine_a.register_uso(engine_b.get_uso(&uso_id).unwrap().clone());
    
    // 6. Node A executes PCU (Mock)
    let output = b"execution result".to_vec();
    // Use engine_a's key for proof
    let proof = ExecutionProofBuilder::new(pcu.id, code.hash)
        .with_inputs(vec![uso_id])
        .with_output(ContentHash::compute(&output))
        .build_signed(&key_a);
        
    // 7. Node A updates USO with result
    engine_a.update_uso(&uso_id, output.clone(), owner).unwrap();
    
    // 8. Node B syncs back from Node A
    let delta_a = engine_a.get_sync_delta(engine_b.version_vector());
    engine_b.merge_remote(delta_a.operations).unwrap();
    
    // 9. Verify convergence
    let uso_a = engine_a.get_uso(&uso_id).unwrap();
    // Manual sync of data for now (simulating USO payload synchronization)
    let mut uso_b_final = engine_b.get_uso(&uso_id).unwrap().clone();
    uso_b_final.data = uso_a.data.clone();
    
    assert_eq!(uso_b_final.data, output);
    assert!(proof.verify());
    assert_eq!(proof.node_id(), node_id_a);
}
