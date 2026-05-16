//! NEXUS Fuzz Tests
//!
//! Property-based fuzzing for core NEXUS operations:
//! - PCU inputs: arbitrary bytecode, parameters, identities
//! - Causal merges: arbitrary tensor data and merge sequences
//! - Content hashing: arbitrary data including edge cases
//!
//! Uses proptest for property-based testing with shrinking.
//!
//! Copyright (c) 2025 SYNTRIASS Labs Private Limited
//! Inventor: Katta Naga Sri Ganesh

use proptest::prelude::*;

// ============================================================================
// PCU FUZZING
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Fuzz: PCU serialization never panics
    #[test]
    fn fuzz_pcu_serialization_no_panic(
        bytecode in prop::collection::vec(any::<u8>(), 0..1000),
        params in prop::collection::vec(any::<u8>(), 0..500),
        input_hashes in prop::collection::vec(any::<[u8; 32]>(), 0..20),
        principal_bytes in any::<[u8; 32]>(),
    ) {
        use nexus_pcu::{PCU, WasmModule, ContentHash};
        use nexus_pcu::identity::{IdentityContext, PrincipalId, CapabilitySet};

        let code = WasmModule::new(bytecode);
        let inputs: Vec<ContentHash> = input_hashes.into_iter()
            .map(ContentHash::from_bytes)
            .collect();
        let principal = PrincipalId::from_bytes(principal_bytes);
        let identity = IdentityContext::new(principal, CapabilitySet::default());

        let pcu = PCU::new(code, inputs, params, identity);
        
        // Must not panic
        let bytes = pcu.to_bytes().expect("serialization must succeed");
        let _ = PCU::from_bytes(&bytes);
    }

    /// Fuzz: PCU ID is always deterministic
    #[test]
    fn fuzz_pcu_id_deterministic(
        bytecode in prop::collection::vec(any::<u8>(), 1..100),
        params in prop::collection::vec(any::<u8>(), 0..100),
        principal_bytes in any::<[u8; 32]>(),
    ) {
        use nexus_pcu::{PCU, WasmModule};
        use nexus_pcu::identity::{IdentityContext, PrincipalId, CapabilitySet};

        let principal = PrincipalId::from_bytes(principal_bytes);
        let identity = IdentityContext::new(principal, CapabilitySet::default());

        let pcu1 = PCU::new(
            WasmModule::new(bytecode.clone()),
            vec![],
            params.clone(),
            identity.clone(),
        );
        let pcu2 = PCU::new(
            WasmModule::new(bytecode),
            vec![],
            params,
            identity,
        );

        prop_assert_eq!(pcu1.id, pcu2.id, "PCU ID must be deterministic");
    }
}

// ============================================================================
// USO FUZZING
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Fuzz: USO never panics on arbitrary data
    #[test]
    fn fuzz_uso_creation_no_panic(
        data in prop::collection::vec(any::<u8>(), 0..10000),
        owner_bytes in any::<[u8; 32]>(),
    ) {
        use nexus_pcu::uso::USO;
        use nexus_pcu::identity::PrincipalId;

        let owner = PrincipalId::from_bytes(owner_bytes);
        let uso = USO::new(data, owner);
        
        // Must not panic
        let bytes = uso.to_bytes().expect("serialization must succeed");
        let _ = USO::from_bytes(&bytes);
    }

    /// Fuzz: USO ID equals content hash
    #[test]
    fn fuzz_uso_id_invariant(
        data in prop::collection::vec(any::<u8>(), 1..1000),
        owner_bytes in any::<[u8; 32]>(),
    ) {
        use nexus_pcu::uso::USO;
        use nexus_pcu::identity::PrincipalId;
        use nexus_pcu::ContentHash;

        let owner = PrincipalId::from_bytes(owner_bytes);
        let uso = USO::new(data.clone(), owner);
        
        let expected_id = ContentHash::compute(&data);
        prop_assert_eq!(uso.id, expected_id, "USO ID must equal content hash");
    }

    /// Fuzz: USO merge is deterministic
    #[test]
    fn fuzz_uso_merge_deterministic(
        data_a in prop::collection::vec(any::<u8>(), 1..500),
        data_b in prop::collection::vec(any::<u8>(), 1..500),
        owner_bytes in any::<[u8; 32]>(),
    ) {
        use nexus_pcu::uso::USO;
        use nexus_pcu::identity::PrincipalId;

        let owner = PrincipalId::from_bytes(owner_bytes);
        let uso_a = USO::new(data_a, owner);
        let uso_b = USO::new(data_b, owner);

        // Same merge twice
        let mut merged1 = uso_a.clone();
        let mut merged2 = uso_a.clone();
        merged1.merge(&uso_b);
        merged2.merge(&uso_b);

        prop_assert_eq!(merged1.data, merged2.data, "USO merge must be deterministic");
    }
}

// ============================================================================
// CONTENT HASH FUZZING
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Fuzz: ContentHash never panics
    #[test]
    fn fuzz_content_hash_no_panic(data in prop::collection::vec(any::<u8>(), 0..100000)) {
        use nexus_pcu::ContentHash;
        
        // Must not panic
        let hash = ContentHash::compute(&data);
        let _ = hash.to_hex();
        let _ = hash.as_bytes();
    }

    /// Fuzz: ContentHash is always deterministic
    #[test]
    fn fuzz_content_hash_deterministic(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        use nexus_pcu::ContentHash;
        
        let h1 = ContentHash::compute(&data);
        let h2 = ContentHash::compute(&data);
        
        prop_assert_eq!(h1, h2, "ContentHash must be deterministic");
    }

    /// Fuzz: ContentHash is collision-resistant
    #[test]
    fn fuzz_content_hash_collision_resistant(
        data1 in prop::collection::vec(any::<u8>(), 1..100),
        data2 in prop::collection::vec(any::<u8>(), 1..100),
    ) {
        use nexus_pcu::ContentHash;
        
        prop_assume!(data1 != data2);
        
        let h1 = ContentHash::compute(&data1);
        let h2 = ContentHash::compute(&data2);
        
        prop_assert_ne!(h1, h2, "Different data should produce different hashes");
    }
}

// ============================================================================
// CAUSAL TENSOR FUZZING
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Fuzz: CausalTensor creation never panics
    #[test]
    fn fuzz_causal_tensor_creation_no_panic(
        data in prop::collection::vec(any::<u8>(), 0..10000),
        node_id in any::<u64>(),
    ) {
        use nexus_core::causal::{CausalTensor, VectorClock};
        use nexus_core::crypto::generate_signing_key;

        let signing_key = generate_signing_key();
        let mut clock = VectorClock::new();

        // Should not panic, but may error for very large data
        let result = CausalTensor::new(
            data,
            vec![],
            node_id,
            &mut clock,
            &signing_key,
        );
        
        // Just check it doesn't panic
        let _ = result;
    }

    /// Fuzz: CausalTensor merge is commutative
    #[test]
    fn fuzz_causal_merge_commutative(
        data_a in prop::collection::vec(any::<u8>(), 1..500),
        data_b in prop::collection::vec(any::<u8>(), 1..500),
    ) {
        use nexus_core::causal::{CausalTensor, VectorClock};
        use nexus_core::crypto::generate_signing_key;

        let signing_key = generate_signing_key();

        let mut clock_a = VectorClock::new();
        let tensor_a = CausalTensor::new(data_a, vec![], 1, &mut clock_a, &signing_key).unwrap();

        let mut clock_b = VectorClock::new();
        let tensor_b = CausalTensor::new(data_b, vec![], 2, &mut clock_b, &signing_key).unwrap();

        let mut clock1 = VectorClock::new();
        let merged_ab = CausalTensor::merge(&tensor_a, &tensor_b, 3, &mut clock1, &signing_key).unwrap();

        let mut clock2 = VectorClock::new();
        let merged_ba = CausalTensor::merge(&tensor_b, &tensor_a, 3, &mut clock2, &signing_key).unwrap();

        prop_assert_eq!(merged_ab.data, merged_ba.data, "Merge must be commutative");
    }

    /// Fuzz: CausalTensor merge is idempotent
    #[test]
    fn fuzz_causal_merge_idempotent(data in prop::collection::vec(any::<u8>(), 1..500)) {
        use nexus_core::causal::{CausalTensor, VectorClock};
        use nexus_core::crypto::generate_signing_key;

        let signing_key = generate_signing_key();
        let mut clock = VectorClock::new();
        
        let tensor = CausalTensor::new(data, vec![], 1, &mut clock, &signing_key).unwrap();
        let merged = CausalTensor::merge(&tensor, &tensor, 1, &mut clock.clone(), &signing_key).unwrap();

        prop_assert_eq!(tensor.id, merged.id, "Merge with self must be idempotent");
    }
}

// ============================================================================
// CRYPTOGRAPHY FUZZING
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Fuzz: Signature verification is consistent
    #[test]
    fn fuzz_signature_verification(message in prop::collection::vec(any::<u8>(), 0..10000)) {
        use nexus_pcu::crypto::{generate_signing_key, derive_verifying_key, sign, verify};

        let key = generate_signing_key();
        let verifying_key = derive_verifying_key(&key);
        let signature = sign(&key, &message);

        // Should always verify
        prop_assert!(verify(&verifying_key, &message, &signature).is_ok());
    }

    /// Fuzz: Wrong message fails verification
    #[test]
    fn fuzz_signature_wrong_message(
        message1 in prop::collection::vec(any::<u8>(), 1..1000),
        message2 in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        use nexus_pcu::crypto::{generate_signing_key, derive_verifying_key, sign, verify};

        prop_assume!(message1 != message2);

        let key = generate_signing_key();
        let verifying_key = derive_verifying_key(&key);
        let signature = sign(&key, &message1);

        // Should fail for wrong message
        prop_assert!(verify(&verifying_key, &message2, &signature).is_err());
    }
}
