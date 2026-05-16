// Property-based tests for CRDT merge invariants in nexus-sync
// Copyright (c) 2025 SYNTRIASS Labs Private Limited

use proptest::prelude::*;
use nexus_sync::crdt_uso::*;
use nexus_sync::VersionVector;
use nexus_pcu::PrincipalId;

proptest! {
    #[test]
    fn prop_counter_merge_idempotent(val in 0u64..1000) {
        let owner = PrincipalId::generate();
        let mut uso1 = CrdtUSO::counter("node1", owner);
        uso1.increment(val).unwrap();
        
        let mut uso2 = uso1.clone();
        uso1.merge(&uso2).unwrap();
        
        // merge(A, A) == A
        prop_assert_eq!(uso1.counter_value().unwrap(), uso2.counter_value().unwrap());
        prop_assert_eq!(uso1.id, uso2.id);
    }

    #[test]
    fn prop_counter_merge_commutative(val1 in 0u64..1000, val2 in 0u64..1000) {
        let owner = PrincipalId::generate();
        let mut uso1 = CrdtUSO::counter("node1", owner);
        uso1.increment(val1).unwrap();
        
        let mut uso2 = CrdtUSO::counter("node2", owner);
        uso2.increment(val2).unwrap();
        
        let mut uso1_merge_2 = uso1.clone();
        uso1_merge_2.merge(&uso2).unwrap();
        
        let mut uso2_merge_1 = uso2.clone();
        uso2_merge_1.merge(&uso1).unwrap();
        
        // merge(A, B) == merge(B, A)
        prop_assert_eq!(uso1_merge_2.counter_value().unwrap(), uso2_merge_1.counter_value().unwrap());
        prop_assert_eq!(uso1_merge_2.id, uso2_merge_1.id);
    }

    #[test]
    fn prop_pn_counter_merge_associative(val1 in 0u64..100, val2 in 0u64..100, val3 in 0u64..100) {
        let owner = PrincipalId::generate();
        let mut uso1 = CrdtUSO::pn_counter("node1", owner);
        uso1.increment(val1).unwrap();
        
        let mut uso2 = CrdtUSO::pn_counter("node2", owner);
        uso2.decrement(val2).unwrap();
        
        let mut uso3 = CrdtUSO::pn_counter("node3", owner);
        uso3.increment(val3).unwrap();
        
        // (A merge B) merge C
        let mut left = uso1.clone();
        left.merge(&uso2).unwrap();
        left.merge(&uso3).unwrap();
        
        // A merge (B merge C)
        let mut mid = uso2.clone();
        mid.merge(&uso3).unwrap();
        let mut right = uso1.clone();
        right.merge(&mid).unwrap();
        
        prop_assert_eq!(left.counter_value().unwrap(), right.counter_value().unwrap());
        prop_assert_eq!(left.id, right.id);
    }

    #[test]
    fn prop_set_merge_commutative(
        items1 in prop::collection::vec(any::<String>(), 0..5),
        items2 in prop::collection::vec(any::<String>(), 0..5)
    ) {
        let owner = PrincipalId::generate();
        let mut uso1 = CrdtUSO::set("node1", owner);
        for item in items1 {
            uso1.add_to_set(item).unwrap();
        }
        
        let mut uso2 = CrdtUSO::set("node2", owner);
        for item in items2 {
            uso2.add_to_set(item).unwrap();
        }
        
        let mut left = uso1.clone();
        left.merge(&uso2).unwrap();
        
        let mut right = uso2.clone();
        right.merge(&uso1).unwrap();
        
        prop_assert_eq!(left.set_elements().unwrap(), right.set_elements().unwrap());
    }

    #[test]
    fn prop_vv_happens_before_reflexive(v in arb_vv()) {
        prop_assert!(v.happens_before(&v));
    }

    #[test]
    fn prop_vv_merge_is_lub(v1 in arb_vv(), v2 in arb_vv()) {
        let merged = v1.merge(&v2);
        prop_assert!(v1.happens_before(&merged));
        prop_assert!(v2.happens_before(&merged));
    }
}

fn arb_vv() -> impl Strategy<Value = VersionVector> {
    prop::collection::btree_map("[a-z]", 0..1000u64, 0..5)
        .prop_map(|versions| VersionVector { versions })
}
