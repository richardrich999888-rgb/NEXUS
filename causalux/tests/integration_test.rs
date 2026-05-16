use causalux_v2::*;
use causalux_v2::bft::*;
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;
use std::collections::BTreeSet;
use std::time::Duration;

fn create_keypair() -> SigningKey {
    let mut bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
    SigningKey::from_bytes(&bytes)
}

#[test]
fn test_e2e_collaborative_editing() {
    // 1. Setup two nodes (Alice and Bob)
    let alice_kp = create_keypair();
    let bob_kp = create_keypair();
    
    let alice_id = "alice_node".to_string();
    let bob_id = "bob_node".to_string();
    
    // 2. Initialize CRDT documents for both
    let mut alice_doc = CRDTDocument::new("doc1".to_string(), alice_id.clone());
    let mut bob_doc = CRDTDocument::new("doc1".to_string(), bob_id.clone());
    
    // 3. Alice writes "Hello" (Offline)
    alice_doc.insert_text(0, "Hello");
    
    // 4. Bob writes "World" (Offline, concurrent)
    bob_doc.insert_text(0, "World");
    
    // 5. Wrap CRDT operations into CausalOps for transport/validation
    let mut alice_vv = VersionVector::new();
    alice_vv.increment(&alice_id);
    
    let alice_op = CausalOp::new(
        "sync_state".to_string(),
        alice_doc.to_json(),
        BTreeSet::new(),
        alice_vv.clone(),
        alice_id.clone(),
        &alice_kp,
    );
    
    let mut bob_vv = VersionVector::new();
    bob_vv.increment(&bob_id);
    
    let bob_op = CausalOp::new(
        "sync_state".to_string(),
        bob_doc.to_json(),
        BTreeSet::new(),
        bob_vv.clone(),
        bob_id.clone(),
        &bob_kp,
    );
    
    // 6. Setup BFT Validator
    let validators = vec![
        ValidatorInfo {
            id: CausalOp::derive_identity(&alice_kp.verifying_key()),
            public_key: alice_kp.verifying_key().clone(),
            priority: 1,
            reputation: 1.0,
        },
        ValidatorInfo {
            id: CausalOp::derive_identity(&bob_kp.verifying_key()),
            public_key: bob_kp.verifying_key().clone(),
            priority: 1,
            reputation: 1.0,
        },
    ];
    
    let mut bft = BFTValidator::new(validators, 0, Duration::from_secs(10)).unwrap();
    
    // Alice submits her op
    bft.submit_for_validation(alice_op.clone()).expect("Alice submit failed");
    
    // Alice signs her own op (she is a validator)
    bft.set_validator_keypair(alice_kp.clone());
    let alice_sig = bft.sign_operation(&alice_op.id).unwrap();
    let res = bft.add_validator_signature(alice_sig).unwrap();
    
    // Should be validated with 1 signature since f=0 implies quorum size 1
    match res {
        ValidationResult::Validated(_) => println!("Alice op validated!"),
        _ => panic!("Alice op should be validated"),
    }
    
    // 7. Sync Protocol (Alice -> Bob)
    let mut alice_sync = HierarchicalSync::new(100, true);
    let mut bob_sync = HierarchicalSync::new(100, true);
    
    alice_sync.add_operations(vec![alice_op.clone()]);
    
    let bob_req = bob_sync.prepare_sync_request(
        bob_id.clone(),
        bob_vv,
        "initial_root".to_string(),
    );
    
    let alice_resp = alice_sync.handle_sync_request(bob_req);
    let _stats = bob_sync.apply_sync_response(alice_resp, None).expect("Sync failed");
    
    // 8. Bob applies Alice's op (Mock merge)
    println!("Bob synced op from Alice: {}", alice_op.id);
    
    // 9. Manual CRDT Merge verification
    alice_doc.merge(&bob_doc);
    bob_doc.merge(&alice_doc);
    
    assert_eq!(alice_doc.content.to_string(), bob_doc.content.to_string());
    println!("Converged Text: {}", alice_doc.content.to_string());
}
