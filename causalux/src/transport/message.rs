//! Wire Protocol Messages
//! 
//! Defines the message format for CAUSALUX network communication.

use serde::{Deserialize, Serialize};
use crate::version_vector::VersionVector;
use crate::causal_op::CausalOp;

/// Message types for the sync protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MessageType {
    /// Handshake: Initial connection setup
    Hello(HelloPayload),
    /// Handshake response
    Welcome(WelcomePayload),
    
    /// Request sync with version vector
    SyncRequest(SyncRequestPayload),
    /// Response with operations
    SyncResponse(SyncResponsePayload),
    
    /// Push a single operation
    Operation(OperationPayload),
    /// Acknowledge operation receipt
    OperationAck(OperationAckPayload),
    
    /// Request specific operations by ID
    FetchOps(FetchOpsPayload),
    /// Response with requested operations
    FetchOpsResponse(FetchOpsResponsePayload),
    
    /// Heartbeat to keep connection alive
    Ping(u64),  // timestamp
    /// Heartbeat response
    Pong(u64),  // echo timestamp
    
    /// Error message
    Error(ErrorPayload),
    
    /// Graceful disconnect
    Goodbye(String),  // reason
}

/// Hello handshake payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayload {
    /// Protocol version
    pub version: u32,
    /// Node ID
    pub node_id: String,
    /// Capabilities (features supported)
    pub capabilities: Vec<String>,
    /// Current version vector summary
    pub version_summary: VersionVectorSummary,
}

/// Welcome response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomePayload {
    /// Server's node ID
    pub node_id: String,
    /// Accepted capabilities
    pub capabilities: Vec<String>,
    /// Session token for this connection
    pub session_token: String,
    /// Server's version vector summary
    pub version_summary: VersionVectorSummary,
}

/// Compact version vector summary for handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionVectorSummary {
    /// Total operations across all nodes
    pub total_ops: u64,
    /// Number of known nodes
    pub node_count: usize,
    /// Hash of full version vector
    pub hash: String,
}

impl From<&VersionVector> for VersionVectorSummary {
    fn from(vv: &VersionVector) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", vv).as_bytes());
        Self {
            total_ops: vv.total_operations(),
            node_count: vv.node_ids().len(),
            hash: format!("{:x}", hasher.finalize())[..16].to_string(),
        }
    }
}

/// Sync request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequestPayload {
    /// Full version vector
    pub version_vector: VersionVector,
    /// Maximum operations to receive
    pub max_ops: Option<usize>,
    /// Specific document/key to sync (optional)
    pub scope: Option<String>,
}

/// Sync response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponsePayload {
    /// Operations the requester is missing
    pub operations: Vec<CausalOp>,
    /// Whether there are more operations available
    pub has_more: bool,
    /// Continuation token for pagination
    pub continuation: Option<String>,
    /// Server's current version vector
    pub version_vector: VersionVector,
}

/// Single operation push payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPayload {
    /// The operation
    pub operation: CausalOp,
    /// Request acknowledgment
    pub require_ack: bool,
}

/// Operation acknowledgment payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationAckPayload {
    /// Operation ID that was received
    pub operation_id: String,
    /// Whether it was successfully applied
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Fetch specific operations payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOpsPayload {
    /// Operation IDs to fetch
    pub operation_ids: Vec<String>,
}

/// Fetch operations response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOpsResponsePayload {
    /// Found operations
    pub operations: Vec<CausalOp>,
    /// IDs that were not found
    pub not_found: Vec<String>,
}

/// Error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Error code
    pub code: u32,
    /// Human-readable message
    pub message: String,
    /// Additional details
    pub details: Option<serde_json::Value>,
}

/// Error codes
pub mod error_codes {
    pub const UNKNOWN: u32 = 0;
    pub const PROTOCOL_ERROR: u32 = 1;
    pub const AUTH_REQUIRED: u32 = 2;
    pub const AUTH_FAILED: u32 = 3;
    pub const RATE_LIMITED: u32 = 4;
    pub const VERSION_MISMATCH: u32 = 5;
    pub const OPERATION_INVALID: u32 = 6;
    pub const INTERNAL_ERROR: u32 = 7;
}

/// Wire message with envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    /// Message ID for request/response correlation
    pub id: String,
    /// Timestamp
    pub timestamp: u64,
    /// The actual message
    pub message: MessageType,
}

impl SyncMessage {
    /// Create a new message with auto-generated ID
    pub fn new(message: MessageType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message,
        }
    }

    /// Create a response to a request
    pub fn response_to(request_id: &str, message: MessageType) -> Self {
        Self {
            id: request_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            message,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to binary (MessagePack for efficiency)
    pub fn to_bytes(&self) -> Vec<u8> {
        // For now, use JSON. Can switch to msgpack later
        self.to_json().unwrap_or_default().into_bytes()
    }

    /// Deserialize from binary
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::str::from_utf8(bytes)?;
        Ok(Self::from_json(json)?)
    }
}

/// Peer-to-peer message wrapper (includes routing info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMessage {
    /// Sender node ID
    pub from: String,
    /// Recipient node ID (empty for broadcast)
    pub to: Option<String>,
    /// Hop count (for loop detection)
    pub hops: u8,
    /// The sync message
    pub payload: SyncMessage,
}

impl PeerMessage {
    pub fn new(from: String, payload: SyncMessage) -> Self {
        Self {
            from,
            to: None,
            hops: 0,
            payload,
        }
    }

    pub fn to(mut self, recipient: String) -> Self {
        self.to = Some(recipient);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = SyncMessage::new(MessageType::Ping(12345));
        let json = msg.to_json().unwrap();
        let parsed = SyncMessage::from_json(&json).unwrap();
        
        assert_eq!(msg.id, parsed.id);
        match parsed.message {
            MessageType::Ping(ts) => assert_eq!(ts, 12345),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_hello_payload() {
        let hello = HelloPayload {
            version: 1,
            node_id: "node1".to_string(),
            capabilities: vec!["sync".to_string(), "push".to_string()],
            version_summary: VersionVectorSummary {
                total_ops: 100,
                node_count: 5,
                hash: "abc123".to_string(),
            },
        };
        
        let msg = SyncMessage::new(MessageType::Hello(hello));
        let json = msg.to_json().unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("node1"));
    }

    #[test]
    fn test_peer_message() {
        let sync_msg = SyncMessage::new(MessageType::Ping(1));
        let peer_msg = PeerMessage::new("sender".to_string(), sync_msg)
            .to("receiver".to_string());
        
        assert_eq!(peer_msg.from, "sender");
        assert_eq!(peer_msg.to, Some("receiver".to_string()));
    }
}
