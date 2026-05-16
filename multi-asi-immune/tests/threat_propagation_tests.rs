//! Threat propagation tests.

use multi_asi_immune::identity::keypair::AsiIdentity;
use multi_asi_immune::threat::pattern::{ThreatPattern, ThreatCategory};
use multi_asi_immune::threat::signature::SignedThreatReport;
use multi_asi_immune::threat::memory::{ThreatMemory, ThreatAddResult};
use multi_asi_immune::reputation::aggregation::ReputationAggregator;

#[test]
fn test_threat_report_signing() {
    let identity = AsiIdentity::generate();
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [42; 32], 0.9);
    
    let report = SignedThreatReport::new(&identity, pattern, 0.85, 100);
    
    assert!(report.verify(&identity.public_identity()));
}

#[test]
fn test_threat_memory_add() {
    let mut memory = ThreatMemory::new(100, 3600);
    let reputation = ReputationAggregator::new(100);
    let identity = AsiIdentity::generate();
    
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
    let report = SignedThreatReport::new(&identity, pattern, 0.8, 0);
    
    let result = memory.add(report, &reputation, identity.id, 0);
    
    assert!(matches!(result, ThreatAddResult::Added));
    assert_eq!(memory.len(), 1);
}

#[test]
fn test_duplicate_rejection() {
    let mut memory = ThreatMemory::new(100, 3600);
    let reputation = ReputationAggregator::new(100);
    let identity = AsiIdentity::generate();
    
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
    let report = SignedThreatReport::new(&identity, pattern.clone(), 0.8, 0);
    
    memory.add(report.clone(), &reputation, identity.id, 0);
    
    // Same pattern, same reporter = duplicate
    let result = memory.add(report, &reputation, identity.id, 1);
    
    assert!(matches!(result, ThreatAddResult::Duplicate));
    assert_eq!(memory.len(), 1);
}

#[test]
fn test_pattern_confirmation() {
    let mut memory = ThreatMemory::new(100, 3600);
    let mut reputation = ReputationAggregator::new(100);
    
    let identity1 = AsiIdentity::generate();
    let identity2 = AsiIdentity::generate();
    
    // Build reputation for identity2
    for t in 0..20 {
        reputation.record_positive(identity1.id, identity2.id, t);
    }
    
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
    
    // First report
    let report1 = SignedThreatReport::new(&identity1, pattern.clone(), 0.8, 0);
    memory.add(report1, &reputation, identity1.id, 0);
    
    // Second report from different identity
    let report2 = SignedThreatReport::new(&identity2, pattern, 0.85, 1);
    let result = memory.add(report2, &reputation, identity1.id, 1);
    
    assert!(matches!(result, ThreatAddResult::Confirmed { .. }));
}

#[test]
fn test_threat_expiry() {
    let mut memory = ThreatMemory::new(100, 100); // Short TTL
    let reputation = ReputationAggregator::new(100);
    let identity = AsiIdentity::generate();
    
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
    let report = SignedThreatReport::new(&identity, pattern, 0.8, 0);
    
    memory.add(report, &reputation, identity.id, 0);
    assert_eq!(memory.len(), 1);
    
    memory.expire(200);
    assert_eq!(memory.len(), 0);
}

#[test]
fn test_active_threats_threshold() {
    let mut memory = ThreatMemory::new(100, 3600);
    let reputation = ReputationAggregator::new(100);
    let identity = AsiIdentity::generate();
    
    // Add threat with low confidence
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.3);
    let report = SignedThreatReport::new(&identity, pattern, 0.3, 0);
    memory.add(report, &reputation, identity.id, 0);
    
    // Should not appear in active threats with high threshold
    assert!(memory.active_threats(0.8).is_empty());
    
    // Should appear with low threshold
    assert!(!memory.active_threats(0.1).is_empty());
}
