//! Reputation tests.

use multi_asi_immune::reputation::score::ReputationScore;
use multi_asi_immune::reputation::aggregation::ReputationAggregator;
use multi_asi_immune::identity::keypair::AsiId;

fn make_id(n: u8) -> AsiId {
    AsiId([n; 32])
}

#[test]
fn test_initial_reputation() {
    let score = ReputationScore::new();
    assert_eq!(score.get(0), ReputationScore::INITIAL);
}

#[test]
fn test_reputation_increases_with_positive() {
    let mut score = ReputationScore::new();
    
    for t in 0..20 {
        score.record_positive(t);
    }
    
    assert!(score.get(20) > ReputationScore::INITIAL);
}

#[test]
fn test_reputation_decreases_with_negative() {
    let mut score = ReputationScore::new();
    
    for t in 0..20 {
        score.record_negative(t);
    }
    
    assert!(score.get(20) < ReputationScore::INITIAL);
}

#[test]
fn test_reputation_decay() {
    let mut score = ReputationScore::new();
    
    for t in 0..50 {
        score.record_positive(t);
    }
    
    let at_50 = score.get(50);
    let at_100 = score.get(100);
    let at_500 = score.get(500);
    
    assert!(at_100 < at_50);
    assert!(at_500 < at_100);
}

#[test]
fn test_confidence_increases() {
    let mut score = ReputationScore::new();
    
    let initial_conf = score.confidence();
    
    for t in 0..50 {
        score.record_positive(t);
    }
    
    assert!(score.confidence() > initial_conf);
}

#[test]
fn test_transitive_trust() {
    let mut agg = ReputationAggregator::new(100);
    
    let alice = make_id(1);
    let bob = make_id(2);
    let charlie = make_id(3);
    
    // Alice trusts Bob
    for t in 0..20 {
        agg.record_positive(alice, bob, t);
    }
    
    // Bob trusts Charlie
    for t in 0..20 {
        agg.record_positive(bob, charlie, t);
    }
    
    // Alice should have some trust in Charlie through Bob
    let alice_charlie = agg.get_aggregated(alice, charlie, 20);
    assert!(alice_charlie > ReputationScore::INITIAL);
}

#[test]
fn test_self_reputation() {
    let mut agg = ReputationAggregator::new(100);
    let alice = make_id(1);
    
    assert_eq!(agg.get_aggregated(alice, alice, 0), 1.0);
}

#[test]
fn test_bad_actor_isolation() {
    let mut agg = ReputationAggregator::new(100);
    
    let observer = make_id(1);
    let bad_actor = make_id(2);
    
    for t in 0..50 {
        agg.record_negative(observer, bad_actor, t);
    }
    
    let reputation = agg.get_direct(observer, bad_actor, 50);
    assert!(reputation < 0.15);
}
