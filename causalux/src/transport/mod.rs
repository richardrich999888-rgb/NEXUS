//! Transport Layer - Network communication for CAUSALUX
//! 
//! Provides WebSocket-based peer-to-peer communication for sync operations.

pub mod message;
pub mod server;
pub mod client;
pub mod peer;

pub use message::{SyncMessage, MessageType, PeerMessage};
pub use server::SyncServer;
pub use client::SyncClient;
pub use peer::{PeerInfo, PeerManager, PeerState};
