//! Rate Limiting and DoS Protection
//!
//! Implements per-peer connection limits and rate limiting to prevent
//! resource exhaustion attacks.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use dashmap::DashMap;
use tracing::{warn, debug};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum connections per peer
    pub max_connections_per_peer: usize,
    /// Maximum messages per second per peer
    pub max_messages_per_second: u32,
    /// Maximum bytes per second per peer
    pub max_bytes_per_second: u64,
    /// Time window for rate limiting (seconds)
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_connections_per_peer: 10,
            max_messages_per_second: 100,
            max_bytes_per_second: 10 * 1024 * 1024, // 10 MB/s
            window_seconds: 1,
        }
    }
}

/// Per-peer rate limit state
#[derive(Debug)]
struct PeerRateLimit {
    /// Connection count
    connections: usize,
    /// Message timestamps in current window
    message_timestamps: Vec<Instant>,
    /// Bytes transferred in current window
    bytes_in_window: u64,
    /// Window start time
    window_start: Instant,
}

impl PeerRateLimit {
    fn new() -> Self {
        Self {
            connections: 0,
            message_timestamps: Vec::new(),
            bytes_in_window: 0,
            window_start: Instant::now(),
        }
    }

    fn reset_window(&mut self) {
        self.message_timestamps.clear();
        self.bytes_in_window = 0;
        self.window_start = Instant::now();
    }
}

/// Rate limiter for network connections
pub struct RateLimiter {
    config: RateLimitConfig,
    peers: Arc<DashMap<SocketAddr, RwLock<PeerRateLimit>>>,
}

impl RateLimiter {
    /// Create new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create new rate limiter with custom configuration
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            peers: Arc::new(DashMap::new()),
        }
    }

    /// Check if a new connection from this peer is allowed
    pub fn allow_connection(&self, peer: SocketAddr) -> bool {
        let entry = self.peers
            .entry(peer)
            .or_insert_with(|| RwLock::new(PeerRateLimit::new()));
        let mut peer_state = entry.write();

        if peer_state.connections >= self.config.max_connections_per_peer {
            warn!("Rate limit: too many connections from {}", peer);
            return false;
        }

        peer_state.connections += 1;
        debug!("Connection allowed for {} (total: {})", peer, peer_state.connections);
        true
    }

    /// Record connection closure
    pub fn record_disconnect(&self, peer: SocketAddr) {
        if let Some(entry) = self.peers.get(&peer) {
            let mut state = entry.write();
            if state.connections > 0 {
                state.connections -= 1;
            }
            // Remove entry if no connections and window expired
            if state.connections == 0 && state.window_start.elapsed().as_secs() > self.config.window_seconds {
                drop(state);
                self.peers.remove(&peer);
            }
        }
    }

    /// Check if a message from this peer is allowed
    pub fn allow_message(&self, peer: SocketAddr, size_bytes: usize) -> bool {
        let entry = self.peers
            .entry(peer)
            .or_insert_with(|| RwLock::new(PeerRateLimit::new()));
        let mut peer_state = entry.write();

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);

        // Reset window if expired
        if now.duration_since(peer_state.window_start) > window_duration {
            peer_state.reset_window();
        }

        // Check message rate
        let messages_in_window = peer_state.message_timestamps.len() as u32;
        if messages_in_window >= self.config.max_messages_per_second {
            warn!("Rate limit: too many messages from {} ({} in window)", peer, messages_in_window);
            return false;
        }

        // Check byte rate
        let new_bytes = peer_state.bytes_in_window + size_bytes as u64;
        if new_bytes > self.config.max_bytes_per_second {
            warn!("Rate limit: too many bytes from {} ({} bytes in window)", peer, new_bytes);
            return false;
        }

        // Record message
        peer_state.message_timestamps.push(now);
        peer_state.bytes_in_window = new_bytes;

        // Clean old timestamps (outside current window)
        peer_state.message_timestamps.retain(|&ts| now.duration_since(ts) <= window_duration);

        true
    }

    /// Get current connection count for a peer
    pub fn connection_count(&self, peer: SocketAddr) -> usize {
        self.peers
            .get(&peer)
            .map(|entry| entry.read().connections)
            .unwrap_or(0)
    }

    /// Clean up expired entries (call periodically)
    pub fn cleanup(&self) {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds * 2); // Keep for 2x window

        self.peers.retain(|peer, entry| {
            let state = entry.read();
            let should_keep = state.connections > 0 
                || now.duration_since(state.window_start) < window_duration;
            if !should_keep {
                debug!("Cleaning up rate limit state for {}", peer);
            }
            should_keep
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_peer() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
    }

    #[test]
    fn test_connection_limit() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_connections_per_peer: 2,
            ..Default::default()
        });

        let peer = test_peer();
        assert!(limiter.allow_connection(peer));
        assert!(limiter.allow_connection(peer));
        assert!(!limiter.allow_connection(peer)); // Should be blocked
    }

    #[test]
    fn test_message_rate_limit() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_messages_per_second: 5,
            window_seconds: 1,
            ..Default::default()
        });

        let peer = test_peer();
        for _ in 0..5 {
            assert!(limiter.allow_message(peer, 100));
        }
        assert!(!limiter.allow_message(peer, 100)); // Should be blocked
    }

    #[test]
    fn test_byte_rate_limit() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_bytes_per_second: 1000,
            window_seconds: 1,
            ..Default::default()
        });

        let peer = test_peer();
        assert!(limiter.allow_message(peer, 500));
        assert!(limiter.allow_message(peer, 400));
        assert!(!limiter.allow_message(peer, 200)); // Should be blocked (exceeds 1000)
    }

    #[test]
    fn test_disconnect_decrements_count() {
        let limiter = RateLimiter::with_config(RateLimitConfig {
            max_connections_per_peer: 2,
            ..Default::default()
        });

        let peer = test_peer();
        assert!(limiter.allow_connection(peer));
        assert!(limiter.allow_connection(peer));
        assert!(!limiter.allow_connection(peer));

        limiter.record_disconnect(peer);
        assert!(limiter.allow_connection(peer)); // Should allow after disconnect
    }
}

