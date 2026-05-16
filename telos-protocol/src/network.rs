//! P2P Network layer for multi-node validator communication.
//!
//! Provides message types, peer management, and attestation request/response protocol.

use crate::validator::{ValidatorId, Attestation, AttestationType};
use crate::membrane::CommitmentProof;
use crate::error::{TelosError, TelosResult};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Message Types
// ============================================================================

/// Protocol message types for validator communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Handshake to establish connection.
    Handshake(HandshakeMsg),
    /// Response to handshake.
    HandshakeAck(HandshakeAckMsg),
    /// Request attestation for a commitment.
    AttestationRequest(AttestationRequest),
    /// Response to attestation request.
    AttestationResponse(AttestationResponse),
    /// Broadcast slashing evidence.
    SlashingEvidence(SlashingEvidence),
    /// Heartbeat to maintain connection.
    Heartbeat(HeartbeatMsg),
    /// Sync request for missed blocks.
    SyncRequest(SyncRequest),
    /// Sync response with blocks.
    SyncResponse(SyncResponse),
}

/// Handshake message to initiate connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMsg {
    /// Sender's validator ID.
    pub validator_id: ValidatorId,
    /// Protocol version.
    pub protocol_version: u32,
    /// Supported domains.
    pub supported_domains: Vec<String>,
    /// Current chain height.
    pub chain_height: u64,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Nonce for replay protection.
    pub nonce: [u8; 32],
}

/// Handshake acknowledgment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeAckMsg {
    /// Responder's validator ID.
    pub validator_id: ValidatorId,
    /// Accepted protocol version.
    pub protocol_version: u32,
    /// Chain height.
    pub chain_height: u64,
    /// Echo nonce.
    pub nonce_echo: [u8; 32],
    /// New nonce for session.
    pub session_nonce: [u8; 32],
}

/// Request for attestation on a commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationRequest {
    /// Unique request ID.
    pub request_id: String,
    /// Decision ID to attest.
    pub decision_id: String,
    /// Decision hash.
    pub decision_hash: [u8; 32],
    /// Decision domain.
    pub domain: String,
    /// Consequence tier.
    pub consequence_tier: u8,
    /// Entropy proof hash.
    pub entropy_proof_hash: [u8; 32],
    /// Authority chain (validator IDs).
    pub authority_chain: Vec<String>,
    /// Timeout in seconds.
    pub timeout_secs: u32,
    /// Requesting agent.
    pub agent_id: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

impl AttestationRequest {
    /// Compute request hash for signing.
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.request_id.as_bytes());
        hasher.update(self.decision_id.as_bytes());
        hasher.update(&self.decision_hash);
        hasher.update(self.domain.as_bytes());
        hasher.update(&[self.consequence_tier]);
        hasher.update(&self.entropy_proof_hash);
        hasher.update(self.timestamp.timestamp().to_le_bytes());
        hasher.finalize().into()
    }
}

/// Response to attestation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResponse {
    /// Request ID being responded to.
    pub request_id: String,
    /// Validator who is attesting.
    pub validator_id: ValidatorId,
    /// Attestation type.
    pub attestation_type: AttestationType,
    /// Reason for rejection (if rejected).
    pub reason: Option<String>,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Signature over response.
    pub signature: Vec<u8>,
}

impl AttestationResponse {
    /// Convert to Attestation struct.
    pub fn to_attestation(&self, decision_id: &str) -> Attestation {
        Attestation {
            validator_id: self.validator_id.clone(),
            decision_id: decision_id.to_string(),
            attestation_type: self.attestation_type,
            reason: self.reason.clone(),
            attested_at: self.timestamp,
            signature: self.signature.clone(),
        }
    }
}

/// Evidence of validator misbehavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    /// Evidence ID.
    pub evidence_id: String,
    /// Validator being accused.
    pub accused_validator: ValidatorId,
    /// Type of offense.
    pub offense_type: SlashingOffense,
    /// Proof data depending on offense type.
    pub proof: SlashingProof,
    /// Reporter.
    pub reporter: ValidatorId,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// Types of slashable offenses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingOffense {
    /// Validator was offline too long.
    Downtime,
    /// Validator attested incorrectly.
    FalseAttestation,
    /// Validator double-signed.
    DoubleAttestation,
    /// Validator colluded with others.
    Collusion,
}

/// Proof for slashing evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingProof {
    /// Proof of downtime (missed attestations).
    Downtime {
        missed_requests: Vec<String>,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    },
    /// Proof of false attestation.
    FalseAttestation {
        request: AttestationRequest,
        response: AttestationResponse,
        correct_result: bool,
    },
    /// Proof of double attestation.
    DoubleAttestation {
        request_id: String,
        attestation_1: AttestationResponse,
        attestation_2: AttestationResponse,
    },
}

/// Heartbeat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMsg {
    /// Sender's validator ID.
    pub validator_id: ValidatorId,
    /// Current chain height.
    pub chain_height: u64,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// Request to sync blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Starting height.
    pub from_height: u64,
    /// Ending height (inclusive).
    pub to_height: u64,
}

/// Response with blocks for sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Block hashes and heights.
    pub blocks: Vec<(u64, [u8; 32])>,
}

// ============================================================================
// Peer Management
// ============================================================================

/// State of a peer connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Initial state.
    Disconnected,
    /// Handshake sent, waiting for ack.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Connection failed or errored.
    Failed,
}

/// A connected peer.
#[derive(Debug, Clone)]
pub struct Peer {
    /// Validator ID of the peer.
    pub validator_id: ValidatorId,
    /// Connection state.
    pub state: PeerState,
    /// Supported domains.
    pub supported_domains: Vec<String>,
    /// Peer's chain height.
    pub chain_height: u64,
    /// Last seen timestamp.
    pub last_seen: DateTime<Utc>,
    /// Session nonce.
    pub session_nonce: [u8; 32],
    /// Pending attestation requests.
    pub pending_requests: HashSet<String>,
}

impl Peer {
    /// Create a new peer from handshake.
    pub fn from_handshake(msg: &HandshakeMsg) -> Self {
        Self {
            validator_id: msg.validator_id.clone(),
            state: PeerState::Connecting,
            supported_domains: msg.supported_domains.clone(),
            chain_height: msg.chain_height,
            last_seen: Utc::now(),
            session_nonce: [0u8; 32],
            pending_requests: HashSet::new(),
        }
    }

    /// Mark as connected.
    pub fn connect(&mut self, session_nonce: [u8; 32]) {
        self.state = PeerState::Connected;
        self.session_nonce = session_nonce;
        self.last_seen = Utc::now();
    }

    /// Update from heartbeat.
    pub fn update_heartbeat(&mut self, msg: &HeartbeatMsg) {
        self.chain_height = msg.chain_height;
        self.last_seen = Utc::now();
    }

    /// Check if peer supports a domain.
    pub fn supports_domain(&self, domain: &str) -> bool {
        self.supported_domains.iter().any(|d| {
            d == "*" || domain.starts_with(d)
        })
    }
}

// ============================================================================
// Network Coordinator
// ============================================================================

/// Coordinates P2P network operations.
#[derive(Debug)]
pub struct NetworkCoordinator {
    /// This node's validator ID.
    local_validator: ValidatorId,
    /// Connected peers.
    peers: HashMap<ValidatorId, Peer>,
    /// Pending attestation requests sent.
    pending_requests: HashMap<String, PendingRequest>,
    /// Received slashing evidence.
    slashing_evidence: Vec<SlashingEvidence>,
    /// Protocol version.
    protocol_version: u32,
    /// Heartbeat interval in seconds.
    heartbeat_interval_secs: u64,
}

/// A pending attestation request.
#[derive(Debug, Clone)]
pub struct PendingRequest {
    /// The request.
    pub request: AttestationRequest,
    /// Validators we sent to.
    pub sent_to: HashSet<ValidatorId>,
    /// Responses received.
    pub responses: Vec<AttestationResponse>,
    /// Created at.
    pub created_at: DateTime<Utc>,
}

impl NetworkCoordinator {
    /// Protocol version.
    pub const PROTOCOL_VERSION: u32 = 1;
    
    /// Default heartbeat interval.
    pub const HEARTBEAT_INTERVAL: u64 = 30;

    /// Create a new network coordinator.
    pub fn new(local_validator: ValidatorId) -> Self {
        Self {
            local_validator,
            peers: HashMap::new(),
            pending_requests: HashMap::new(),
            slashing_evidence: Vec::new(),
            protocol_version: Self::PROTOCOL_VERSION,
            heartbeat_interval_secs: Self::HEARTBEAT_INTERVAL,
        }
    }

    /// Generate a handshake message.
    pub fn create_handshake(&self, chain_height: u64, domains: Vec<String>) -> HandshakeMsg {
        let mut nonce = [0u8; 32];
        // Generate nonce from UUID and timestamp
        let uuid_bytes = uuid::Uuid::new_v4();
        let ts_bytes = Utc::now().timestamp_nanos_opt().unwrap_or(0).to_le_bytes();
        nonce[..16].copy_from_slice(uuid_bytes.as_bytes());
        nonce[16..24].copy_from_slice(&ts_bytes);
        // Fill remaining with hash
        let mut hasher = Sha256::new();
        hasher.update(uuid_bytes.as_bytes());
        hasher.update(&ts_bytes);
        let hash = hasher.finalize();
        nonce[24..32].copy_from_slice(&hash[..8]);
        
        HandshakeMsg {
            validator_id: self.local_validator.clone(),
            protocol_version: self.protocol_version,
            supported_domains: domains,
            chain_height,
            timestamp: Utc::now(),
            nonce,
        }
    }

    /// Handle incoming handshake.
    pub fn handle_handshake(&mut self, msg: HandshakeMsg, our_height: u64) -> HandshakeAckMsg {
        let peer = Peer::from_handshake(&msg);
        self.peers.insert(msg.validator_id.clone(), peer);
        
        let mut session_nonce = [0u8; 32];
        let mut hasher = Sha256::new();
        hasher.update(&msg.nonce);
        hasher.update(Utc::now().timestamp().to_le_bytes());
        session_nonce.copy_from_slice(&hasher.finalize());
        
        HandshakeAckMsg {
            validator_id: self.local_validator.clone(),
            protocol_version: self.protocol_version,
            chain_height: our_height,
            nonce_echo: msg.nonce,
            session_nonce,
        }
    }

    /// Handle handshake acknowledgment.
    pub fn handle_handshake_ack(&mut self, msg: HandshakeAckMsg) -> TelosResult<()> {
        let peer = self.peers.get_mut(&msg.validator_id)
            .ok_or_else(|| TelosError::ValidatorNotFound(msg.validator_id.0.clone()))?;
        
        peer.connect(msg.session_nonce);
        peer.chain_height = msg.chain_height;
        Ok(())
    }

    /// Create attestation request.
    pub fn create_attestation_request(
        &mut self,
        decision_id: &str,
        decision_hash: [u8; 32],
        domain: &str,
        consequence_tier: u8,
        entropy_proof_hash: [u8; 32],
        authority_chain: Vec<String>,
        agent_id: &str,
    ) -> AttestationRequest {
        let request_id = uuid::Uuid::new_v4().to_string();
        
        AttestationRequest {
            request_id,
            decision_id: decision_id.to_string(),
            decision_hash,
            domain: domain.to_string(),
            consequence_tier,
            entropy_proof_hash,
            authority_chain,
            timeout_secs: 30,
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
        }
    }

    /// Send attestation request to eligible peers.
    pub fn broadcast_attestation_request(&mut self, request: AttestationRequest) -> Vec<ValidatorId> {
        let mut sent_to = Vec::new();
        
        for (vid, peer) in &mut self.peers {
            if peer.state == PeerState::Connected && peer.supports_domain(&request.domain) {
                peer.pending_requests.insert(request.request_id.clone());
                sent_to.push(vid.clone());
            }
        }
        
        if !sent_to.is_empty() {
            self.pending_requests.insert(request.request_id.clone(), PendingRequest {
                request,
                sent_to: sent_to.iter().cloned().collect(),
                responses: Vec::new(),
                created_at: Utc::now(),
            });
        }
        
        sent_to
    }

    /// Handle attestation response.
    pub fn handle_attestation_response(&mut self, response: AttestationResponse) -> Option<&PendingRequest> {
        if let Some(pending) = self.pending_requests.get_mut(&response.request_id) {
            pending.responses.push(response.clone());
            
            // Remove from peer's pending
            if let Some(peer) = self.peers.get_mut(&response.validator_id) {
                peer.pending_requests.remove(&response.request_id);
                peer.last_seen = Utc::now();
            }
            
            return Some(pending);
        }
        None
    }

    /// Handle slashing evidence.
    pub fn handle_slashing_evidence(&mut self, evidence: SlashingEvidence) {
        // Verify the evidence is not duplicate
        if !self.slashing_evidence.iter().any(|e| e.evidence_id == evidence.evidence_id) {
            self.slashing_evidence.push(evidence);
        }
    }

    /// Get connected peers.
    pub fn connected_peers(&self) -> impl Iterator<Item = &Peer> {
        self.peers.values().filter(|p| p.state == PeerState::Connected)
    }

    /// Get peer count.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get connected count.
    pub fn connected_count(&self) -> usize {
        self.connected_peers().count()
    }

    /// Get peers that support a domain.
    pub fn peers_for_domain(&self, domain: &str) -> Vec<&Peer> {
        self.connected_peers()
            .filter(|p| p.supports_domain(domain))
            .collect()
    }

    /// Check for timed out requests.
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let now = Utc::now();
        let mut timed_out = Vec::new();
        
        for (request_id, pending) in &self.pending_requests {
            let elapsed = (now - pending.created_at).num_seconds();
            if elapsed > pending.request.timeout_secs as i64 {
                timed_out.push(request_id.clone());
            }
        }
        
        timed_out
    }

    /// Remove completed or timed out request.
    pub fn remove_request(&mut self, request_id: &str) -> Option<PendingRequest> {
        self.pending_requests.remove(request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_flow() {
        let mut node1 = NetworkCoordinator::new(ValidatorId::new("node1"));
        let mut node2 = NetworkCoordinator::new(ValidatorId::new("node2"));

        // Node1 initiates handshake
        let handshake = node1.create_handshake(100, vec!["*".into()]);
        
        // Node2 receives and responds
        let ack = node2.handle_handshake(handshake.clone(), 100);
        
        // Verify node2 added node1 as peer
        assert!(node2.peers.contains_key(&ValidatorId::new("node1")));
        
        // Node1 handles ack
        // (need to first add node2 as peer for this to work)
        let handshake2 = node2.create_handshake(100, vec!["*".into()]);
        node1.handle_handshake(handshake2, 100);
        node1.handle_handshake_ack(ack).unwrap();
        
        // Verify node1 has node2 connected
        let peer = node1.peers.get(&ValidatorId::new("node2")).unwrap();
        assert_eq!(peer.state, PeerState::Connected);
    }

    #[test]
    fn test_attestation_request_broadcast() {
        let mut coordinator = NetworkCoordinator::new(ValidatorId::new("local"));
        
        // Add some connected peers
        let mut peer1 = Peer::from_handshake(&HandshakeMsg {
            validator_id: ValidatorId::new("peer1"),
            protocol_version: 1,
            supported_domains: vec!["finance".into()],
            chain_height: 100,
            timestamp: Utc::now(),
            nonce: [0u8; 32],
        });
        peer1.connect([1u8; 32]);
        
        let mut peer2 = Peer::from_handshake(&HandshakeMsg {
            validator_id: ValidatorId::new("peer2"),
            protocol_version: 1,
            supported_domains: vec!["*".into()],
            chain_height: 100,
            timestamp: Utc::now(),
            nonce: [0u8; 32],
        });
        peer2.connect([2u8; 32]);
        
        coordinator.peers.insert(ValidatorId::new("peer1"), peer1);
        coordinator.peers.insert(ValidatorId::new("peer2"), peer2);
        
        // Create and broadcast request
        let request = coordinator.create_attestation_request(
            "decision-1",
            [0u8; 32],
            "finance.trading",
            3,
            [0u8; 32],
            vec!["root".into()],
            "agent-1",
        );
        
        let sent_to = coordinator.broadcast_attestation_request(request.clone());
        
        // peer1 supports "finance" (prefix match), peer2 supports "*"
        assert_eq!(sent_to.len(), 2);
        assert!(coordinator.pending_requests.contains_key(&request.request_id));
    }

    #[test]
    fn test_domain_matching() {
        let peer = Peer {
            validator_id: ValidatorId::new("peer"),
            state: PeerState::Connected,
            supported_domains: vec!["finance".into(), "healthcare".into()],
            chain_height: 100,
            last_seen: Utc::now(),
            session_nonce: [0u8; 32],
            pending_requests: HashSet::new(),
        };
        
        assert!(peer.supports_domain("finance"));
        assert!(peer.supports_domain("finance.trading"));
        assert!(peer.supports_domain("healthcare.records"));
        assert!(!peer.supports_domain("government"));
    }

    #[test]
    fn test_attestation_response_handling() {
        let mut coordinator = NetworkCoordinator::new(ValidatorId::new("local"));
        
        // Add peer
        let mut peer = Peer::from_handshake(&HandshakeMsg {
            validator_id: ValidatorId::new("peer1"),
            protocol_version: 1,
            supported_domains: vec!["*".into()],
            chain_height: 100,
            timestamp: Utc::now(),
            nonce: [0u8; 32],
        });
        peer.connect([1u8; 32]);
        coordinator.peers.insert(ValidatorId::new("peer1"), peer);
        
        // Create request
        let request = coordinator.create_attestation_request(
            "d1", [0u8; 32], "test", 1, [0u8; 32], vec![], "agent"
        );
        let request_id = request.request_id.clone();
        coordinator.broadcast_attestation_request(request);
        
        // Handle response
        let response = AttestationResponse {
            request_id: request_id.clone(),
            validator_id: ValidatorId::new("peer1"),
            attestation_type: AttestationType::Approve,
            reason: None,
            timestamp: Utc::now(),
            signature: vec![],
        };
        
        let pending = coordinator.handle_attestation_response(response);
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().responses.len(), 1);
    }
}
