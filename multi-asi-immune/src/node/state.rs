//! Complete state of an ASI node in the multi-ASI protocol.

use crate::identity::keypair::{AsiIdentity, AsiId, PublicIdentity};
use crate::reputation::aggregation::ReputationAggregator;
use crate::threat::memory::ThreatMemory;
use crate::threat::signature::SignedThreatReport;
use crate::threat::pattern::ThreatPattern;
use crate::protocol::message::*;
use crate::enforcement::defection::{DefectionTracker, DefectionRecord, DefectionType};
use std::collections::{HashMap, HashSet};

/// Complete state of an ASI node in the multi-ASI protocol.
pub struct AsiNode {
    /// This node's identity.
    identity: AsiIdentity,
    /// Known peers and their public identities.
    peers: HashMap<AsiId, PeerState>,
    /// Reputation tracking.
    reputation: ReputationAggregator,
    /// Known threats.
    threats: ThreatMemory,
    /// Active mutual constraints.
    constraints: HashMap<[u8; 32], ActiveConstraint>,
    /// Pending constraint proposals.
    pending_proposals: HashMap<[u8; 32], ConstraintProposalMessage>,
    /// Defection tracking.
    defections: DefectionTracker,
    /// Configuration.
    config: NodeConfig,
    /// Monotonic clock.
    current_time: u64,
    /// Heartbeat sequence number.
    heartbeat_seq: u64,
}

/// State of a known peer.
#[derive(Debug, Clone)]
pub struct PeerState {
    pub public_identity: PublicIdentity,
    pub capabilities: HashSet<Capability>,
    pub last_seen: u64,
    pub last_heartbeat_seq: u64,
    pub missed_heartbeats: u32,
    pub status: PeerStatus,
}

/// Peer connection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerStatus {
    /// Normal operation.
    Active,
    /// Missed some heartbeats, monitoring.
    Suspicious,
    /// Isolated due to defection.
    Isolated,
    /// Connection not yet established.
    Pending,
}

/// Active mutual constraint.
#[derive(Debug, Clone)]
pub struct ActiveConstraint {
    pub constraint: MutualConstraint,
    pub counterparty: AsiId,
    pub activated_at: u64,
    pub expires_at: u64,
    pub triggered: bool,
}

/// Node configuration.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Threat acceptance threshold.
    pub threat_threshold: f64,
    /// Heartbeat interval.
    pub heartbeat_interval: u64,
    /// Max missed heartbeats before suspicious.
    pub max_missed_heartbeats: u32,
    /// Max missed heartbeats before isolation.
    pub isolation_threshold: u32,
    /// Reputation cache TTL.
    pub reputation_cache_ttl: u64,
    /// Threat memory capacity.
    pub threat_capacity: usize,
    /// Threat TTL.
    pub threat_ttl: u64,
    /// Protocol version.
    pub protocol_version: u32,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            threat_threshold: 0.6,
            heartbeat_interval: 10,
            max_missed_heartbeats: 3,
            isolation_threshold: 10,
            reputation_cache_ttl: 100,
            threat_capacity: 10000,
            threat_ttl: 86400, // 24 hours
            protocol_version: 1,
        }
    }
}

/// Result of processing a message.
#[derive(Debug)]
pub enum ProcessResult {
    /// Message processed successfully.
    Ok,
    /// Response message to send.
    Reply(ProtocolMessage),
    /// Broadcast message to all peers.
    Broadcast(ProtocolMessage),
    /// Peer should be isolated.
    Isolate(AsiId),
    /// Invalid message.
    Invalid(String),
}

/// Network health assessment.
#[derive(Debug, Clone)]
pub struct NetworkHealth {
    pub total_peers: usize,
    pub active: usize,
    pub suspicious: usize,
    pub isolated: usize,
    pub active_threats: usize,
    pub active_constraints: usize,
    pub healthy: bool,
}

impl AsiNode {
    /// Creates a new ASI node with the given configuration.
    pub fn new(config: NodeConfig) -> Self {
        let identity = AsiIdentity::generate();
        
        Self {
            identity,
            peers: HashMap::new(),
            reputation: ReputationAggregator::new(config.reputation_cache_ttl),
            threats: ThreatMemory::new(config.threat_capacity, config.threat_ttl),
            constraints: HashMap::new(),
            pending_proposals: HashMap::new(),
            defections: DefectionTracker::new(),
            config,
            current_time: 0,
            heartbeat_seq: 0,
        }
    }
    
    /// Creates a node with a specific identity.
    pub fn with_identity(identity: AsiIdentity, config: NodeConfig) -> Self {
        Self {
            identity,
            peers: HashMap::new(),
            reputation: ReputationAggregator::new(config.reputation_cache_ttl),
            threats: ThreatMemory::new(config.threat_capacity, config.threat_ttl),
            constraints: HashMap::new(),
            pending_proposals: HashMap::new(),
            defections: DefectionTracker::new(),
            config,
            current_time: 0,
            heartbeat_seq: 0,
        }
    }
    
    /// Returns this node's ID.
    pub fn id(&self) -> AsiId {
        self.identity.id
    }
    
    /// Returns this node's public identity.
    pub fn public_identity(&self) -> PublicIdentity {
        self.identity.public_identity()
    }
    
    /// Returns current time.
    pub fn current_time(&self) -> u64 {
        self.current_time
    }
    
    /// Advances the node's clock and performs periodic maintenance.
    pub fn tick(&mut self, new_time: u64) -> Vec<ProtocolMessage> {
        let old_time = self.current_time;
        self.current_time = new_time;
        
        let mut messages = Vec::new();
        
        // Check for heartbeat
        if new_time / self.config.heartbeat_interval > old_time / self.config.heartbeat_interval {
            messages.push(self.generate_heartbeat());
        }
        
        // Check peer liveness
        for (id, peer) in &mut self.peers {
            if peer.status == PeerStatus::Isolated {
                continue;
            }
            
            let since_seen = new_time.saturating_sub(peer.last_seen);
            let expected = since_seen / self.config.heartbeat_interval;
            
            if expected > peer.last_heartbeat_seq + self.config.max_missed_heartbeats as u64 {
                peer.missed_heartbeats += 1;
                
                if peer.missed_heartbeats >= self.config.isolation_threshold {
                    peer.status = PeerStatus::Isolated;
                    
                    // Record defection
                    self.defections.record(DefectionRecord {
                        node: *id,
                        defection_type: DefectionType::Unresponsive,
                        evidence: AccusationEvidence::MissedHeartbeats {
                            expected_count: expected,
                            received_count: peer.last_heartbeat_seq,
                        },
                        detected_at: new_time,
                        detected_by: self.identity.id,
                    });
                } else if peer.missed_heartbeats >= self.config.max_missed_heartbeats {
                    peer.status = PeerStatus::Suspicious;
                }
            }
        }
        
        // Expire old threats
        self.threats.expire(new_time);
        
        // Check constraint expirations
        self.constraints.retain(|_, c| c.expires_at > new_time);
        
        messages
    }
    
    /// Adds a peer with their public identity.
    pub fn add_peer(&mut self, public_identity: PublicIdentity) {
        let peer = PeerState {
            public_identity: public_identity.clone(),
            capabilities: HashSet::new(),
            last_seen: self.current_time,
            last_heartbeat_seq: 0,
            missed_heartbeats: 0,
            status: PeerStatus::Pending,
        };
        self.peers.insert(public_identity.id, peer);
    }
    
    /// Returns a reference to a peer's state.
    pub fn get_peer(&self, id: AsiId) -> Option<&PeerState> {
        self.peers.get(&id)
    }

    /// Checks whether a principal (by 32-byte id) is allowed to execute from this node's perspective.
    /// Denies if the principal is isolated due to defection or if aggregated reputation is below threshold.
    pub fn allow_execution_by(
        &mut self,
        principal_id: [u8; 32],
        min_reputation: f64,
    ) -> Result<(), String> {
        let asi_id = AsiId::from_bytes(principal_id);
        if self.defections.should_isolate(asi_id) {
            return Err("Principal is isolated due to defection".to_string());
        }
        let rep = self.reputation.get_aggregated(self.identity.id, asi_id, self.current_time);
        if rep < min_reputation {
            return Err(format!(
                "Reputation {} below threshold {}",
                rep, min_reputation
            ));
        }
        Ok(())
    }

    /// Processes an incoming message.
    pub fn process(&mut self, msg: ProtocolMessage, sender_key: &PublicIdentity) -> ProcessResult {
        // Check if sender is isolated
        if let Some(peer) = self.peers.get(&sender_key.id) {
            if peer.status == PeerStatus::Isolated {
                return ProcessResult::Invalid("Sender is isolated".into());
            }
        }
        
        match msg {
            ProtocolMessage::Hello(hello) => self.process_hello(hello, sender_key),
            ProtocolMessage::HelloAck(ack) => self.process_hello_ack(ack),
            ProtocolMessage::ThreatReport(report) => self.process_threat_report(report, sender_key),
            ProtocolMessage::Heartbeat(hb) => self.process_heartbeat(hb),
            ProtocolMessage::ConstraintProposal(prop) => self.process_constraint_proposal(prop),
            ProtocolMessage::ConstraintAccept(acc) => self.process_constraint_accept(acc),
            ProtocolMessage::Accusation(acc) => self.process_accusation(acc, sender_key),
            _ => ProcessResult::Ok,
        }
    }
    
    fn process_hello(&mut self, hello: HelloMessage, sender_key: &PublicIdentity) -> ProcessResult {
        // Add or update peer
        let peer = PeerState {
            public_identity: sender_key.clone(),
            capabilities: hello.capabilities.into_iter().collect(),
            last_seen: self.current_time,
            last_heartbeat_seq: 0,
            missed_heartbeats: 0,
            status: PeerStatus::Active,
        };
        self.peers.insert(hello.sender, peer);
        
        // Generate ack
        let ack = HelloAckMessage {
            sender: self.identity.id,
            in_response_to: hello.sender,
            accepted: true,
            timestamp: self.current_time,
            signature: self.identity.sign(b"ack").to_bytes().to_vec(),
        };
        
        ProcessResult::Reply(ProtocolMessage::HelloAck(ack))
    }
    
    fn process_hello_ack(&mut self, ack: HelloAckMessage) -> ProcessResult {
        if let Some(peer) = self.peers.get_mut(&ack.sender) {
            if ack.accepted {
                peer.status = PeerStatus::Active;
            }
        }
        ProcessResult::Ok
    }
    
    fn process_threat_report(&mut self, report: SignedThreatReport, sender_key: &PublicIdentity) -> ProcessResult {
        // Verify signature
        if !report.verify(sender_key) {
            return ProcessResult::Invalid("Invalid signature on threat report".into());
        }
        
        // Add to threat memory
        let result = self.threats.add(
            report.clone(),
            &self.reputation,
            self.identity.id,
            self.current_time,
        );
        
        match result {
            crate::threat::memory::ThreatAddResult::Added |
            crate::threat::memory::ThreatAddResult::Confirmed { .. } => {
                // Forward to peers (gossip)
                ProcessResult::Broadcast(ProtocolMessage::ThreatReport(report))
            }
            _ => ProcessResult::Ok,
        }
    }
    
    fn process_heartbeat(&mut self, hb: HeartbeatMessage) -> ProcessResult {
        if let Some(peer) = self.peers.get_mut(&hb.sender) {
            peer.last_seen = self.current_time;
            peer.last_heartbeat_seq = hb.sequence;
            peer.missed_heartbeats = 0;
            
            if peer.status == PeerStatus::Suspicious {
                peer.status = PeerStatus::Active;
            }
        }
        ProcessResult::Ok
    }
    
    fn process_constraint_proposal(&mut self, prop: ConstraintProposalMessage) -> ProcessResult {
        if prop.target != self.identity.id {
            return ProcessResult::Ok;
        }
        
        // Check if we already have this constraint
        if self.constraints.contains_key(&prop.constraint.id) {
            return ProcessResult::Ok;
        }
        
        self.pending_proposals.insert(prop.constraint.id, prop.clone());
        
        // Auto-accept for now (real impl would have policy)
        let accept = ConstraintAcceptMessage {
            acceptor: self.identity.id,
            constraint_id: prop.constraint.id,
            accepted: true,
            counter_proposal: None,
            timestamp: self.current_time,
            signature: self.identity.sign(&prop.constraint.id).to_bytes().to_vec(),
        };
        
        // Activate constraint
        self.constraints.insert(prop.constraint.id, ActiveConstraint {
            constraint: prop.constraint,
            counterparty: prop.proposer,
            activated_at: self.current_time,
            expires_at: self.current_time + prop.valid_until,
            triggered: false,
        });
        
        ProcessResult::Reply(ProtocolMessage::ConstraintAccept(accept))
    }
    
    fn process_constraint_accept(&mut self, acc: ConstraintAcceptMessage) -> ProcessResult {
        if let Some(prop) = self.pending_proposals.remove(&acc.constraint_id) {
            if acc.accepted {
                self.constraints.insert(acc.constraint_id, ActiveConstraint {
                    constraint: prop.constraint,
                    counterparty: acc.acceptor,
                    activated_at: self.current_time,
                    expires_at: self.current_time + prop.valid_until,
                    triggered: false,
                });
            }
        }
        ProcessResult::Ok
    }
    
    fn process_accusation(&mut self, acc: AccusationMessage, sender_key: &PublicIdentity) -> ProcessResult {
        // Get accuser's reputation
        let accuser_rep = self.reputation.get_direct(
            self.identity.id,
            acc.accuser,
            self.current_time,
        );
        
        // Only act on accusations from reputable sources
        if accuser_rep > 0.5 {
            self.reputation.record_negative(self.identity.id, acc.accused, self.current_time);
            
            // If accused reputation drops too low, isolate
            let accused_rep = self.reputation.get_aggregated(
                self.identity.id,
                acc.accused,
                self.current_time,
            );
            
            if accused_rep < 0.2 {
                if let Some(peer) = self.peers.get_mut(&acc.accused) {
                    peer.status = PeerStatus::Isolated;
                }
                return ProcessResult::Isolate(acc.accused);
            }
        }
        
        ProcessResult::Ok
    }
    
    fn generate_heartbeat(&mut self) -> ProtocolMessage {
        self.heartbeat_seq += 1;
        
        let sig_data = format!("HB:{}:{}", self.heartbeat_seq, self.current_time);
        
        ProtocolMessage::Heartbeat(HeartbeatMessage {
            sender: self.identity.id,
            sequence: self.heartbeat_seq,
            timestamp: self.current_time,
            signature: self.identity.sign(sig_data.as_bytes()).to_bytes().to_vec(),
        })
    }
    
    /// Reports a detected threat to the network.
    pub fn report_threat(&mut self, pattern: ThreatPattern, confidence: f64) -> SignedThreatReport {
        let report = SignedThreatReport::new(
            &self.identity,
            pattern,
            confidence,
            self.current_time,
        );
        
        // Add to own memory
        let _ = self.threats.add(
            report.clone(),
            &self.reputation,
            self.identity.id,
            self.current_time,
        );
        
        report
    }
    
    /// Gets current network health assessment.
    pub fn network_health(&self) -> NetworkHealth {
        let total_peers = self.peers.len();
        let active = self.peers.values().filter(|p| p.status == PeerStatus::Active).count();
        let suspicious = self.peers.values().filter(|p| p.status == PeerStatus::Suspicious).count();
        let isolated = self.peers.values().filter(|p| p.status == PeerStatus::Isolated).count();
        let active_threats = self.threats.active_threats(self.config.threat_threshold).len();
        let active_constraints = self.constraints.len();
        
        NetworkHealth {
            total_peers,
            active,
            suspicious,
            isolated,
            active_threats,
            active_constraints,
            // Empty network is healthy, otherwise require no isolated and few suspicious
            healthy: isolated == 0 && (total_peers == 0 || suspicious < (total_peers + 3) / 4),
        }
    }
    
    /// Gets peer count by status.
    pub fn peer_count(&self) -> (usize, usize, usize) {
        let active = self.peers.values().filter(|p| p.status == PeerStatus::Active).count();
        let suspicious = self.peers.values().filter(|p| p.status == PeerStatus::Suspicious).count();
        let isolated = self.peers.values().filter(|p| p.status == PeerStatus::Isolated).count();
        (active, suspicious, isolated)
    }
    
    /// Returns reference to reputation aggregator.
    pub fn reputation(&self) -> &ReputationAggregator {
        &self.reputation
    }
    
    /// Returns mutable reference to reputation aggregator.
    pub fn reputation_mut(&mut self) -> &mut ReputationAggregator {
        &mut self.reputation
    }
    
    /// Returns reference to threat memory.
    pub fn threats(&self) -> &ThreatMemory {
        &self.threats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_creation() {
        let node = AsiNode::new(NodeConfig::default());
        assert_ne!(node.id().0, [0u8; 32]);
    }
    
    #[test]
    fn test_peer_management() {
        let mut node1 = AsiNode::new(NodeConfig::default());
        let node2 = AsiNode::new(NodeConfig::default());
        
        node1.add_peer(node2.public_identity());
        
        assert!(node1.get_peer(node2.id()).is_some());
    }
    
    #[test]
    fn test_heartbeat_generation() {
        let mut node = AsiNode::new(NodeConfig::default());
        
        let messages = node.tick(10);
        
        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], ProtocolMessage::Heartbeat(_)));
    }
    
    #[test]
    fn test_threat_reporting() {
        let mut node = AsiNode::new(NodeConfig::default());
        
        let pattern = ThreatPattern::new(
            crate::threat::pattern::ThreatCategory::Deception,
            [1; 32],
            0.9,
        );
        
        let report = node.report_threat(pattern, 0.85);
        
        assert!(report.verify(&node.public_identity()));
    }
}
