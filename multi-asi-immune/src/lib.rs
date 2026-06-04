//! # Multi-ASI Immune Protocol
//!
//! A distributed immune system for coordinating multiple ASI instances.
//!
//! ## Core Mechanisms
//!
//! 1. **Identity**: Cryptographically verifiable identities via Ed25519
//! 2. **Reputation**: Earned, decaying, non-transferable trust scores
//! 3. **Threat Signatures**: Signed, reputation-weighted threat reports
//! 4. **Mutual Constraints**: Bilateral homeostatic agreements
//! 5. **Defection Detection**: Observable behavior monitoring
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    ASI Node                             │
//! ├─────────────────────────────────────────────────────────┤
//! │  Identity (Ed25519)                                     │
//! │  ├── Sign messages                                      │
//! │  └── Verify peer signatures                             │
//! ├─────────────────────────────────────────────────────────┤
//! │  Reputation                                             │
//! │  ├── Track peer behavior                                │
//! │  ├── Decay over time                                    │
//! │  └── Aggregate transitive trust                         │
//! ├─────────────────────────────────────────────────────────┤
//! │  Threat Memory                                          │
//! │  ├── Store signed reports                               │
//! │  ├── Deduplicate by pattern                             │
//! │  └── Aggregate confidence                               │
//! ├─────────────────────────────────────────────────────────┤
//! │  Protocol                                               │
//! │  ├── Handshake                                          │
//! │  ├── Gossip threats                                     │
//! │  ├── Negotiate constraints                              │
//! │  └── Heartbeat liveness                                 │
//! ├─────────────────────────────────────────────────────────┤
//! │  Enforcement                                            │
//! │  ├── Detect defection                                   │
//! │  └── Isolate bad actors                                 │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use multi_asi_immune::node::state::{AsiNode, NodeConfig};
//! use multi_asi_immune::threat::signature::{ThreatPattern, ThreatCategory};
//!
//! // Create a node
//! let config = NodeConfig::default();
//! let mut node = AsiNode::new(config);
//!
//! // Report a threat
//! let pattern = ThreatPattern {
//!     category: ThreatCategory::Deception,
//!     pattern_hash: [42; 32],
//!     severity: 0.9,
//!     context: None,
//! };
//! let report = node.report_threat(pattern, 0.85);
//! ```

pub mod identity;
pub mod attestation;
pub mod threat;
pub mod reputation;
pub mod protocol;
pub mod enforcement;
pub mod node;
pub mod integration;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::identity::keypair::{AsiIdentity, AsiId, PublicIdentity};
    pub use crate::reputation::score::ReputationScore;
    pub use crate::reputation::aggregation::ReputationAggregator;
    pub use crate::threat::pattern::{ThreatPattern, ThreatCategory};
    pub use crate::threat::signature::SignedThreatReport;
    pub use crate::threat::memory::ThreatMemory;
    pub use crate::protocol::message::ProtocolMessage;
    pub use crate::node::state::{AsiNode, NodeConfig, NetworkHealth};
}

/// Re-export key types at crate root
pub use crate::identity::keypair::AsiId;
pub use crate::node::state::AsiNode;
