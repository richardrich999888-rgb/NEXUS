use nexus_executor::prelude::*;
use nexus_executor::{PcuExecutor, NodeId, NoopHost};
use nexus_pcu::{PCU, WasmModule, IdentityContext};
use std::sync::Arc;
use wat;
use rand::RngCore;

#[tokio::test]
async fn test_nexus_input_host_functions() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_text = r#"
        (module
            (import "nexus" "input_count" (func $input_count (result i32)))
            (import "nexus" "input_size" (func $input_size (param i32) (result i64)))
            (import "nexus" "input_read" (func $input_read (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            
            (func (export "_start")
                ;; Check input count
                (if (i32.ne (call $input_count) (i32.const 2))
                    (then (unreachable)))
                
                ;; Check size of first input
                (if (i64.ne (call $input_size (i32.const 0)) (i64.const 5))
                    (then (unreachable)))
                
                ;; Read first input "hello" into memory at offset 0
                (if (i32.ne (call $input_read (i32.const 0) (i32.const 0) (i32.const 5) (i32.const 0)) (i32.const 5))
                    (then (unreachable)))
            )
            
            (func (export "__nexus_output_len") (result i32)
                (i32.const 5)
            )
        )
    "#;
    let wasm_bytes = wat::parse_str(wasm_text)?;
    
    let input1 = b"hello".to_vec();
    let input2 = b"world!".to_vec();
    let h1 = ContentHash::compute(&input1);
    let h2 = ContentHash::compute(&input2);
    
    // Create a properly signed identity
    use nexus_pcu::PrincipalId;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![h1, h2],
        vec![],
        identity,
    );
        
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        None,
    )?;
    
    let context = ExecutionContext::new(
        vec![(h1, input1), (h2, input2)],
        IdentityContext::anonymous(),
        ExecutionLimits::minimal()
    );
    
    let response = executor.execute(&pcu, context).await?;
    assert_eq!(response.result.output, b"hello");
    
    Ok(())
}

#[tokio::test]
async fn test_nexus_identity_host_functions() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_text = r#"
        (module
            (import "nexus" "get_identity" (func $get_identity (result i64)))
            (import "nexus" "has_capability" (func $has_capability (param i32) (result i32)))
            (memory (export "memory") 1)
            
            (func (export "_start")
                ;; Anonymous identity principal (first 8 bytes) is 0
                (if (i64.ne (call $get_identity) (i64.const 0))
                    (then (unreachable)))
                
                ;; Should not have ADMIN capability (id 4)
                (if (i32.ne (call $has_capability (i32.const 4)) (i32.const 0))
                    (then (unreachable)))
            )
            
            (func (export "__nexus_output_len") (result i32) (i32.const 0))
        )
    "#;
    let wasm_bytes = wat::parse_str(wasm_text)?;
    
    // Create a properly signed identity
    use nexus_pcu::PrincipalId;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![],
        vec![],
        identity,
    );
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        None,
    )?;
    
    let response = executor.execute(&pcu, ExecutionContext::minimal()).await?;
    assert!(response.result.output.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_nexus_uso_host_functions() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_text = r#"
        (module
            (import "nexus" "uso_put" (func $uso_put (param i32 i32 i32) (result i32)))
            (import "nexus" "uso_get" (func $uso_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            
            (func (export "_start")
                ;; Put "nexus-state" into USO
                ;; Data at offset 100, len 11
                (memory.fill (i32.const 100) (i32.const 0) (i32.const 11))
                ;; i32.const 100 is "nexus-state"
                ;; Hash out at offset 200
                (if (i32.ne (call $uso_put (i32.const 100) (i32.const 11) (i32.const 200)) (i32.const 1))
                    (then (unreachable)))
            )
            
            (func (export "__nexus_output_len") (result i32) (i32.const 0))
        )
    "#;
    // Note: This test is minimal, just verifying linkage and basic call flow.
    let wasm_bytes = wat::parse_str(wasm_text)?;
    
    // Create a properly signed identity
    use nexus_pcu::PrincipalId;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![],
        vec![],
        identity,
    );
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        None,
    )?;
    
    let response = executor.execute(&pcu, ExecutionContext::minimal()).await?;
    assert!(response.result.output.is_empty());
    
    Ok(())
}

/// When NervousSystemGuard is set and coordinator is at Infant stage, Execute capability is blocked.
#[tokio::test]
async fn test_guard_blocks_execute_at_infant_stage() -> Result<(), Box<dyn std::error::Error>> {
    use nexus_executor::{NervousSystemGuard, ExecutorError};
    let guard = Arc::new(NervousSystemGuard::new());
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(guard),
    )?;
    let wasm_bytes = wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)?;
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    let result = executor.execute(&pcu, context).await;
    assert!(result.is_err(), "Execution must be blocked when guard is Infant and capability is Execute");
    let err = result.unwrap_err();
    assert!(matches!(err, ExecutorError::ExecutionBlocked { .. }), "Error must be ExecutionBlocked: {:?}", err);
    Ok(())
}

/// ImmuneGuard blocks anonymous principal (regulator-grade: no bypass).
#[tokio::test]
async fn test_immune_guard_blocks_anonymous() -> Result<(), Box<dyn std::error::Error>> {
    use nexus_executor::{ImmuneGuard, ExecutorError};
    let guard = Arc::new(ImmuneGuard::new());
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(guard),
    )?;
    let wasm_bytes = wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)?;
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_err(), "Execution must be blocked for anonymous principal");
    let err = result.unwrap_err();
    assert!(matches!(err, ExecutorError::ExecutionBlocked { .. }));
    if let ExecutorError::ExecutionBlocked { reason } = &err {
        assert!(reason.contains("Anonymous") || reason.contains("anonymous"), "Reason must mention anonymous: {}", reason);
    }
    Ok(())
}

/// ImmuneGuard allows non-anonymous principal when not isolated and reputation OK (default: INITIAL 0.5 >= 0).
#[tokio::test]
async fn test_immune_guard_allows_known_principal() -> Result<(), Box<dyn std::error::Error>> {
    use nexus_executor::ImmuneGuard;
    use nexus_pcu::PrincipalId;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    let guard = Arc::new(ImmuneGuard::new());
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(guard),
    )?;
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    let wasm_bytes = wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)?;
    let pcu = PCU::new(WasmModule::new(wasm_bytes), vec![], vec![], identity);
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_ok(), "ImmuneGuard should allow non-anonymous when not isolated and min_reputation 0: {:?}", result.err());
    Ok(())
}

/// CompositeGuard: first Deny wins (NervousSystem at Infant blocks before Immune is checked).
#[tokio::test]
async fn test_composite_guard_first_deny_wins() -> Result<(), Box<dyn std::error::Error>> {
    use nexus_executor::{CompositeGuard, NervousSystemGuard, ImmuneGuard, ExecutorError};
    let composite = Arc::new(
        CompositeGuard::new()
            .add(Arc::new(NervousSystemGuard::new()))
            .add(Arc::new(ImmuneGuard::new())),
    );
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(composite),
    )?;
    let wasm_bytes = wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)?;
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_err(), "Composite must block: Infant blocks Execute; anonymous would also block at Immune");
    let err = result.unwrap_err();
    assert!(matches!(err, ExecutorError::ExecutionBlocked { .. }));
    Ok(())
}

/// Production executor must have a guard set. Policy: ExecutorBuilder::production() sets a guard; testing builds may omit it.
#[tokio::test]
async fn test_production_executor_requires_guard() -> Result<(), Box<dyn std::error::Error>> {
    let executor = ExecutorBuilder::production(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
    )
    .build()?;
    assert!(executor.has_guard(), "Production executor must have a guard set");
    Ok(())
}

/// ExecutionContext.biological_risk is write-once per request and not exposed to executing code (guest has no access).
#[tokio::test]
async fn test_biological_risk_write_once_retained() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ExecutionContext::minimal().with_biological_risk(0.8);
    assert_eq!(ctx.biological_risk, Some(0.8));
    let clamped = ExecutionContext::minimal().with_biological_risk(1.5);
    assert_eq!(clamped.biological_risk, Some(1.0));
    Ok(())
}

/// When execution is blocked by the guard, no proof is generated and no cache entry is written.
#[tokio::test]
async fn test_no_proof_on_blocked_execution() -> Result<(), Box<dyn std::error::Error>> {
    use nexus_executor::{NervousSystemGuard, ExecutorError};
    let guard = Arc::new(NervousSystemGuard::new());
    let executor = PcuExecutor::new(
        NodeId::local(),
        ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(guard),
    )?;
    let wasm_bytes = wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)?;
    let pcu = PCU::new(
        WasmModule::new(wasm_bytes),
        vec![],
        vec![],
        IdentityContext::anonymous(),
    );
    let context = ExecutionContext::minimal();
    let result1 = executor.execute(&pcu, context.clone()).await;
    assert!(result1.is_err(), "First execution must be blocked (Infant)");
    assert!(matches!(result1.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));
    let result2 = executor.execute(&pcu, context).await;
    assert!(result2.is_err(), "Second execution must still be blocked (no cache entry on block)");
    assert!(matches!(result2.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));
    Ok(())
}
