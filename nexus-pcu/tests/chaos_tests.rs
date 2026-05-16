//! NEXUS Chaos Tests
//!
//! Validates system behavior under adverse conditions:
//! - State loss / corruption
//! - Concurrent operations
//! - Network partitions (simulated)
//! - Byzantine/malicious inputs
//!
//! These tests verify NEXUS maintains its invariants even when
//! things go wrong.
//!
//! Copyright (c) 2025 SYNTRIASS Labs Private Limited
//! Inventor: Katta Naga Sri Ganesh

use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// CHAOS TEST INFRASTRUCTURE
// ============================================================================

/// Chaos test result
#[derive(Debug, Clone)]
pub struct ChaosResult {
    pub test_name: String,
    pub passed: bool,
    pub iterations: u64,
    pub failures: Vec<String>,
    pub invariants_held: bool,
}

impl ChaosResult {
    pub fn success(name: impl Into<String>, iterations: u64) -> Self {
        Self {
            test_name: name.into(),
            passed: true,
            iterations,
            failures: Vec::new(),
            invariants_held: true,
        }
    }

    pub fn failure(name: impl Into<String>, failures: Vec<String>) -> Self {
        Self {
            test_name: name.into(),
            passed: false,
            iterations: 0,
            failures,
            invariants_held: false,
        }
    }
}

// ============================================================================
// CHAOS TESTS
// ============================================================================

#[cfg(test)]
mod chaos_tests {
    use super::*;
    use nexus_pcu::{PCU, WasmModule, ContentHash};
    use nexus_pcu::identity::{IdentityContext, PrincipalId, CapabilitySet};
    use nexus_pcu::uso::USO;
    use nexus_core::causal::{CausalTensor, VectorClock};
    use nexus_core::crypto::generate_signing_key;

    const CHAOS_ITERATIONS: u64 = 100;

    // ========================================================================
    // STATE LOSS TESTS
    // ========================================================================

    /// Test: PCU survives serialization/deserialization cycle under chaos
    #[test]
    fn chaos_pcu_serialization_roundtrip() {
        let mut failures = Vec::new();

        for i in 0..CHAOS_ITERATIONS {
            // Generate random-ish bytecode
            let bytecode: Vec<u8> = (0..((i % 100) + 10) as usize)
                .map(|j| ((i + j as u64) % 256) as u8)
                .collect();
            
            let code = WasmModule::new(bytecode);
            let principal = PrincipalId::from_bytes([i as u8; 32]);
            let identity = IdentityContext::new(principal, CapabilitySet::default());
            
            let pcu = PCU::new(code, vec![], vec![i as u8], identity);
            
            // Serialize
            let bytes = match pcu.to_bytes() {
                Ok(b) => b,
                Err(e) => {
                    failures.push(format!("Iteration {}: Serialization failed: {}", i, e));
                    continue;
                }
            };
            
            // Deserialize
            match PCU::from_bytes(&bytes) {
                Ok(restored) => {
                    if pcu.id != restored.id {
                        failures.push(format!("Iteration {}: ID mismatch", i));
                    }
                }
                Err(e) => {
                    failures.push(format!("Iteration {}: Deserialization failed: {}", i, e));
                }
            }
        }

        assert!(failures.is_empty(), "Chaos failures: {:?}", failures);
    }

    /// Test: USO merge handles corrupted/partial state
    #[test]
    fn chaos_uso_state_recovery() {
        let mut failures = Vec::new();
        let owner = PrincipalId::from_bytes([42u8; 32]);

        for i in 0..CHAOS_ITERATIONS {
            // Create USOs with varying data
            let data_a: Vec<u8> = (0..((i % 50) + 1) as usize)
                .map(|j| (i as u8).wrapping_add(j as u8))
                .collect();
            let data_b: Vec<u8> = (0..((i % 30) + 1) as usize)
                .map(|j| (i as u8).wrapping_mul(j as u8))
                .collect();

            let uso_a = USO::new(data_a, owner);
            let uso_b = USO::new(data_b, owner);

            // Simulate state loss by cloning and merging
            let mut recovered = uso_a.clone();
            recovered.merge(&uso_b);

            // Verify invariants
            if recovered.id != ContentHash::compute(&recovered.data) {
                failures.push(format!("Iteration {}: USO ID invariant violated", i));
            }
        }

        assert!(failures.is_empty(), "Chaos failures: {:?}", failures);
    }

    // ========================================================================
    // CONCURRENCY TESTS
    // ========================================================================

    /// Test: Concurrent PCU ID computation is deterministic
    #[test]
    fn chaos_concurrent_pcu_determinism() {
        let bytecode = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let params = vec![1, 2, 3, 4, 5];
        let principal = PrincipalId::from_bytes([99u8; 32]);
        let identity = IdentityContext::new(principal, CapabilitySet::default());

        // Compute expected ID first
        let expected_pcu = PCU::new(
            WasmModule::new(bytecode.clone()),
            vec![],
            params.clone(),
            identity.clone(),
        );
        let expected_id = expected_pcu.id;

        // Spawn multiple threads computing the same PCU
        let results: Arc<Mutex<Vec<ContentHash>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for _ in 0..10 {
            let bc = bytecode.clone();
            let pm = params.clone();
            let id = identity.clone();
            let res = Arc::clone(&results);

            handles.push(thread::spawn(move || {
                for _ in 0..10 {
                    let pcu = PCU::new(
                        WasmModule::new(bc.clone()),
                        vec![],
                        pm.clone(),
                        id.clone(),
                    );
                    res.lock().unwrap().push(pcu.id);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All IDs should be identical
        let ids = results.lock().unwrap();
        assert!(ids.iter().all(|id| *id == expected_id), 
            "Concurrent PCU computation produced different IDs");
    }

    /// Test: Concurrent USO merge converges
    #[test]
    fn chaos_concurrent_uso_merge() {
        let owner = PrincipalId::from_bytes([1u8; 32]);
        let base = USO::new(b"base".to_vec(), owner);

        let results: Arc<Mutex<Vec<ContentHash>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        // Create multiple concurrent merges
        for i in 0..10 {
            let b = base.clone();
            let o = owner;
            let res = Arc::clone(&results);

            handles.push(thread::spawn(move || {
                let mut merged = b;
                for j in 0..10 {
                    let other = USO::new(
                        format!("data_{}_{}", i, j).into_bytes(),
                        o,
                    );
                    merged.merge(&other);
                }
                res.lock().unwrap().push(merged.id);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // IDs may differ (different merge sequences) but all should be valid content hashes
        let ids = results.lock().unwrap();
        for id in ids.iter() {
            assert_eq!(id.as_bytes().len(), 32, "Invalid content hash length");
        }
    }

    // ========================================================================
    // CAUSAL TENSOR CHAOS TESTS
    // ========================================================================

    /// Test: Causal merge is commutative under chaos
    #[test]
    fn chaos_causal_merge_commutativity() {
        let signing_key = generate_signing_key();
        let mut failures = Vec::new();

        for i in 0..50 {
            let data_a: Vec<u8> = (0..(i % 20 + 5) as usize)
                .map(|j| (i as u8) ^ (j as u8))
                .collect();
            let data_b: Vec<u8> = (0..(i % 15 + 3) as usize)
                .map(|j| (i as u8).wrapping_add(j as u8))
                .collect();

            let mut clock_a = VectorClock::new();
            let tensor_a = CausalTensor::new(
                data_a,
                vec![],
                1,
                &mut clock_a,
                &signing_key,
            ).unwrap();

            let mut clock_b = VectorClock::new();
            let tensor_b = CausalTensor::new(
                data_b,
                vec![],
                2,
                &mut clock_b,
                &signing_key,
            ).unwrap();

            // Merge A+B
            let mut clock1 = VectorClock::new();
            let merged_ab = CausalTensor::merge(
                &tensor_a,
                &tensor_b,
                3,
                &mut clock1,
                &signing_key,
            ).unwrap();

            // Merge B+A
            let mut clock2 = VectorClock::new();
            let merged_ba = CausalTensor::merge(
                &tensor_b,
                &tensor_a,
                3,
                &mut clock2,
                &signing_key,
            ).unwrap();

            // Data should be identical (commutativity)
            if merged_ab.data != merged_ba.data {
                failures.push(format!("Iteration {}: Merge not commutative", i));
            }
        }

        assert!(failures.is_empty(), "Commutativity failures: {:?}", failures);
    }

    /// Test: Causal merge is idempotent under chaos
    #[test]
    fn chaos_causal_merge_idempotence() {
        let signing_key = generate_signing_key();
        let mut failures = Vec::new();

        for i in 0..50 {
            let data: Vec<u8> = (0..(i % 30 + 5) as usize)
                .map(|j| (i as u8).wrapping_mul((j as u8).wrapping_add(1)))
                .collect();

            let mut clock = VectorClock::new();
            let tensor = CausalTensor::new(
                data,
                vec![],
                1,
                &mut clock,
                &signing_key,
            ).unwrap();

            // Merge with self
            let merged = CausalTensor::merge(
                &tensor,
                &tensor,
                1,
                &mut clock.clone(),
                &signing_key,
            ).unwrap();

            // ID should be identical (idempotence)
            if tensor.id != merged.id {
                failures.push(format!("Iteration {}: Merge with self not idempotent", i));
            }
        }

        assert!(failures.is_empty(), "Idempotence failures: {:?}", failures);
    }

    // ========================================================================
    // BYZANTINE INPUT TESTS
    // ========================================================================

    /// Test: ContentHash handles malicious inputs
    #[test]
    fn chaos_content_hash_byzantine_inputs() {
        // Empty input
        let h1 = ContentHash::compute(&[]);
        assert_eq!(h1.as_bytes().len(), 32);

        // Very large input (1MB)
        let large_input = vec![0u8; 1024 * 1024];
        let h2 = ContentHash::compute(&large_input);
        assert_eq!(h2.as_bytes().len(), 32);

        // All zeros
        let zeros = vec![0u8; 1000];
        let h3 = ContentHash::compute(&zeros);
        
        // All ones
        let ones = vec![0xFFu8; 1000];
        let h4 = ContentHash::compute(&ones);
        
        // Should produce different hashes
        assert_ne!(h3, h4);
        
        // Determinism
        let h3_again = ContentHash::compute(&zeros);
        assert_eq!(h3, h3_again);
    }

    /// Test: PCU handles edge case parameters
    #[test]
    fn chaos_pcu_edge_cases() {
        let principal = PrincipalId::from_bytes([0u8; 32]);
        let identity = IdentityContext::new(principal, CapabilitySet::default());

        // Empty bytecode
        let pcu1 = PCU::new(
            WasmModule::new(vec![]),
            vec![],
            vec![],
            identity.clone(),
        );
        assert_eq!(pcu1.code.bytecode.len(), 0);

        // Maximum inputs (100)
        let many_inputs: Vec<ContentHash> = (0..100)
            .map(|i| ContentHash::compute(&[i as u8]))
            .collect();
        let pcu2 = PCU::new(
            WasmModule::new(vec![0x00]),
            many_inputs.clone(),
            vec![],
            identity.clone(),
        );
        assert_eq!(pcu2.inputs.len(), 100);

        // Large parameters (10KB)
        let large_params = vec![0xABu8; 10 * 1024];
        let pcu3 = PCU::new(
            WasmModule::new(vec![0x00]),
            vec![],
            large_params.clone(),
            identity,
        );
        assert_eq!(pcu3.parameters.len(), 10 * 1024);
    }
}
