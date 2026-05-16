// Property-based tests for nexus-pcu
// Copyright (c) 2025 SYNTRIASS Labs Private Limited

use proptest::prelude::*;
use nexus_pcu::*;

proptest! {
    #[test]
    fn prop_pcu_deterministic_id(
        code_bytes in any::<Vec<u8>>(),
        input_hashes in prop::collection::vec(any::<[u8; 32]>(), 0..10),
        parameters in any::<Vec<u8>>(),
        principal_bytes in any::<[u8; 32]>(),
    ) {
        let code = WasmModule::new(code_bytes);
        let inputs: Vec<ContentHash> = input_hashes.into_iter().map(ContentHash::from_bytes).collect();
        let principal = PrincipalId::from_bytes(principal_bytes);
        let identity = IdentityContext::new(principal, CapabilitySet::default());

        let pcu1 = PCU::new(code.clone(), inputs.clone(), parameters.clone(), identity.clone());
        let pcu2 = PCU::new(code, inputs, parameters, identity);

        prop_assert_eq!(pcu1.id, pcu2.id);
    }

    #[test]
    fn prop_content_hash_determinism(data in any::<Vec<u8>>()) {
        let h1 = ContentHash::compute(&data);
        let h2 = ContentHash::compute(&data);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn prop_pcu_serialization_roundtrip(
        code_bytes in any::<Vec<u8>>(),
        input_hashes in prop::collection::vec(any::<[u8; 32]>(), 0..5),
        parameters in any::<Vec<u8>>(),
        principal_bytes in any::<[u8; 32]>(),
    ) {
        let code = WasmModule::new(code_bytes);
        let inputs: Vec<ContentHash> = input_hashes.into_iter().map(ContentHash::from_bytes).collect();
        let principal = PrincipalId::from_bytes(principal_bytes);
        let identity = IdentityContext::new(principal, CapabilitySet::default());

        let pcu = PCU::new(code, inputs, parameters, identity);
        let bytes = pcu.to_bytes().expect("PCU serialization failed");
        let restored = PCU::from_bytes(&bytes).expect("PCU deserialization failed");

        prop_assert_eq!(pcu.id, restored.id);
        prop_assert_eq!(pcu.code.hash, restored.code.hash);
        prop_assert_eq!(pcu.inputs, restored.inputs);
        prop_assert_eq!(pcu.parameters, restored.parameters);
        prop_assert_eq!(pcu.identity.principal, restored.identity.principal);
    }
}
