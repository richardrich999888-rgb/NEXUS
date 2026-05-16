// NEXUS Sync Layer - Powered by CAUSALUX
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
// Inventor: Katta Naga Sri Ganesh
//
// This module integrates CAUSALUX's distributed sync primitives
// with NEXUS's PCU and USO abstractions.

pub mod sync_engine;
pub mod crdt_uso;
pub mod adapters;

// Re-export CAUSALUX types for convenience
pub use causalux_v2::{
    // Core sync primitives
    VersionVector,
    CausalOp,
    CausalDAG,
    ConflictPolicy,
    ConflictResolution,
    
    // CRDTs for automatic merge
    RGAText,
    GCounter,
    PNCounter,
    LWWRegister,
    ORSet,
    LWWMap,
    CRDTDocument,
    
    // Snapshots for GC
    Snapshot,
    SnapshotManager,
    
    // Sync protocols
    HierarchicalSync,
    AdaptiveSync,
    SyncStrategy,
    SyncStats,
    SyncRequest,
    SyncResponse,
};

// Re-export our adapters
pub use sync_engine::{NexusSyncEngine, SyncDelta};
pub use crdt_uso::{CrdtUSO, USOType};
pub use adapters::ContentHashAdapter;

/// NEXUS-CAUSALUX integration version
pub const SYNC_VERSION: &str = "1.0.0";
