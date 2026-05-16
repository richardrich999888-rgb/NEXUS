// CAUSALUX v3.0 - Visionary Architecture
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Inventor: Katta Naga Sri Ganesh

//! CAUSALUX v3.0: Post-Cloud Execution Fabric with Visionary Layers
//! 
//! This library provides a production-ready distributed execution fabric with:
//! - Conflict-free guarantees via version vectors
//! - Constant memory footprint via snapshot-based GC
//! - Byzantine fault tolerance (optional)
//! - Hierarchical sync protocol for offline-first operation
//! 
//! ## V3.0 Visionary Layers:
//! - **Morgan Economy**: Token-based metering and incentive alignment
//! - **Tesla Resonance**: Smart sync routing via data affinity patterns
//! - **Da Vinci Atom**: Unified primitive for all data types
//! 
//! # Quick Start
//! 
//! ```rust
//! use causalux_v2::{CausalDAG, ConflictPolicy};
//! use ed25519_dalek::Keypair;
//! use rand::rngs::OsRng;
//! 
//! let node_id = "node1".to_string();
//! let mut dag = CausalDAG::new(
//!     node_id.clone(),
//!     10000,  // Snapshot every 10K operations
//!     ConflictPolicy::LastWriterWins
//! );
//! 
//! // Create and insert operations
//! // Operations automatically tracked with version vectors
//! ```

pub mod version_vector;
pub mod content_address;
pub mod causal_op;
pub mod snapshot;
pub mod conflict;
pub mod dag;
pub mod crdt;
pub mod sync;
pub mod presence;
pub mod envelope;
pub mod observability;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "network")]
pub mod network;

/// Transport Layer - WebSocket-based P2P communication
#[cfg(feature = "transport")]
pub mod transport;

pub mod runtime;

#[cfg(feature = "bft")]
pub mod bft;

/// Distributed GPU Computing (CAUSALUX-COMPUTE)
#[cfg(feature = "compute")]
pub mod compute;

/// Morgan Economy Layer - Token-based metering
#[cfg(feature = "economy")]
pub mod economy;

/// Tesla Resonance Layer - Smart sync routing
#[cfg(feature = "resonance")]
pub mod resonance;

/// Da Vinci Atom Layer - Unified primitives
#[cfg(feature = "atom")]
pub mod atom;

pub use version_vector::VersionVector;
pub use content_address::ContentAddress;
pub use causal_op::CausalOp;
pub use snapshot::{Snapshot, SnapshotManager};
pub use conflict::{ConflictPolicy, ConflictResolver, ConflictResolution};
pub use dag::CausalDAG;
pub use crdt::{RGAText, GCounter, PNCounter, LWWRegister, ORSet, LWWMap, CRDTDocument};
pub use sync::{HierarchicalSync, AdaptiveSync, SyncStrategy, SyncStats, SyncRequest, SyncResponse};
pub use presence::{UserPresence, CausalCursor, PresenceManager, PresenceStatus};
pub use envelope::{SovereignEnvelope, KeyDerivation, DerivedKey, KeyScope};
pub use observability::{
    AuditLogEntry, AuditOperation,AuditResult, HealthStatus, HealthState, 
    init_observability,
};

#[cfg(feature = "observability")]
pub use observability::CausaluxMetrics;

#[cfg(feature = "bft")]
pub use bft::BFTValidator;

#[cfg(feature = "economy")]
pub use economy::{CausalToken, TokenBalance, EconomyLedger, OperationPricing};

#[cfg(feature = "resonance")]
pub use resonance::{AffinityTracker, ResonantRouter, RoutingDecision};

#[cfg(feature = "atom")]
pub use atom::{CausalAtom, AtomValue, AtomComposer, CompositeAtom};

pub use runtime::{CausaluxRuntime, RuntimeConfig, RuntimeMetrics, RuntimeError, SyncResult};

/// CAUSALUX v3.0 version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default snapshot interval (operations)
pub const DEFAULT_SNAPSHOT_INTERVAL: usize = 10_000;

/// Default max snapshots to keep in memory
pub const DEFAULT_MAX_SNAPSHOTS: usize = 100;

/// Target memory footprint (bytes)
pub const TARGET_MEMORY_FOOTPRINT: usize = 1_073_741_824; // 1 GB
