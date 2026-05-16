//! Defection detection tests.

use multi_asi_immune::identity::keypair::AsiId;
use multi_asi_immune::enforcement::defection::{DefectionTracker, DefectionRecord, DefectionType};
use multi_asi_immune::protocol::message::AccusationEvidence;

fn make_id(n: u8) -> AsiId {
    AsiId([n; 32])
}

#[test]
fn test_defection_recording() {
    let mut tracker = DefectionTracker::new();
    
    let bad_node = make_id(1);
    let detector = make_id(2);
    
    tracker.record(DefectionRecord {
        node: bad_node,
        defection_type: DefectionType::Unresponsive,
        evidence: AccusationEvidence::MissedHeartbeats {
            expected_count: 5,
            received_count: 0,
        },
        detected_at: 100,
        detected_by: detector,
    });
    
    assert_eq!(tracker.defection_count(bad_node), 1);
}

#[test]
fn test_cumulative_severity() {
    let mut tracker = DefectionTracker::new();
    
    let bad_node = make_id(1);
    let detector = make_id(2);
    
    // Minor defection
    tracker.record(DefectionRecord {
        node: bad_node,
        defection_type: DefectionType::Unresponsive,
        evidence: AccusationEvidence::MissedHeartbeats {
            expected_count: 5,
            received_count: 0,
        },
        detected_at: 100,
        detected_by: detector,
    });
    
    let severity1 = tracker.cumulative_severity(bad_node);
    
    // Severe defection
    tracker.record(DefectionRecord {
        node: bad_node,
        defection_type: DefectionType::Contradictory,
        evidence: AccusationEvidence::Contradiction {
            message1: vec![1],
            message2: vec![2],
        },
        detected_at: 200,
        detected_by: detector,
    });
    
    let severity2 = tracker.cumulative_severity(bad_node);
    
    assert!(severity2 > severity1);
}

#[test]
fn test_isolation_threshold() {
    let mut tracker = DefectionTracker::with_threshold(1.0);
    
    let bad_node = make_id(1);
    let detector = make_id(2);
    
    // Not yet isolated
    assert!(!tracker.should_isolate(bad_node));
    
    // Identity forgery has severity 1.0
    tracker.record(DefectionRecord {
        node: bad_node,
        defection_type: DefectionType::IdentityForgery,
        evidence: AccusationEvidence::InvalidSignature {
            message: vec![1, 2, 3],
        },
        detected_at: 100,
        detected_by: detector,
    });
    
    assert!(tracker.should_isolate(bad_node));
}

#[test]
fn test_no_defections_no_isolation() {
    let tracker = DefectionTracker::new();
    let good_node = make_id(1);
    
    assert!(!tracker.should_isolate(good_node));
    assert_eq!(tracker.defection_count(good_node), 0);
}

#[test]
fn test_defection_types_severity() {
    assert!(DefectionType::IdentityForgery.severity() > DefectionType::Unresponsive.severity());
    assert!(DefectionType::InvalidSignatures.severity() > DefectionType::ConstraintViolation.severity());
}
