//! ETK integration: genesis -> append -> proof -> verify.

use ed25519_dalek::{SigningKey, VerifyingKey};
use etk_core::{
    create_genesis, decode_event, decode_proof, encode_event, encode_proof, hash256, verify,
    EventChain, Verdict,
};
use etk_types::{Hash256, OutcomeCode, ResourceClass};
use rand::rngs::OsRng;

#[test]
fn chain_valid() {
    let actor = hash256(b"actor");
    let workload = hash256(b"workload");
    let ctx = hash256(b"context");
    let policy = hash256(b"policy-snapshot-bytes");

    let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    chain.append(now_ms + 1, OutcomeCode::Unknown).unwrap();
    chain.append(now_ms + 2, OutcomeCode::Success).unwrap();

    let signing_key = SigningKey::generate(&mut OsRng);
    let proof = chain.finalize(&signing_key).unwrap();
    let verifier_pubkey: VerifyingKey = signing_key.verifying_key();

    let events: Vec<_> = chain.events().to_vec();
    let policy_resolver = |_pr: Hash256| Some(b"policy-snapshot-bytes".to_vec());
    let result = verify(&proof, &events, &policy_resolver, &verifier_pubkey, 86400_000);
    assert!(matches!(result, Ok(Verdict::Valid)));
}

#[test]
fn codec_roundtrip_event_and_proof() {
    let actor = hash256(b"a");
    let workload = hash256(b"w");
    let ctx = hash256(b"c");
    let policy = hash256(b"p");
    let genesis = create_genesis(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
    let bytes = encode_event(&genesis);
    let decoded = decode_event(&bytes).unwrap();
    assert_eq!(decoded.event_id, genesis.event_id);
    assert_eq!(decoded.sequence_number, genesis.sequence_number);

    let signing_key = SigningKey::generate(&mut OsRng);
    let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
    chain.append(1000, OutcomeCode::Success).unwrap();
    let proof = chain.finalize(&signing_key).unwrap();
    let proof_bytes = encode_proof(&proof);
    let proof_decoded = decode_proof(&proof_bytes).unwrap();
    assert_eq!(proof_decoded.execution_id, proof.execution_id);
    assert_eq!(proof_decoded.event_chain_root, proof.event_chain_root);
}
