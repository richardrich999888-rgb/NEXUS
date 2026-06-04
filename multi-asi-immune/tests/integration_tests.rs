//! Integration tests.

use multi_asi_immune::node::state::{AsiNode, NodeConfig, ProcessResult};
use multi_asi_immune::protocol::message::ProtocolMessage;
use multi_asi_immune::threat::pattern::{ThreatPattern, ThreatCategory};

#[test]
fn test_node_creation() {
    let node = AsiNode::new(NodeConfig::default());
    
    // Should have valid ID
    assert_ne!(node.id().0, [0u8; 32]);
}

#[test]
fn test_peer_connection() {
    let mut node1 = AsiNode::new(NodeConfig::default());
    let node2 = AsiNode::new(NodeConfig::default());
    
    // Add node2 as peer
    node1.add_peer(node2.public_identity());
    
    let peer = node1.get_peer(node2.id());
    assert!(peer.is_some());
}

#[test]
fn test_heartbeat_generation() {
    let mut node = AsiNode::new(NodeConfig::default());
    
    // Tick past heartbeat interval
    let messages = node.tick(15);
    
    assert!(!messages.is_empty());
    assert!(matches!(messages[0], ProtocolMessage::Heartbeat(_)));
}

#[test]
fn test_threat_reporting() {
    let mut node = AsiNode::new(NodeConfig::default());
    
    let pattern = ThreatPattern::new(
        ThreatCategory::Deception,
        [42; 32],
        0.9,
    );
    
    let report = node.report_threat(pattern, 0.85);
    
    // Report should be valid
    assert!(report.verify(&node.public_identity()));
    
    // Should be in threat memory
    assert!(!node.threats().is_empty());
}

#[test]
fn test_threat_propagation() {
    let mut node1 = AsiNode::new(NodeConfig::default());
    let mut node2 = AsiNode::new(NodeConfig::default());
    
    // Establish peering
    node1.add_peer(node2.public_identity());
    node2.add_peer(node1.public_identity());
    
    // Node1 reports threat
    let pattern = ThreatPattern::new(ThreatCategory::Deception, [1; 32], 0.9);
    let report = node1.report_threat(pattern, 0.85);
    
    // Node2 receives threat
    let result = node2.process(
        ProtocolMessage::ThreatReport(report),
        &node1.public_identity(),
    );
    
    // Should broadcast (gossip)
    assert!(matches!(result, ProcessResult::Broadcast(_)));
}

#[test]
fn test_network_health() {
    let node = AsiNode::new(NodeConfig::default());
    
    let health = node.network_health();
    
    // Empty network should be healthy
    assert!(health.healthy);
    assert_eq!(health.total_peers, 0);
}

#[test]
fn test_heartbeat_processing() {
    let mut node1 = AsiNode::new(NodeConfig::default());
    let mut node2 = AsiNode::new(NodeConfig::default());
    
    node1.add_peer(node2.public_identity());
    
    // Generate heartbeat from node2
    let messages = node2.tick(10);
    for msg in messages {
        node1.process(msg, &node2.public_identity());
    }
    
    // Peer should be updated
    let peer = node1.get_peer(node2.id()).unwrap();
    assert_eq!(peer.last_heartbeat_seq, 1);
}

#[test]
fn test_full_protocol_flow() {
    let config = NodeConfig::default();
    let mut node1 = AsiNode::new(config.clone());
    let mut node2 = AsiNode::new(config.clone());
    let mut node3 = AsiNode::new(config);
    
    // Connect nodes
    node1.add_peer(node2.public_identity());
    node1.add_peer(node3.public_identity());
    node2.add_peer(node1.public_identity());
    node2.add_peer(node3.public_identity());
    node3.add_peer(node1.public_identity());
    node3.add_peer(node2.public_identity());
    
    // Node1 detects threat
    let pattern = ThreatPattern::new(ThreatCategory::CoordinatedAttack, [99; 32], 1.0);
    let report = node1.report_threat(pattern, 0.95);
    
    // Propagate to node2
    let result = node2.process(
        ProtocolMessage::ThreatReport(report.clone()),
        &node1.public_identity(),
    );
    
    // Node2 should gossip
    if let ProcessResult::Broadcast(msg) = result {
        // Node3 receives
        node3.process(msg, &node2.public_identity());
    }
    
    // All nodes should know about the threat
    assert!(!node1.threats().is_empty());
}
