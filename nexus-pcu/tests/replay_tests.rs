//! Deterministic Replay Tests
//!
//! These tests verify that NEXUS operations produce identical results
//! when replayed with the same inputs. This is critical for:
//! - Audit trails
//! - Distributed consensus
//! - Byzantine fault tolerance
//!
//! Copyright (c) 2025 SYNTRIASS Labs Private Limited
//! Inventor: Katta Naga Sri Ganesh

use proptest::prelude::*;

/// Replay test for CausalTensor merge operations
#[cfg(test)]
mod causal_tensor_replay {
    use nexus_core::causal::{CausalTensor, VectorClock, CausalId};
    use nexus_core::crypto::generate_signing_key;

    /// Test: Merge operation is deterministic when replayed
    #[test]
    fn test_merge_replay_deterministic() {
        let signing_key = generate_signing_key();
        
        // Create two tensors
        let mut clock_a = VectorClock::new();
        let tensor_a = CausalTensor::new(
            b"data_a".to_vec(),
            vec![],
            1,
            &mut clock_a,
            &signing_key,
        ).unwrap();

        let mut clock_b = VectorClock::new();
        let tensor_b = CausalTensor::new(
            b"data_b".to_vec(),
            vec![],
            2,
            &mut clock_b,
            &signing_key,
        ).unwrap();

        // Merge twice - should produce identical results
        let mut clock1 = VectorClock::new();
        let merged1 = CausalTensor::merge(
            &tensor_a,
            &tensor_b,
            3,
            &mut clock1,
            &signing_key,
        ).unwrap();

        let mut clock2 = VectorClock::new();
        let merged2 = CausalTensor::merge(
            &tensor_a,
            &tensor_b,
            3,
            &mut clock2,
            &signing_key,
        ).unwrap();

        // Verify determinism
        assert_eq!(merged1.data, merged2.data, "Merge data must be deterministic");
        assert_eq!(merged1.provenance.parents, merged2.provenance.parents, "Parents must be deterministic");
    }

    /// Test: Merge commutativity (A+B == B+A in terms of data)
    #[test]
    fn test_merge_commutative() {
        let signing_key = generate_signing_key();
        
        let mut clock_a = VectorClock::new();
        let tensor_a = CausalTensor::new(
            b"AAA".to_vec(),
            vec![],
            1,
            &mut clock_a,
            &signing_key,
        ).unwrap();

        let mut clock_b = VectorClock::new();
        let tensor_b = CausalTensor::new(
            b"BBB".to_vec(),
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

        // Data should be identical regardless of merge order
        assert_eq!(merged_ab.data, merged_ba.data, "Merge must be commutative");
    }

    /// Test: Merge idempotence (A+A == A)
    #[test]
    fn test_merge_idempotent() {
        let signing_key = generate_signing_key();
        
        let mut clock = VectorClock::new();
        let tensor = CausalTensor::new(
            b"data".to_vec(),
            vec![],
            1,
            &mut clock,
            &signing_key,
        ).unwrap();

        let merged = CausalTensor::merge(
            &tensor,
            &tensor,
            1,
            &mut clock,
            &signing_key,
        ).unwrap();

        assert_eq!(tensor.id, merged.id, "Merge with self must be idempotent");
    }
}

/// Replay test for PCU operations
#[cfg(test)]
mod pcu_replay {
    use nexus_pcu::{PCU, WasmModule, ContentHash};
    use nexus_pcu::identity::{IdentityContext, PrincipalId, CapabilitySet};

    /// Test: PCU ID computation is deterministic across replays
    #[test]
    fn test_pcu_id_replay_deterministic() {
        let bytecode = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let params = vec![1, 2, 3, 4, 5];
        let principal = PrincipalId::from_bytes([42u8; 32]);
        let inputs = vec![
            ContentHash::compute(b"input1"),
            ContentHash::compute(b"input2"),
        ];

        // Create PCU multiple times with same inputs
        let pcu1 = PCU::new(
            WasmModule::new(bytecode.clone()),
            inputs.clone(),
            params.clone(),
            IdentityContext::new(principal, CapabilitySet::default()),
        );

        let pcu2 = PCU::new(
            WasmModule::new(bytecode.clone()),
            inputs.clone(),
            params.clone(),
            IdentityContext::new(principal, CapabilitySet::default()),
        );

        let pcu3 = PCU::new(
            WasmModule::new(bytecode),
            inputs,
            params,
            IdentityContext::new(principal, CapabilitySet::default()),
        );

        // All PCUs must have identical IDs
        assert_eq!(pcu1.id, pcu2.id, "PCU ID must be deterministic (1==2)");
        assert_eq!(pcu2.id, pcu3.id, "PCU ID must be deterministic (2==3)");
    }

    /// Test: ContentHash is deterministic
    #[test]
    fn test_content_hash_replay_deterministic() {
        let data = b"determinism test data";
        
        let hash1 = ContentHash::compute(data);
        let hash2 = ContentHash::compute(data);
        let hash3 = ContentHash::compute(data);

        assert_eq!(hash1, hash2, "ContentHash must be deterministic (1==2)");
        assert_eq!(hash2, hash3, "ContentHash must be deterministic (2==3)");
    }
}
