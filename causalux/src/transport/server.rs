//! WebSocket Server
//! 
//! Axum-based WebSocket server for accepting peer connections.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::message::*;
use super::peer::{PeerInfo, PeerManager, PeerState};
use crate::version_vector::VersionVector;
use crate::dag::CausalDAG;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub bind_address: String,
    /// Maximum connections
    pub max_connections: usize,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval_secs: u64,
    /// Protocol version
    pub protocol_version: u32,
    /// Node ID
    pub node_id: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8765".to_string(),
            max_connections: 100,
            heartbeat_interval_secs: 30,
            protocol_version: 1,
            node_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Sync server state
pub struct SyncServer {
    /// Configuration
    config: ServerConfig,
    /// Peer manager
    peers: Arc<RwLock<PeerManager>>,
    /// Sender channels to peers (by node_id)
    senders: Arc<RwLock<HashMap<String, mpsc::Sender<SyncMessage>>>>,
    /// The local DAG (for sync operations)
    dag: Arc<RwLock<CausalDAG>>,
}

impl SyncServer {
    /// Create a new sync server
    pub fn new(config: ServerConfig, dag: Arc<RwLock<CausalDAG>>) -> Self {
        Self {
            peers: Arc::new(RwLock::new(PeerManager::new(config.node_id.clone()))),
            senders: Arc::new(RwLock::new(HashMap::new())),
            config,
            dag,
        }
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.config.node_id
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.connected_count()
    }

    /// Handle incoming message from a peer
    pub async fn handle_message(
        &self,
        from_node: &str,
        message: SyncMessage,
    ) -> Option<SyncMessage> {
        match message.message {
            MessageType::Hello(hello) => {
                self.handle_hello(from_node, &message.id, hello).await
            }
            MessageType::SyncRequest(req) => {
                self.handle_sync_request(from_node, &message.id, req).await
            }
            MessageType::Operation(op) => {
                self.handle_operation(from_node, &message.id, op).await
            }
            MessageType::Ping(ts) => {
                Some(SyncMessage::response_to(&message.id, MessageType::Pong(ts)))
            }
            MessageType::Pong(ts) => {
                // Update latency
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let latency = (now - ts) as u32;
                
                let mut peers = self.peers.write().await;
                if let Some(peer) = peers.get_mut(from_node) {
                    peer.update_latency(latency);
                }
                None
            }
            MessageType::Goodbye(_) => {
                self.peers.write().await.mark_disconnected(from_node);
                None
            }
            _ => None,
        }
    }

    /// Handle Hello handshake
    async fn handle_hello(
        &self,
        from_node: &str,
        request_id: &str,
        hello: HelloPayload,
    ) -> Option<SyncMessage> {
        // Verify protocol version
        if hello.version != self.config.protocol_version {
            return Some(SyncMessage::response_to(
                request_id,
                MessageType::Error(ErrorPayload {
                    code: error_codes::VERSION_MISMATCH,
                    message: format!(
                        "Protocol version mismatch: expected {}, got {}",
                        self.config.protocol_version, hello.version
                    ),
                    details: None,
                }),
            ));
        }

        // Generate session token
        let session_token = uuid::Uuid::new_v4().to_string();

        // Register peer
        {
            let mut peers = self.peers.write().await;
            peers.mark_connected(from_node, hello.capabilities.clone(), session_token.clone());
        }

        // Get our version vector
        let our_vv = {
            let dag = self.dag.read().await;
            dag.get_version_vector().clone()
        };

        Some(SyncMessage::response_to(
            request_id,
            MessageType::Welcome(WelcomePayload {
                node_id: self.config.node_id.clone(),
                capabilities: vec!["sync".to_string(), "push".to_string()],
                session_token,
                version_summary: VersionVectorSummary::from(&our_vv),
            }),
        ))
    }

    /// Handle sync request
    async fn handle_sync_request(
        &self,
        from_node: &str,
        request_id: &str,
        req: SyncRequestPayload,
    ) -> Option<SyncMessage> {
        let dag = self.dag.read().await;
        let our_vv = dag.get_version_vector();

        // Find operations the requester doesn't have
        let ops = dag.get_operations_since(&req.version_vector);
        
        // Limit to max_ops if specified
        let (ops, has_more) = if let Some(max) = req.max_ops {
            if ops.len() > max {
                (ops.into_iter().take(max).collect(), true)
            } else {
                (ops, false)
            }
        } else {
            (ops, false)
        };

        // Update peer's version vector
        drop(dag);
        {
            let mut peers = self.peers.write().await;
            peers.update_version_vector(from_node, req.version_vector);
        }

        Some(SyncMessage::response_to(
            request_id,
            MessageType::SyncResponse(SyncResponsePayload {
                operations: ops,
                has_more,
                continuation: None,
                version_vector: our_vv.clone(),
            }),
        ))
    }

    /// Handle incoming operation
    async fn handle_operation(
        &self,
        _from_node: &str,
        request_id: &str,
        op: OperationPayload,
    ) -> Option<SyncMessage> {
        // Apply operation to DAG
        let result = {
            let mut dag = self.dag.write().await;
            dag.insert(op.operation.clone())
        };

        if op.require_ack {
            match result {
                Ok(_) => Some(SyncMessage::response_to(
                    request_id,
                    MessageType::OperationAck(OperationAckPayload {
                        operation_id: op.operation.id.clone(),
                        success: true,
                        error: None,
                    }),
                )),
                Err(e) => Some(SyncMessage::response_to(
                    request_id,
                    MessageType::OperationAck(OperationAckPayload {
                        operation_id: op.operation.id.clone(),
                        success: false,
                        error: Some(format!("{:?}", e)),
                    }),
                )),
            }
        } else {
            None
        }
    }

    /// Broadcast an operation to all connected peers
    pub async fn broadcast_operation(&self, op: crate::causal_op::CausalOp) {
        let msg = SyncMessage::new(MessageType::Operation(OperationPayload {
            operation: op,
            require_ack: false,
        }));

        let senders = self.senders.read().await;
        for (_, sender) in senders.iter() {
            let _ = sender.send(msg.clone()).await;
        }
    }

    /// Send a message to a specific peer
    pub async fn send_to(&self, node_id: &str, message: SyncMessage) -> bool {
        let senders = self.senders.read().await;
        if let Some(sender) = senders.get(node_id) {
            sender.send(message).await.is_ok()
        } else {
            false
        }
    }

    /// Register a sender channel for a peer
    pub async fn register_sender(&self, node_id: String, sender: mpsc::Sender<SyncMessage>) {
        self.senders.write().await.insert(node_id, sender);
    }

    /// Unregister a sender channel
    pub async fn unregister_sender(&self, node_id: &str) {
        self.senders.write().await.remove(node_id);
    }

    /// Get peers that need sync (they have more ops than us)
    pub async fn peers_to_sync_from(&self) -> Vec<String> {
        let our_vv = self.dag.read().await.get_version_vector().clone();
        let peers = self.peers.read().await;
        peers.peers_ahead_of(&our_vv)
            .iter()
            .map(|p| p.node_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conflict::ConflictPolicy;

    fn create_test_dag() -> Arc<RwLock<CausalDAG>> {
        Arc::new(RwLock::new(CausalDAG::new(
            "test".to_string(),
            1000,
            ConflictPolicy::LastWriterWins,
        )))
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let dag = create_test_dag();
        let server = SyncServer::new(config, dag);
        
        assert_eq!(server.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let config = ServerConfig::default();
        let dag = create_test_dag();
        let server = SyncServer::new(config, dag);
        
        let ping = SyncMessage::new(MessageType::Ping(12345));
        let response = server.handle_message("peer1", ping).await;
        
        assert!(response.is_some());
        match response.unwrap().message {
            MessageType::Pong(ts) => assert_eq!(ts, 12345),
            _ => panic!("Expected Pong"),
        }
    }
}
