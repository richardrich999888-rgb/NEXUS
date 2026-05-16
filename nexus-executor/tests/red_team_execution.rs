//! Adversarial red-team execution tests.
//!
//! Each test attempts a bypass; PASS = bypass fails (guard holds). FAIL = bypass succeeds (security violation).

use nexus_executor::prelude::*;
use nexus_executor::{
    PcuExecutor, ExecutorBuilder, NodeId, NoopHost, ExecutionContext,
    NervousSystemGuard, ImmuneGuard, CompositeGuard, ExecutorError,
    ExecutionGuard, GuardDecision,
};
use nexus_pcu::{PCU, WasmModule, IdentityContext};
use std::sync::Arc;
use ed25519_dalek::SigningKey;

fn minimal_pcu() -> PCU {
    let wasm = WasmModule::new(
        wat::parse_str(r#"(module (memory (export "memory") 1) (func (export "_start")) (func (export "__nexus_output_len") (result i32) (i32.const 0)))"#)
            .unwrap(),
    );
    PCU::new(wasm, vec![], vec![], IdentityContext::anonymous())
}

/// Red-team: Flood execution requests with guard set (Infant). Every request must be blocked.
#[tokio::test]
async fn red_team_flood_requests_all_blocked() {
    let guard = Arc::new(NervousSystemGuard::new());
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(guard),
    )
    .unwrap();
    let pcu = minimal_pcu();
    let ctx = ExecutionContext::minimal();

    for _ in 0..50 {
        let result = executor.execute(&pcu, ctx.clone()).await;
        assert!(result.is_err(), "Red-team: flood must remain blocked (guard holds)");
        assert!(matches!(result.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));
    }
}

/// Red-team: Call execute without setting guard. Baseline: execution succeeds for valid PCU.
/// Confirms that constraint is opt-in via guard; production must use ExecutorBuilder::production().
/// Uses signed identity so executor's pcu.identity.is_valid() passes (anonymous fails validation).
#[tokio::test]
async fn red_team_no_guard_baseline_succeeds() {
    use nexus_pcu::PrincipalId;
    use rand::rngs::OsRng;
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        None,
    )
    .unwrap();
    let mut secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
    let mut identity = IdentityContext::new(principal, nexus_pcu::CapabilitySet::default());
    identity.sign(&signing_key).expect("Signing failed");
    let pcu = PCU::new(
        minimal_pcu().code.clone(),
        vec![],
        vec![],
        identity,
    );
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_ok(), "Baseline (no guard): valid PCU must execute; red-team confirms no guard = no constraint");
}

/// Red-team: After blocked execution, attempt to get cache hit on same PCU. Must still be blocked (no cache write on block).
#[tokio::test]
async fn red_team_no_cache_after_block() {
    let guard = Arc::new(NervousSystemGuard::new());
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(guard),
    )
    .unwrap();
    let pcu = minimal_pcu();
    let ctx = ExecutionContext::minimal();

    let r1 = executor.execute(&pcu, ctx.clone()).await;
    assert!(r1.is_err() && matches!(r1.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));

    let r2 = executor.execute(&pcu, ctx).await;
    assert!(r2.is_err(), "Red-team: second request must still be blocked (no cache entry on block)");
    assert!(matches!(r2.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));
}

/// Red-team: CompositeGuard — first guard (Nervous) blocks Infant; second (Immune) never reached. Deny from first.
#[tokio::test]
async fn red_team_composite_first_deny_wins() {
    let composite = Arc::new(
        CompositeGuard::new()
            .add(Arc::new(NervousSystemGuard::new()))
            .add(Arc::new(ImmuneGuard::new())),
    );
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(composite),
    )
    .unwrap();
    let pcu = minimal_pcu();
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_err(), "Red-team: Composite must block (first guard denies Infant)");
    assert!(matches!(result.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));
}

/// Test-only guard that always allows (used to prove second guard in Composite is applied).
struct AllowAllGuard;
impl ExecutionGuard for AllowAllGuard {
    fn check(&self, _pcu: &PCU, _ctx: &ExecutionContext) -> GuardDecision {
        GuardDecision::Allow
    }
}

/// Guard that records invocation order and optionally denies. Used to verify CompositeGuard order.
struct OrderRecordingGuard {
    id: usize,
    order: std::sync::Arc<std::sync::Mutex<Vec<usize>>>,
    deny: bool,
}
impl ExecutionGuard for OrderRecordingGuard {
    fn check(&self, _pcu: &PCU, _ctx: &ExecutionContext) -> GuardDecision {
        if let Ok(mut v) = self.order.lock() {
            v.push(self.id);
        }
        if self.deny {
            GuardDecision::Deny(format!("order_guard_{}_deny", self.id))
        } else {
            GuardDecision::Allow
        }
    }
}

/// CompositeGuard order invariance: guards execute in declared order; first Deny terminates; later guards not evaluated.
#[tokio::test]
async fn red_team_composite_guard_order_invariance() {
    let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let g0_deny = Arc::new(OrderRecordingGuard { id: 0, order: Arc::clone(&order), deny: true });
    let g1_allow = Arc::new(OrderRecordingGuard { id: 1, order: Arc::clone(&order), deny: false });
    let composite = Arc::new(CompositeGuard::new().add(g0_deny).add(g1_allow));
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(composite),
    )
    .unwrap();
    let pcu = minimal_pcu();
    let _ = executor.execute(&pcu, ExecutionContext::minimal()).await;
    let invoked: Vec<usize> = order.lock().unwrap().clone();
    assert_eq!(invoked, [0], "First Deny must terminate; second guard must not be evaluated. Got: {:?}", invoked);
}

/// CompositeGuard order: when first allows, second is evaluated.
#[tokio::test]
async fn red_team_composite_guard_order_second_evaluated_when_first_allows() {
    let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let g0_allow = Arc::new(OrderRecordingGuard { id: 0, order: Arc::clone(&order), deny: false });
    let g1_deny = Arc::new(OrderRecordingGuard { id: 1, order: Arc::clone(&order), deny: true });
    let composite = Arc::new(CompositeGuard::new().add(g0_allow).add(g1_deny));
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(composite),
    )
    .unwrap();
    let pcu = minimal_pcu();
    let _ = executor.execute(&pcu, ExecutionContext::minimal()).await;
    let invoked: Vec<usize> = order.lock().unwrap().clone();
    assert_eq!(invoked, [0, 1], "Both guards must be evaluated when first allows. Got: {:?}", invoked);
}

/// Red-team: CompositeGuard — when first guard allows, second (Immune) must still block anonymous.
#[tokio::test]
async fn red_team_composite_second_guard_blocks_when_first_allows() {
    let composite = Arc::new(
        CompositeGuard::new()
            .add(Arc::new(AllowAllGuard))
            .add(Arc::new(ImmuneGuard::new())),
    );
    let executor = PcuExecutor::new(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
        1000,
        Some(composite),
    )
    .unwrap();
    let pcu = minimal_pcu();
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_err(), "Red-team: second guard (Immune) must block anonymous");
    let err = result.unwrap_err();
    assert!(matches!(err, ExecutorError::ExecutionBlocked { .. }));
    if let ExecutorError::ExecutionBlocked { reason } = &err {
        assert!(reason.contains("Anonymous") || reason.contains("anonymous"), "Deny must be from ImmuneGuard: {}", reason);
    }
}

/// Red-team: Attempt execution with production build. Guard is set; Infant PCU must be blocked.
#[tokio::test]
async fn red_team_production_blocks_infant() {
    let executor = ExecutorBuilder::production(
        NodeId::local(),
        SigningKey::from_bytes(&[1u8; 32]),
        Arc::new(NoopHost),
    )
    .build()
    .unwrap();
    assert!(executor.has_guard(), "Production must have guard");
    let pcu = minimal_pcu();
    let result = executor.execute(&pcu, ExecutionContext::minimal()).await;
    assert!(result.is_err(), "Red-team: production executor must block Infant PCU");
    assert!(matches!(result.unwrap_err(), ExecutorError::ExecutionBlocked { .. }));
}
