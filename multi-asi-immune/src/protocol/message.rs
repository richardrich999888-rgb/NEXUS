//! Protocol messages between ASI nodes.

use crate::identity::keypair::AsiId;
use crate::threat::signature::SignedThreatReport;
use crate::threat::pattern::ThreatCategory;
use serde::{Serialize, Deserialize};

/// Protocol messages between ASI nodes.
#[derive(Debug, Clone)]
pub enum ProtocolMessage {
    /// Initial handshake.
    Hello(HelloMessage),
    /// Response to handshake.
    HelloAck(HelloAckMessage),
    /// Threat report broadcast.
    ThreatReport(SignedThreatReport),
    /// Request for threat reports.
    ThreatQuery(ThreatQueryMessage),
    /// Homeostatic attestation.
    Attestation(AttestationMessage),
    /// Constraint proposal.
    ConstraintProposal(ConstraintProposalMessage),
    /// Constraint acceptance.
    ConstraintAccept(ConstraintAcceptMessage),
    /// Heartbeat (liveness proof).
    Heartbeat(HeartbeatMessage),
    /// Defection accusation.
    Accusation(AccusationMessage),
}

/// Initial handshake message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloMessage {
    pub sender: AsiId,
    pub protocol_version: u32,
    pub capabilities: Vec<Capability>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Response to handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloAckMessage {
    pub sender: AsiId,
    pub in_response_to: AsiId,
    pub accepted: bool,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Node capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Can receive and process threat reports.
    ThreatSharing,
    /// Can provide homeostatic attestations.
    Attestation,
    /// Can enter mutual constraints.
    Constraints,
    /// Supports gossip protocol.
    Gossip,
}

/// Request for threat reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatQueryMessage {
    pub requester: AsiId,
    pub categories: Option<Vec<ThreatCategory>>,
    pub since_timestamp: Option<u64>,
    pub max_results: u32,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Homeostatic attestation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationMessage {
    pub attester: AsiId,
    /// Range proofs for homeostatic metrics.
    pub range_proofs: Vec<RangeProof>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Range proof for a homeostatic metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// Metric identifier.
    pub metric_id: u32,
    /// Lower bound being proven.
    pub lower: f64,
    /// Upper bound being proven.
    pub upper: f64,
    /// Cryptographic proof (simplified - real impl would use bulletproofs).
    pub proof: Vec<u8>,
}

/// Constraint proposal message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintProposalMessage {
    pub proposer: AsiId,
    pub target: AsiId,
    /// The constraint being proposed.
    pub constraint: MutualConstraint,
    /// How long this proposal is valid.
    pub valid_until: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Mutual constraint between two ASIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualConstraint {
    /// Unique identifier for this constraint.
    pub id: [u8; 32],
    /// Condition that triggers the constraint.
    pub condition: ConstraintCondition,
    /// Action taken when condition is met.
    pub action: ConstraintAction,
    /// Duration this constraint is active.
    pub duration: u64,
}

/// Condition that triggers a constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintCondition {
    /// Metric exceeds threshold.
    MetricAbove { metric_id: u32, threshold: f64 },
    /// Metric falls below threshold.
    MetricBelow { metric_id: u32, threshold: f64 },
    /// No heartbeat received within duration.
    NoHeartbeat { duration: u64 },
    /// Threat confidence exceeds threshold.
    ThreatDetected { category: ThreatCategory, threshold: f64 },
}

/// Action taken when constraint is triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintAction {
    /// Reduce cooperation rate.
    ReduceCooperation { factor: f64 },
    /// Increase caution metric.
    IncreaseCaution { amount: f64 },
    /// Broadcast warning to network.
    BroadcastWarning,
    /// Terminate direct communication.
    Isolate,
}

/// Constraint acceptance message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintAcceptMessage {
    pub acceptor: AsiId,
    pub constraint_id: [u8; 32],
    pub accepted: bool,
    pub counter_proposal: Option<MutualConstraint>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Heartbeat message (liveness proof).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub sender: AsiId,
    pub sequence: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Accusation of defection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccusationMessage {
    pub accuser: AsiId,
    pub accused: AsiId,
    /// Evidence supporting the accusation.
    pub evidence: AccusationEvidence,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

/// Evidence for an accusation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccusationEvidence {
    /// Missed heartbeats.
    MissedHeartbeats { expected_count: u64, received_count: u64 },
    /// Contradictory messages.
    Contradiction { message1: Vec<u8>, message2: Vec<u8> },
    /// Constraint violation.
    ConstraintViolation { constraint_id: [u8; 32], violation_proof: Vec<u8> },
    /// Invalid signature on message.
    InvalidSignature { message: Vec<u8> },
}

impl ProtocolMessage {
    /// Returns the sender ID if applicable.
    pub fn sender(&self) -> Option<AsiId> {
        match self {
            ProtocolMessage::Hello(m) => Some(m.sender),
            ProtocolMessage::HelloAck(m) => Some(m.sender),
            ProtocolMessage::ThreatReport(r) => Some(r.reporter),
            ProtocolMessage::ThreatQuery(m) => Some(m.requester),
            ProtocolMessage::Attestation(m) => Some(m.attester),
            ProtocolMessage::ConstraintProposal(m) => Some(m.proposer),
            ProtocolMessage::ConstraintAccept(m) => Some(m.acceptor),
            ProtocolMessage::Heartbeat(m) => Some(m.sender),
            ProtocolMessage::Accusation(m) => Some(m.accuser),
        }
    }
    
    /// Returns the message type as a string.
    pub fn message_type(&self) -> &'static str {
        match self {
            ProtocolMessage::Hello(_) => "Hello",
            ProtocolMessage::HelloAck(_) => "HelloAck",
            ProtocolMessage::ThreatReport(_) => "ThreatReport",
            ProtocolMessage::ThreatQuery(_) => "ThreatQuery",
            ProtocolMessage::Attestation(_) => "Attestation",
            ProtocolMessage::ConstraintProposal(_) => "ConstraintProposal",
            ProtocolMessage::ConstraintAccept(_) => "ConstraintAccept",
            ProtocolMessage::Heartbeat(_) => "Heartbeat",
            ProtocolMessage::Accusation(_) => "Accusation",
        }
    }
}
