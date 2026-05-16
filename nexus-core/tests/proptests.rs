// NEXUS Core: Property-based Tests
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use nexus_core::causal::{CausalTensor, VectorClock};
use nexus_core::crypto::generate_signing_key;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_merge_idempotent(data in prop::collection::vec(0..255u8, 0..1024)) {
        let signing_key = generate_signing_key();
        let mut clock = VectorClock::new();
        
        let tensor = CausalTensor::new(
            data,
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
        
        prop_assert_eq!(tensor.id, merged.id);
    }

    #[test]
    fn prop_merge_commutative(
        data1 in prop::collection::vec(0..255u8, 0..1024),
        data2 in prop::collection::vec(0..255u8, 0..1024)
    ) {
        let signing_key = generate_signing_key();
        
        let mut clock1 = VectorClock::new();
        let tensor1 = CausalTensor::new(
            data1,
            vec![],
            1,
            &mut clock1,
            &signing_key,
        ).unwrap();
        
        let mut clock2 = VectorClock::new();
        let tensor2 = CausalTensor::new(
            data2,
            vec![],
            2,
            &mut clock2,
            &signing_key,
        ).unwrap();
        
        // Merge order 1
        let mut clock_m1 = VectorClock::new();
        let merged1 = CausalTensor::merge(&tensor1, &tensor2, 3, &mut clock_m1, &signing_key).unwrap();
        
        // Merge order 2
        let mut clock_m2 = VectorClock::new();
        let merged2 = CausalTensor::merge(&tensor2, &tensor1, 3, &mut clock_m2, &signing_key).unwrap();
        
        prop_assert_eq!(merged1.data, merged2.data);
        prop_assert_eq!(merged1.provenance.parents, merged2.provenance.parents);
    }
}
