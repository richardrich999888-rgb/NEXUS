//! WebSocket Client
//! 
//! Client for connecting to CAUSALUX sync servers.

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, oneshot};
use std::collections::HashMap;
use std::time::Duration;

use super::message::*;
use super::peer::{PeerInfo, PeerState};
use crate::version_vector::VersionVector;
use crate::causal_op::CausalOp;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Our node ID
    pub node_id: String,
    /// Protocol version
    pub protocol_version: u32,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Reconnect delay
    pub reconnect_delay: Duration,
    /// Max reconnect attempts
    pub max_reconnect_attempts: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            protocol_version: 1,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 10,
        }
    }
}

/// Pending request awaiting response
struct PendingRequest {
    response_tx: oneshot::Sender<SyncMessage>,
}

/// Sync client for connecting to servers
pub struct SyncClient {
    /// Configuration
    config: ClientConfig,
    /// Connected server info
    server: Arc<RwLock<Option<PeerInfo>>>,
    /// Session token after handshake
    session_token: Arc<RwLock<Option<String>>>,
    /// Sender to the connection
    sender: Arc<RwLock<Option<mpsc::Sender<SyncMessage>>>>,
    /// Pending requests by message ID
    pending: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// Current version vector
    version_vector: Arc<RwLock<VersionVector>>,
    /// Incoming operations channel
    operations_rx: Arc<RwLock<Option<mpsc::Receiver<CausalOp>>>>,
    operations_tx: mpsc::Sender<CausalOp>,
}

impl SyncClient {
    /// Create a new sync client
    pub fn new(config: ClientConfig) -> Self {
        let (ops_tx, ops_rx) = mpsc::channel(1000);
        Self {
            config,
            server: Arc::new(RwLock::new(None)),
            session_token: Arc::new(RwLock::new(None)),
            sender: Arc::new(RwLock::new(None)),
            pending: Arc::new(RwLock::new(HashMap::new())),
            version_vector: Arc::new(RwLock::new(VersionVector::new())),
            operations_rx: Arc::new(RwLock::new(Some(ops_rx))),
            operations_tx: ops_tx,
        }
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.config.node_id
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.server.read().await.as_ref().map(|s| s.is_connected()).unwrap_or(false)
    }

    /// Get session token
    pub async fn session_token(&self) -> Option<String> {
        self.session_token.read().await.clone()
    }

    /// Set the sender channel (called after WebSocket connection established)
    pub async fn set_sender(&self, sender: mpsc::Sender<SyncMessage>) {
        *self.sender.write().await = Some(sender);
    }

    /// Perform handshake with server
    pub async fn handshake(&self, server_address: String) -> Result<(), ClientError> {
        // Create hello message
        let vv = self.version_vector.read().await.clone();
        let hello = SyncMessage::new(MessageType::Hello(HelloPayload {
            version: self.config.protocol_version,
            node_id: self.config.node_id.clone(),
            capabilities: vec!["sync".to_string(), "push".to_string()],
            version_summary: VersionVectorSummary::from(&vv),
        }));

        // Send and wait for response
        let response = self.request(hello).await?;

        match response.message {
            MessageType::Welcome(welcome) => {
                *self.session_token.write().await = Some(welcome.session_token.clone());
                *self.server.write().await = Some(PeerInfo {
                    node_id: welcome.node_id,
                    address: server_address,
                    state: PeerState::Connected,
                    capabilities: welcome.capabilities,
                    session_token: Some(welcome.session_token),
                    version_vector: None,
                    last_seen: std::time::Instant::now(),
                    latency_ms: None,
                    connect_attempts: 1,
                    outbound: true,
                });
                Ok(())
            }
            MessageType::Error(err) => Err(ClientError::ServerError(err.message)),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Request sync from server
    pub async fn sync(&self, our_vv: &VersionVector) -> Result<Vec<CausalOp>, ClientError> {
        let request = SyncMessage::new(MessageType::SyncRequest(SyncRequestPayload {
            version_vector: our_vv.clone(),
            max_ops: Some(1000),
            scope: None,
        }));

        let response = self.request(request).await?;

        match response.message {
            MessageType::SyncResponse(resp) => {
                // Update our knowledge of server's version vector
                if let Some(server) = self.server.write().await.as_mut() {
                    server.version_vector = Some(resp.version_vector);
                }
                Ok(resp.operations)
            }
            MessageType::Error(err) => Err(ClientError::ServerError(err.message)),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Push an operation to server
    pub async fn push_operation(&self, op: CausalOp, require_ack: bool) -> Result<bool, ClientError> {
        let request = SyncMessage::new(MessageType::Operation(OperationPayload {
            operation: op.clone(),
            require_ack,
        }));

        if require_ack {
            let response = self.request(request).await?;
            match response.message {
                MessageType::OperationAck(ack) => Ok(ack.success),
                MessageType::Error(err) => Err(ClientError::ServerError(err.message)),
                _ => Err(ClientError::UnexpectedResponse),
            }
        } else {
            self.send(request).await?;
            Ok(true)
        }
    }

    /// Send a ping and measure latency
    pub async fn ping(&self) -> Result<u32, ClientError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let request = SyncMessage::new(MessageType::Ping(now));
        let response = self.request(request).await?;

        match response.message {
            MessageType::Pong(sent_ts) => {
                let received = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let latency = (received - sent_ts) as u32;
                
                if let Some(server) = self.server.write().await.as_mut() {
                    server.latency_ms = Some(latency);
                }
                
                Ok(latency)
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Disconnect from server
    pub async fn disconnect(&self, reason: &str) -> Result<(), ClientError> {
        let msg = SyncMessage::new(MessageType::Goodbye(reason.to_string()));
        self.send(msg).await?;
        
        *self.server.write().await = None;
        *self.session_token.write().await = None;
        *self.sender.write().await = None;
        
        Ok(())
    }

    /// Handle incoming message from server
    pub async fn handle_message(&self, message: SyncMessage) {
        // Check if this is a response to a pending request
        {
            let mut pending = self.pending.write().await;
            if let Some(req) = pending.remove(&message.id) {
                let _ = req.response_tx.send(message);
                return;
            }
        }

        // Handle unsolicited messages
        match message.message {
            MessageType::Operation(op) => {
                let _ = self.operations_tx.send(op.operation).await;
            }
            MessageType::Ping(ts) => {
                // Respond with pong
                let _ = self.send(SyncMessage::response_to(&message.id, MessageType::Pong(ts))).await;
            }
            _ => {}
        }
    }

    /// Take the operations receiver (for processing incoming ops)
    pub async fn take_operations_receiver(&self) -> Option<mpsc::Receiver<CausalOp>> {
        self.operations_rx.write().await.take()
    }

    /// Send a message without waiting for response
    async fn send(&self, message: SyncMessage) -> Result<(), ClientError> {
        let sender = self.sender.read().await;
        match sender.as_ref() {
            Some(tx) => tx.send(message).await.map_err(|_| ClientError::Disconnected),
            None => Err(ClientError::NotConnected),
        }
    }

    /// Send a request and wait for response
    async fn request(&self, message: SyncMessage) -> Result<SyncMessage, ClientError> {
        let (tx, rx) = oneshot::channel();
        let msg_id = message.id.clone();

        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(msg_id.clone(), PendingRequest { response_tx: tx });
        }

        // Send message
        self.send(message).await?;

        // Wait for response with timeout
        match tokio::time::timeout(self.config.request_timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                self.pending.write().await.remove(&msg_id);
                Err(ClientError::RequestCancelled)
            }
            Err(_) => {
                self.pending.write().await.remove(&msg_id);
                Err(ClientError::Timeout)
            }
        }
    }
}

/// Client errors
#[derive(Debug, Clone)]
pub enum ClientError {
    NotConnected,
    Disconnected,
    Timeout,
    RequestCancelled,
    UnexpectedResponse,
    ServerError(String),
    ConnectionFailed(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::NotConnected => write!(f, "Not connected to server"),
            ClientError::Disconnected => write!(f, "Disconnected from server"),
            ClientError::Timeout => write!(f, "Request timed out"),
            ClientError::RequestCancelled => write!(f, "Request was cancelled"),
            ClientError::UnexpectedResponse => write!(f, "Unexpected response from server"),
            ClientError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ClientError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = ClientConfig::default();
        let client = SyncClient::new(config);
        
        assert!(!client.is_connected().await);
        assert!(client.session_token().await.is_none());
    }
}
