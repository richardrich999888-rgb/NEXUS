//! Peer Management
//! 
//! Tracks connected peers, their state, and capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::version_vector::VersionVector;

/// State of a peer connection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerState {
    /// Connecting (handshake in progress)
    Connecting,
    /// Connected and ready
    Connected,
    /// Syncing data
    Syncing,
    /// Connection lost, will retry
    Reconnecting,
    /// Permanently disconnected
    Disconnected,
}

/// Information about a connected peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer's node ID
    pub node_id: String,
    /// Connection address
    pub address: String,
    /// Current state
    pub state: PeerState,
    /// Peer's capabilities
    pub capabilities: Vec<String>,
    /// Session token
    pub session_token: Option<String>,
    /// Last known version vector
    pub version_vector: Option<VersionVector>,
    /// Last activity time
    pub last_seen: Instant,
    /// Last ping latency
    pub latency_ms: Option<u32>,
    /// Connection attempt count
    pub connect_attempts: u32,
    /// Whether this is an outbound connection
    pub outbound: bool,
}

impl PeerInfo {
    /// Create a new peer info for an outbound connection
    pub fn new_outbound(node_id: String, address: String) -> Self {
        Self {
            node_id,
            address,
            state: PeerState::Connecting,
            capabilities: Vec::new(),
            session_token: None,
            version_vector: None,
            last_seen: Instant::now(),
            latency_ms: None,
            connect_attempts: 1,
            outbound: true,
        }
    }

    /// Create a new peer info for an inbound connection
    pub fn new_inbound(node_id: String, address: String) -> Self {
        Self {
            node_id,
            address,
            state: PeerState::Connecting,
            capabilities: Vec::new(),
            session_token: None,
            version_vector: None,
            last_seen: Instant::now(),
            latency_ms: None,
            connect_attempts: 0,
            outbound: false,
        }
    }

    /// Update last seen time
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Check if peer is stale (no activity for duration)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }

    /// Check if peer is connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state, PeerState::Connected | PeerState::Syncing)
    }

    /// Set connected state after successful handshake
    pub fn set_connected(&mut self, capabilities: Vec<String>, session_token: String) {
        self.state = PeerState::Connected;
        self.capabilities = capabilities;
        self.session_token = Some(session_token);
        self.touch();
    }

    /// Update latency from ping/pong
    pub fn update_latency(&mut self, latency_ms: u32) {
        self.latency_ms = Some(latency_ms);
        self.touch();
    }
}

/// Manages all peer connections
#[derive(Debug)]
pub struct PeerManager {
    /// Our node ID
    node_id: String,
    /// Connected peers by node ID
    peers: HashMap<String, PeerInfo>,
    /// Maximum peers to maintain
    max_peers: usize,
    /// Heartbeat interval
    heartbeat_interval: Duration,
    /// Connection timeout
    connection_timeout: Duration,
    /// Stale peer timeout
    stale_timeout: Duration,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            peers: HashMap::new(),
            max_peers: 50,
            heartbeat_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            stale_timeout: Duration::from_secs(120),
        }
    }

    /// Configure max peers
    pub fn with_max_peers(mut self, max: usize) -> Self {
        self.max_peers = max;
        self
    }

    /// Add a new outbound peer
    pub fn add_outbound(&mut self, node_id: String, address: String) -> bool {
        if self.peers.len() >= self.max_peers {
            return false;
        }
        if self.peers.contains_key(&node_id) {
            return false;
        }
        self.peers.insert(node_id.clone(), PeerInfo::new_outbound(node_id, address));
        true
    }

    /// Add a new inbound peer
    pub fn add_inbound(&mut self, node_id: String, address: String) -> bool {
        if self.peers.len() >= self.max_peers {
            return false;
        }
        if self.peers.contains_key(&node_id) {
            return false;
        }
        self.peers.insert(node_id.clone(), PeerInfo::new_inbound(node_id, address));
        true
    }

    /// Get a peer by node ID
    pub fn get(&self, node_id: &str) -> Option<&PeerInfo> {
        self.peers.get(node_id)
    }

    /// Get a mutable peer by node ID
    pub fn get_mut(&mut self, node_id: &str) -> Option<&mut PeerInfo> {
        self.peers.get_mut(node_id)
    }

    /// Remove a peer
    pub fn remove(&mut self, node_id: &str) -> Option<PeerInfo> {
        self.peers.remove(node_id)
    }

    /// Get all connected peers
    pub fn connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().filter(|p| p.is_connected()).collect()
    }

    /// Get all peer IDs
    pub fn peer_ids(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    /// Get connected peer count
    pub fn connected_count(&self) -> usize {
        self.connected_peers().len()
    }

    /// Get total peer count
    pub fn total_count(&self) -> usize {
        self.peers.len()
    }

    /// Mark peer as connected after handshake
    pub fn mark_connected(
        &mut self,
        node_id: &str,
        capabilities: Vec<String>,
        session_token: String,
    ) -> bool {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.set_connected(capabilities, session_token);
            true
        } else {
            false
        }
    }

    /// Update peer's version vector
    pub fn update_version_vector(&mut self, node_id: &str, vv: VersionVector) {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.version_vector = Some(vv);
            peer.touch();
        }
    }

    /// Mark peer as disconnected
    pub fn mark_disconnected(&mut self, node_id: &str) {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.state = PeerState::Disconnected;
        }
    }

    /// Mark peer as reconnecting
    pub fn mark_reconnecting(&mut self, node_id: &str) {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.state = PeerState::Reconnecting;
            peer.connect_attempts += 1;
        }
    }

    /// Clean up stale peers
    pub fn cleanup_stale(&mut self) -> Vec<String> {
        let stale: Vec<String> = self.peers
            .iter()
            .filter(|(_, p)| p.is_stale(self.stale_timeout))
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &stale {
            self.peers.remove(id);
        }
        
        stale
    }

    /// Get peers that need heartbeat
    pub fn needs_heartbeat(&self) -> Vec<String> {
        self.peers
            .iter()
            .filter(|(_, p)| {
                p.is_connected() && p.last_seen.elapsed() > self.heartbeat_interval
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get peers sorted by latency (best first)
    pub fn peers_by_latency(&self) -> Vec<&PeerInfo> {
        let mut connected: Vec<_> = self.connected_peers();
        connected.sort_by_key(|p| p.latency_ms.unwrap_or(u32::MAX));
        connected
    }

    /// Get peers that have operations we don't have
    pub fn peers_ahead_of(&self, our_vv: &VersionVector) -> Vec<&PeerInfo> {
        self.connected_peers()
            .into_iter()
            .filter(|p| {
                if let Some(their_vv) = &p.version_vector {
                    their_vv.total_operations() > our_vv.total_operations()
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Statistics about peer connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStats {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub average_latency_ms: Option<u32>,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info() {
        let mut peer = PeerInfo::new_outbound("node1".to_string(), "ws://localhost:8080".to_string());
        assert_eq!(peer.state, PeerState::Connecting);
        assert!(peer.outbound);
        
        peer.set_connected(vec!["sync".to_string()], "token123".to_string());
        assert_eq!(peer.state, PeerState::Connected);
        assert!(peer.is_connected());
    }

    #[test]
    fn test_peer_manager() {
        let mut mgr = PeerManager::new("self".to_string());
        
        assert!(mgr.add_outbound("peer1".to_string(), "addr1".to_string()));
        assert!(mgr.add_outbound("peer2".to_string(), "addr2".to_string()));
        
        assert_eq!(mgr.total_count(), 2);
        assert_eq!(mgr.connected_count(), 0);
        
        mgr.mark_connected("peer1", vec![], "token".to_string());
        assert_eq!(mgr.connected_count(), 1);
    }

    #[test]
    fn test_max_peers() {
        let mut mgr = PeerManager::new("self".to_string()).with_max_peers(2);
        
        assert!(mgr.add_outbound("peer1".to_string(), "addr1".to_string()));
        assert!(mgr.add_outbound("peer2".to_string(), "addr2".to_string()));
        assert!(!mgr.add_outbound("peer3".to_string(), "addr3".to_string())); // Should fail
    }
}
