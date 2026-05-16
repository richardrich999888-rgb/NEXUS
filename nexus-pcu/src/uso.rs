// NEXUS USO: Universal State Object
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Key innovation: One primitive replaces databases, caches, queues, files.
// Sync policy determines behavior. Same mental model everywhere.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::content_hash::ContentHash;
use crate::identity::PrincipalId;
use crate::Timestamp;

// ============================================================================
// REGION - Geographic/logical region for sync policies
// ============================================================================

/// Geographic or logical region
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Region(pub String);

impl Region {
    pub fn new(name: impl Into<String>) -> Self {
        Region(name.into())
    }

    /// Well-known regions
    pub fn us_east() -> Self { Region("us-east".to_string()) }
    pub fn us_west() -> Self { Region("us-west".to_string()) }
    pub fn eu_west() -> Self { Region("eu-west".to_string()) }
    pub fn ap_south() -> Self { Region("ap-south".to_string()) }
    pub fn local() -> Self { Region("local".to_string()) }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// SCHEMA REF - Type information for schema evolution
// ============================================================================

/// Reference to schema (for type safety and evolution)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaRef {
    /// Schema name (e.g., "user.profile.v1")
    pub name: String,
    /// Schema version
    pub version: u32,
    /// Hash of schema definition (for verification)
    pub hash: ContentHash,
}

impl SchemaRef {
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        let name = name.into();
        let hash = ContentHash::compute(format!("{}:{}", name, version).as_bytes());
        SchemaRef { name, version, hash }
    }

    /// Untyped/raw bytes
    pub fn raw() -> Self {
        SchemaRef::new("raw", 0)
    }

    /// JSON schema
    pub fn json() -> Self {
        SchemaRef::new("json", 0)
    }
}

// ============================================================================
// SYNC POLICY - How state propagates
// ============================================================================

/// Synchronization policy for USO
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncPolicy {
    /// Sync everywhere immediately (global consistency)
    Global {
        /// Maximum acceptable replication latency
        max_latency_ms: u32,
    },
    
    /// Sync to specific regions only
    Regional {
        regions: Vec<Region>,
    },
    
    /// Sync on-demand (pull-based, lazy)
    OnDemand,
    
    /// Local-only (single node, no sync)
    Local,
}

impl Default for SyncPolicy {
    fn default() -> Self {
        SyncPolicy::OnDemand
    }
}

impl SyncPolicy {
    /// Quick global sync (100ms latency target)
    pub fn global_fast() -> Self {
        SyncPolicy::Global { max_latency_ms: 100 }
    }

    /// Relaxed global sync (1s latency target)
    pub fn global_relaxed() -> Self {
        SyncPolicy::Global { max_latency_ms: 1000 }
    }

    /// Check if this policy requires immediate sync
    pub fn requires_immediate(&self) -> bool {
        matches!(self, SyncPolicy::Global { max_latency_ms } if *max_latency_ms < 500)
    }
}

// ============================================================================
// ACCESS POLICY - Who can read/write this USO
// ============================================================================

/// Access control for USO
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessPolicy {
    /// Owner (full control)
    pub owner: PrincipalId,
    
    /// Principals with read access
    pub readers: Vec<PrincipalId>,
    
    /// Principals with write access
    pub writers: Vec<PrincipalId>,
    
    /// Public read access
    pub public_read: bool,
    
    /// Public write access (dangerous!)
    pub public_write: bool,
}

impl AccessPolicy {
    /// Create with single owner
    pub fn owner_only(owner: PrincipalId) -> Self {
        AccessPolicy {
            owner,
            readers: Vec::new(),
            writers: Vec::new(),
            public_read: false,
            public_write: false,
        }
    }

    /// Create public read-only
    pub fn public_readonly(owner: PrincipalId) -> Self {
        AccessPolicy {
            owner,
            readers: Vec::new(),
            writers: Vec::new(),
            public_read: true,
            public_write: false,
        }
    }

    /// Check if principal can read
    pub fn can_read(&self, principal: &PrincipalId) -> bool {
        self.public_read || 
        *principal == self.owner ||
        self.readers.contains(principal) ||
        self.writers.contains(principal)
    }

    /// Check if principal can write
    pub fn can_write(&self, principal: &PrincipalId) -> bool {
        self.public_write ||
        *principal == self.owner ||
        self.writers.contains(principal)
    }
}

// ============================================================================
// OPERATION - Single operation in causal history
// ============================================================================

/// Single operation on a USO
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Set entire value
    Set { value: Vec<u8> },
    
    /// Merge with another value (for CRDTs)
    Merge { other_hash: ContentHash },
    
    /// Delete the USO
    Delete,
    
    /// Patch operation (for structured data)
    Patch { path: String, value: Vec<u8> },
}

// ============================================================================
// CAUSAL HISTORY - For conflict-free merge
// ============================================================================

/// Entry in causal history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Operation performed
    pub operation: Operation,
    
    /// Who performed it
    pub principal: PrincipalId,
    
    /// When (Lamport timestamp from vector clock)
    pub lamport: u64,
    
    /// When (wall clock, for display)
    pub timestamp: Timestamp,
}

/// Causal history for USO (enables CRDT-style merge)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalHistory {
    /// Vector clock for causality tracking
    pub vector_clock: HashMap<String, u64>,
    
    /// Operation log
    pub operations: Vec<HistoryEntry>,
    
    /// Parent states this was derived from
    pub parents: Vec<ContentHash>,
}

impl CausalHistory {
    /// Create new empty history
    pub fn new() -> Self {
        CausalHistory {
            vector_clock: HashMap::new(),
            operations: Vec::new(),
            parents: Vec::new(),
        }
    }

    /// Create genesis history
    pub fn genesis() -> Self {
        CausalHistory::new()
    }

    /// Tick the vector clock for a node
    pub fn tick(&mut self, node_id: &str) -> u64 {
        let counter = self.vector_clock.entry(node_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Get current Lamport timestamp
    pub fn lamport(&self) -> u64 {
        self.vector_clock.values().copied().max().unwrap_or(0)
    }

    /// Add operation to history
    pub fn add_operation(&mut self, operation: Operation, principal: PrincipalId, node_id: &str) {
        let lamport = self.tick(node_id);
        self.operations.push(HistoryEntry {
            operation,
            principal,
            lamport,
            timestamp: crate::now(),
        });
    }

    /// Merge with another history (CRDT-style)
    pub fn merge(&mut self, other: &CausalHistory) {
        // Merge vector clocks (take max)
        for (node, &counter) in &other.vector_clock {
            let entry = self.vector_clock.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(counter);
        }

        // Merge operations (by Lamport + dedup)
        for op in &other.operations {
            if !self.operations.iter().any(|o| o.lamport == op.lamport && o.principal == op.principal) {
                self.operations.push(op.clone());
            }
        }

        // Sort by Lamport for deterministic order
        self.operations.sort_by_key(|o| o.lamport);
    }

    /// Check if this history happens-before another
    pub fn happens_before(&self, other: &CausalHistory) -> bool {
        // Self happens before other if all our clocks <= other's and at least one <
        let mut at_least_one_less = false;
        
        for (node, &counter) in &self.vector_clock {
            let other_counter = other.vector_clock.get(node).copied().unwrap_or(0);
            if counter > other_counter {
                return false;
            }
            if counter < other_counter {
                at_least_one_less = true;
            }
        }
        
        at_least_one_less
    }
}

impl Default for CausalHistory {
    fn default() -> Self {
        CausalHistory::new()
    }
}

// ============================================================================
// USO - Universal State Object
// ============================================================================

/// Universal State Object
/// 
/// Replaces: databases, caches, queues, file systems, KV stores.
/// One primitive with configurable sync behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USO {
    /// Content-addressed identifier
    pub id: ContentHash,
    
    /// The actual data (opaque bytes)
    pub data: Vec<u8>,
    
    /// Type information (for schema evolution)
    pub schema: SchemaRef,
    
    /// Causal history (for merge)
    pub history: CausalHistory,
    
    /// Access control (who can read/write)
    pub access: AccessPolicy,
    
    /// Sync policy (how this propagates)
    pub sync: SyncPolicy,
    
    /// Created timestamp
    pub created_at: Timestamp,
    
    /// Last modified timestamp
    pub modified_at: Timestamp,
}

impl USO {
    /// Create new USO
    pub fn new(data: Vec<u8>, owner: PrincipalId) -> Self {
        let id = ContentHash::compute(&data);
        let now = crate::now();
        
        USO {
            id,
            data,
            schema: SchemaRef::raw(),
            history: CausalHistory::new(),
            access: AccessPolicy::owner_only(owner),
            sync: SyncPolicy::default(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Create with schema
    pub fn with_schema(mut self, schema: SchemaRef) -> Self {
        self.schema = schema;
        self
    }

    /// Create with sync policy
    pub fn with_sync(mut self, policy: SyncPolicy) -> Self {
        self.sync = policy;
        self
    }

    /// Create with access policy
    pub fn with_access(mut self, access: AccessPolicy) -> Self {
        self.access = access;
        self
    }

    /// Update data (creates new version)
    pub fn update(&mut self, data: Vec<u8>, principal: PrincipalId, node_id: &str) {
        self.data = data.clone();
        self.id = ContentHash::compute(&self.data);
        self.modified_at = crate::now();
        self.history.add_operation(Operation::Set { value: data }, principal, node_id);
    }

    /// Merge with another USO (CRDT-style)
    pub fn merge(&mut self, other: &USO) {
        // Merge histories
        self.history.merge(&other.history);
        
        // Take newer data (by modified_at)
        if other.modified_at > self.modified_at {
            self.data = other.data.clone();
            self.id = other.id;
            self.modified_at = other.modified_at;
        }
        
        // Add parent reference
        if !self.history.parents.contains(&other.id) {
            self.history.parents.push(other.id);
        }
    }

    /// Check if principal can read
    pub fn can_read(&self, principal: &PrincipalId) -> bool {
        self.access.can_read(principal)
    }

    /// Check if principal can write
    pub fn can_write(&self, principal: &PrincipalId) -> bool {
        self.access.can_write(principal)
    }

    /// Get data size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Serialize to bytes
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uso_creation() {
        let owner = PrincipalId::generate();
        let uso = USO::new(b"hello world".to_vec(), owner);
        
        assert!(uso.can_read(&owner));
        assert!(uso.can_write(&owner));
    }

    #[test]
    fn test_uso_access_control() {
        let owner = PrincipalId::generate();
        let other = PrincipalId::generate();
        
        let uso = USO::new(b"secret".to_vec(), owner);
        
        assert!(uso.can_read(&owner));
        assert!(!uso.can_read(&other));
    }

    #[test]
    fn test_uso_public_read() {
        let owner = PrincipalId::generate();
        let other = PrincipalId::generate();
        
        let uso = USO::new(b"public".to_vec(), owner)
            .with_access(AccessPolicy::public_readonly(owner));
        
        assert!(uso.can_read(&other));
        assert!(!uso.can_write(&other));
    }

    #[test]
    fn test_uso_update() {
        let owner = PrincipalId::generate();
        let mut uso = USO::new(b"v1".to_vec(), owner);
        let old_id = uso.id;
        
        uso.update(b"v2".to_vec(), owner, "node1");
        
        assert_ne!(uso.id, old_id);
        assert_eq!(uso.data, b"v2");
        assert_eq!(uso.history.operations.len(), 1);
    }

    #[test]
    fn test_uso_merge() {
        let owner = PrincipalId::generate();
        
        let mut uso1 = USO::new(b"base".to_vec(), owner);
        uso1.update(b"v1".to_vec(), owner, "node1");
        
        let mut uso2 = USO::new(b"base".to_vec(), owner);
        uso2.update(b"v2".to_vec(), owner, "node2");
        
        uso1.merge(&uso2);
        
        // Should have operations from both nodes
        assert!(uso1.history.vector_clock.contains_key("node1"));
        assert!(uso1.history.vector_clock.contains_key("node2"));
    }

    #[test]
    fn test_uso_serialization() {
        let owner = PrincipalId::generate();
        let uso = USO::new(b"test".to_vec(), owner)
            .with_sync(SyncPolicy::global_fast());
        
        let bytes = uso.to_bytes().unwrap();
        let restored = USO::from_bytes(&bytes).unwrap();
        
        assert_eq!(uso.id, restored.id);
        assert_eq!(uso.data, restored.data);
    }

    #[test]
    fn test_sync_policy() {
        let global = SyncPolicy::global_fast();
        let local = SyncPolicy::Local;
        
        assert!(global.requires_immediate());
        assert!(!local.requires_immediate());
    }

    #[test]
    fn test_causal_history_happens_before() {
        let mut h1 = CausalHistory::new();
        h1.tick("node1");
        
        let mut h2 = h1.clone();
        h2.tick("node1");
        h2.tick("node2");
        
        assert!(h1.happens_before(&h2));
        assert!(!h2.happens_before(&h1));
    }
}
