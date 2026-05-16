//! Tesla Resonance Layer
//! 
//! Smart sync routing based on node affinity patterns.
//! Nodes sync more efficiently by detecting natural "resonance" in their data access patterns.

pub mod affinity;
pub mod router;

pub use affinity::{AffinityTracker, NodeAffinity, DataPattern};
pub use router::{ResonantRouter, SyncRoute, RoutingDecision};
