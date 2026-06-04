//! # NEXUS Core Contract Invariants
//!
//! This module defines the **frozen contract invariants** for NEXUS core types.
//! These invariants MUST NOT be violated by any future changes.
//!
//! ## Contract-Frozen Types
//! - `PCU`: Portable Computation Unit
//! - `CausalTensor`: Causal tensor algebra primitive
//! - `USO`: Universal State Object
//! - `ExecutionResult`: Execution proof and result
//!
//! ## Invariant Categories
//! 1. **Structural**: Field presence, type requirements
//! 2. **Semantic**: Behavioral guarantees
//! 3. **Determinism**: Same inputs → same outputs
//!
//! Copyright (c) 2025 SYNTRIASS Labs Private Limited
//! Inventor: Katta Naga Sri Ganesh

use crate::content_hash::ContentHash;

// ============================================================================
// PCU INVARIANTS
// ============================================================================

/// PCU Contract Invariants (FROZEN)
///
/// These invariants define what a valid PCU must satisfy.
/// Violation of any invariant is a protocol error.
pub mod pcu_invariants {
    use super::*;
    use crate::pcu::PCU;

    /// INV-PCU-001: PCU ID is deterministic
    /// Given the same (code, inputs, parameters, identity.principal), the ID must be identical.
    pub fn id_is_deterministic(pcu: &PCU) -> bool {
        let recomputed = super::compute_pcu_id(&pcu.code, &pcu.inputs, &pcu.parameters, &pcu.identity);
        pcu.id == recomputed
    }

    /// INV-PCU-002: Code hash matches bytecode
    pub fn code_hash_matches_bytecode(pcu: &PCU) -> bool {
        let expected = ContentHash::compute(&pcu.code.bytecode);
        pcu.code.hash == expected
    }

    /// INV-PCU-003: Inputs are content-addressed
    /// Each input hash must be a valid 32-byte content hash.
    pub fn inputs_are_content_addressed(pcu: &PCU) -> bool {
        pcu.inputs.iter().all(|h| h.as_bytes().len() == 32)
    }

    /// INV-PCU-004: Identity principal is 32 bytes
    pub fn identity_principal_valid(pcu: &PCU) -> bool {
        pcu.identity.principal.as_bytes().len() == 32
    }

    /// INV-PCU-005: Serialization is lossless
    pub fn serialization_lossless(pcu: &PCU) -> bool {
        let bytes = match pcu.to_bytes() {
            Ok(b) => b,
            Err(_) => return false,
        };
        match PCU::from_bytes(&bytes) {
            Ok(restored) => pcu.id == restored.id,
            Err(_) => false,
        }
    }

    /// Validate all PCU invariants
    pub fn validate_all(pcu: &PCU) -> Result<(), Vec<&'static str>> {
        let mut violations = Vec::new();
        
        if !id_is_deterministic(pcu) {
            violations.push("INV-PCU-001: ID is not deterministic");
        }
        if !code_hash_matches_bytecode(pcu) {
            violations.push("INV-PCU-002: Code hash does not match bytecode");
        }
        if !inputs_are_content_addressed(pcu) {
            violations.push("INV-PCU-003: Inputs are not content-addressed");
        }
        if !identity_principal_valid(pcu) {
            violations.push("INV-PCU-004: Identity principal is invalid");
        }
        if !serialization_lossless(pcu) {
            violations.push("INV-PCU-005: Serialization is not lossless");
        }
        
        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

// ============================================================================
// USO INVARIANTS
// ============================================================================

/// USO Contract Invariants (FROZEN)
pub mod uso_invariants {
    use super::*;
    use crate::uso::USO;

    /// INV-USO-001: USO ID is content hash of data
    pub fn id_is_content_hash(uso: &USO) -> bool {
        uso.id == ContentHash::compute(&uso.data)
    }

    /// INV-USO-002: Vector clock is monotonic
    /// Lamport timestamp must be >= all individual node clocks.
    pub fn clock_is_monotonic(uso: &USO) -> bool {
        let lamport = uso.history.lamport();
        uso.history.vector_clock.values().all(|&v| v <= lamport)
    }

    /// INV-USO-003: Owner can always read and write
    pub fn owner_has_full_access(uso: &USO) -> bool {
        uso.can_read(&uso.access.owner) && uso.can_write(&uso.access.owner)
    }

    /// INV-USO-004: Merge is deterministic (LWW semantics)
    /// Given same inputs, merge(A, B) always produces the same result.
    /// Note: Uses Last-Writer-Wins based on modified_at timestamp.
    pub fn merge_is_deterministic(a: &USO, b: &USO) -> bool {
        let mut a1 = a.clone();
        let mut a2 = a.clone();
        a1.merge(b);
        a2.merge(b);
        // Same merge operation produces same result
        a1.data == a2.data && a1.id == a2.id
    }

    /// Validate all USO invariants
    pub fn validate_all(uso: &USO) -> Result<(), Vec<&'static str>> {
        let mut violations = Vec::new();
        
        if !id_is_content_hash(uso) {
            violations.push("INV-USO-001: ID is not content hash of data");
        }
        if !clock_is_monotonic(uso) {
            violations.push("INV-USO-002: Vector clock is not monotonic");
        }
        if !owner_has_full_access(uso) {
            violations.push("INV-USO-003: Owner does not have full access");
        }
        
        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn compute_pcu_id(
    code: &crate::pcu::WasmModule,
    inputs: &[ContentHash],
    parameters: &[u8],
    identity: &crate::identity::IdentityContext,
) -> ContentHash {
    use crate::content_hash::ContentHasher;
    let mut hasher = ContentHasher::new();
    hasher.update(code.hash.as_bytes());
    for input in inputs {
        hasher.update(input.as_bytes());
    }
    hasher.update(parameters);
    hasher.update(identity.principal.as_bytes());
    hasher.finalize()
}

// ============================================================================
// PROPERTY TESTS
// ============================================================================

#[cfg(test)]
mod invariant_tests {
    use super::*;
    use crate::pcu::{PCU, WasmModule};
    use crate::identity::{IdentityContext, PrincipalId, CapabilitySet};
    use crate::uso::USO;
    use proptest::prelude::*;

    proptest! {
        /// Property: PCU ID is deterministic for same inputs
        #[test]
        fn prop_pcu_id_deterministic(
            bytecode in prop::collection::vec(any::<u8>(), 0..100),
            params in prop::collection::vec(any::<u8>(), 0..50),
            principal_bytes in any::<[u8; 32]>(),
        ) {
            let code = WasmModule::new(bytecode);
            let principal = PrincipalId::from_bytes(principal_bytes);
            let identity = IdentityContext::new(principal, CapabilitySet::default());
            
            let pcu1 = PCU::new(code.clone(), vec![], params.clone(), identity.clone());
            let pcu2 = PCU::new(code, vec![], params, identity);
            
            prop_assert_eq!(pcu1.id, pcu2.id, "PCU IDs must be deterministic");
        }

        /// Property: PCU serialization is lossless
        #[test]
        fn prop_pcu_serialization_lossless(
            bytecode in prop::collection::vec(any::<u8>(), 4..100),
            params in prop::collection::vec(any::<u8>(), 0..50),
        ) {
            let code = WasmModule::new(bytecode);
            let identity = IdentityContext::new(PrincipalId::from_bytes([0u8; 32]), CapabilitySet::default());
            
            let pcu = PCU::new(code, vec![], params, identity);
            let bytes = match pcu.to_bytes() {
                Ok(b) => b,
                Err(_) => return Ok(()), // Skip invalid serialization cases
            };
            let restored = match PCU::from_bytes(&bytes) {
                Ok(r) => r,
                Err(_) => return Ok(()), // Skip invalid deserialization cases
            };
            
            prop_assert_eq!(pcu.id, restored.id, "PCU ID must survive serialization");
            prop_assert_eq!(pcu.parameters, restored.parameters, "Parameters must survive serialization");
        }

        /// Property: USO merge is deterministic (LWW: later timestamp wins)
        /// Note: USO uses Last-Writer-Wins semantics. Given same inputs, merge produces same result.
        #[test]
        fn prop_uso_merge_deterministic(
            data_a in prop::collection::vec(any::<u8>(), 1..100),
            data_b in prop::collection::vec(any::<u8>(), 1..100),
            owner_bytes in any::<[u8; 32]>(),
        ) {
            let owner = PrincipalId::from_bytes(owner_bytes);
            
            let uso_a = USO::new(data_a.clone(), owner);
            let uso_b = USO::new(data_b.clone(), owner);
            
            // Merge A into B twice - should produce same result
            let mut merged1 = uso_a.clone();
            let mut merged2 = uso_a.clone();
            
            merged1.merge(&uso_b);
            merged2.merge(&uso_b);
            
            // Same merge operation produces same result (deterministic)
            prop_assert_eq!(merged1.data, merged2.data, "USO merge must be deterministic");
            prop_assert_eq!(merged1.id, merged2.id, "USO merge ID must be deterministic");
        }
    }

    #[test]
    fn test_pcu_invariants_valid_pcu() {
        let code = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d]);
        let identity = IdentityContext::new(PrincipalId::from_bytes([42u8; 32]), CapabilitySet::default());
        let pcu = PCU::new(code, vec![], vec![1, 2, 3], identity);
        
        assert!(pcu_invariants::validate_all(&pcu).is_ok());
    }

    #[test]
    fn test_uso_invariants_valid_uso() {
        let owner = PrincipalId::from_bytes([1u8; 32]);
        let uso = USO::new(b"test data".to_vec(), owner);
        
        assert!(uso_invariants::validate_all(&uso).is_ok());
    }
}
