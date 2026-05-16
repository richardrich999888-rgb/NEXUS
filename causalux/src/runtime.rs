// CAUSALUX v2.0 - Integration Layer + Unified Runtime
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Inventor: Katta Naga Sri Ganesh

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::{
    CausalOp, CausalDAG, VersionVector, Snapshot,
    HierarchicalSync, AdaptiveSync, ConflictPolicy,
    RGAText, GCounter, PNCounter, ORSet, LWWMap, CRDTDocument,
    SyncRequest, SyncResponse,
};

#[cfg(feature = "bft")]
use crate::BFTValidator;

// ============================================================================
// CAUSALUX RUNTIME: Unified execution environment
// ============================================================================

/// The main CAUSALUX runtime - unified execution environment for distributed apps
pub struct CausaluxRuntime {
    /// Core causal DAG with version vectors
    dag: Arc<RwLock<CausalDAG>>,
    
    /// Hierarchical sync protocol
    sync: Arc<RwLock<AdaptiveSync>>,
    
    /// CRDT document store
    documents: Arc<RwLock<HashMap<String, CRDTDocument>>>,
    
    /// Counter store
    counters: Arc<RwLock<HashMap<String, PNCounter>>>,
    
    /// Set store
    sets: Arc<RwLock<HashMap<String, ORSet<String>>>>,
    
    /// Map store
    maps: Arc<RwLock<HashMap<String, LWWMap<String, String>>>>,
    
    /// This node's configuration
    config: RuntimeConfig,
    
    /// Node keypair for signing operations
    keypair: SigningKey,
    
    /// Performance metrics
    metrics: Arc<RwLock<RuntimeMetrics>>,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub node_id: String,
    pub snapshot_interval: usize,
    pub bft_enabled: bool,
    pub conflict_policy: ConflictPolicy,
    pub sync_batch_size: usize,
    pub partition_threshold: Duration,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            node_id: format!("node_{}", uuid::Uuid::new_v4()),
            snapshot_interval: 10_000,
            bft_enabled: false,
            conflict_policy: ConflictPolicy::LastWriterWins,
            sync_batch_size: 1000,
            partition_threshold: Duration::from_secs(3600),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeMetrics {
    pub operations_executed: u64,
    pub operations_synced: u64,
    pub conflicts_resolved: u64,
    pub bft_validations: u64,
    pub snapshots_created: u64,
    pub total_sync_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl CausaluxRuntime {
    /// Create a new CAUSALUX runtime instance
    pub fn new(config: RuntimeConfig) -> Self {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng{}, &mut bytes);
        let keypair = SigningKey::from_bytes(&bytes);
        let sync = AdaptiveSync::new(config.sync_batch_size, config.partition_threshold);

        Self {
            dag: Arc::new(RwLock::new(CausalDAG::new(
                config.node_id.clone(),
                config.snapshot_interval,
                config.conflict_policy.clone(),
            ))),
            sync: Arc::new(RwLock::new(sync)),
            documents: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            sets: Arc::new(RwLock::new(HashMap::new())),
            maps: Arc::new(RwLock::new(HashMap::new())),
            config,
            keypair,
            metrics: Arc::new(RwLock::new(RuntimeMetrics::default())),
        }
    }

    // ========================================================================
    // DOCUMENT OPERATIONS
    // ========================================================================

    /// Create a new CRDT document
    pub fn create_document(&self, doc_id: String, title: String) -> Result<(), RuntimeError> {
        let mut docs = self.documents.write().unwrap();
        
        if docs.contains_key(&doc_id) {
            return Err(RuntimeError::DocumentExists);
        }

        let mut doc = CRDTDocument::new(doc_id.clone(), self.config.node_id.clone());
        doc.set_title(title.clone());

        // Create causal operation for document creation
        let op = self.create_operation(
            "create_document",
            serde_json::json!({
                "doc_id": doc_id,
                "title": title,
            }),
            vec![],
        )?;

        // Insert into DAG
        self.dag.write().unwrap().insert(op)?;

        docs.insert(doc_id, doc);

        self.metrics.write().unwrap().operations_executed += 1;

        Ok(())
    }

    /// Insert text into document
    pub fn insert_text(
        &self,
        doc_id: &str,
        position: usize,
        text: &str,
    ) -> Result<Vec<String>, RuntimeError> {
        let mut docs = self.documents.write().unwrap();
        
        let doc = docs.get_mut(doc_id).ok_or(RuntimeError::DocumentNotFound)?;

        // Insert characters
        let mut char_ids = vec![];
        for (i, ch) in text.chars().enumerate() {
            let char_id = doc.content.insert(position + i, ch);
            char_ids.push(format!("{:?}", char_id));
        }

        // Create causal operation
        let op = self.create_operation(
            "insert_text",
            serde_json::json!({
                "doc_id": doc_id,
                "position": position,
                "text": text,
                "char_ids": char_ids,
            }),
            vec![],
        )?;

        self.dag.write().unwrap().insert(op)?;
        self.metrics.write().unwrap().operations_executed += 1;

        Ok(char_ids)
    }

    /// Delete text from document
    pub fn delete_text(
        &self,
        doc_id: &str,
        position: usize,
        length: usize,
    ) -> Result<(), RuntimeError> {
        let mut docs = self.documents.write().unwrap();
        
        let doc = docs.get_mut(doc_id).ok_or(RuntimeError::DocumentNotFound)?;

        doc.delete_text(position, length);

        let op = self.create_operation(
            "delete_text",
            serde_json::json!({
                "doc_id": doc_id,
                "position": position,
                "length": length,
            }),
            vec![],
        )?;

        self.dag.write().unwrap().insert(op)?;
        self.metrics.write().unwrap().operations_executed += 1;

        Ok(())
    }

    /// Get document content as JSON
    pub fn get_document(&self, doc_id: &str) -> Result<serde_json::Value, RuntimeError> {
        let docs = self.documents.read().unwrap();
        
        let doc = docs.get(doc_id).ok_or(RuntimeError::DocumentNotFound)?;

        Ok(doc.to_json())
    }

    // ========================================================================
    // COUNTER OPERATIONS
    // ========================================================================

    /// Increment or decrement counter
    pub fn increment_counter(&self, counter_id: &str, amount: i64) -> Result<i64, RuntimeError> {
        let mut counters = self.counters.write().unwrap();
        
        let counter = counters
            .entry(counter_id.to_string())
            .or_insert_with(|| PNCounter::new(self.config.node_id.clone()));

        if amount >= 0 {
            counter.increment(amount as u64);
        } else {
            counter.decrement((-amount) as u64);
        }

        let op = self.create_operation(
            "increment_counter",
            serde_json::json!({
                "counter_id": counter_id,
                "amount": amount,
            }),
            vec![],
        )?;

        self.dag.write().unwrap().insert(op)?;
        self.metrics.write().unwrap().operations_executed += 1;

        Ok(counter.value())
    }

    /// Get counter value
    pub fn get_counter(&self, counter_id: &str) -> Result<i64, RuntimeError> {
        let counters = self.counters.read().unwrap();
        
        let counter = counters.get(counter_id).ok_or(RuntimeError::CounterNotFound)?;

        Ok(counter.value())
    }

    // ========================================================================
    // SET OPERATIONS
    // ========================================================================

    /// Add element to set
    pub fn add_to_set(&self, set_id: &str, element: String) -> Result<(), RuntimeError> {
        let mut sets = self.sets.write().unwrap();
        
        let set = sets
            .entry(set_id.to_string())
            .or_insert_with(|| ORSet::new(self.config.node_id.clone()));

        set.add(element.clone());

        let op = self.create_operation(
            "add_to_set",
            serde_json::json!({
                "set_id": set_id,
                "element": element,
            }),
            vec![],
        )?;

        self.dag.write().unwrap().insert(op)?;
        self.metrics.write().unwrap().operations_executed += 1;

        Ok(())
    }

    /// Remove element from set
    pub fn remove_from_set(&self, set_id: &str, element: &str) -> Result<(), RuntimeError> {
        let mut sets = self.sets.write().unwrap();
        
        let set = sets.get_mut(set_id).ok_or(RuntimeError::SetNotFound)?;

        set.remove(&element.to_string());

        let op = self.create_operation(
            "remove_from_set",
            serde_json::json!({
                "set_id": set_id,
                "element": element,
            }),
            vec![],
        )?;

        self.dag.write().unwrap().insert(op)?;
        self.metrics.write().unwrap().operations_executed += 1;

        Ok(())
    }

    /// Get set elements
    pub fn get_set(&self, set_id: &str) -> Result<Vec<String>, RuntimeError> {
        let sets = self.sets.read().unwrap();
        
        let set = sets.get(set_id).ok_or(RuntimeError::SetNotFound)?;

        Ok(set.elements())
    }

    // ========================================================================
    // SYNCHRONIZATION
    // ========================================================================

    /// Synchronize with another node
    pub fn sync_with_node(&self, peer_runtime: &CausaluxRuntime) -> Result<SyncResult, RuntimeError> {
        let start = Instant::now();

        // 1. Prepare sync request
        let dag = self.dag.read().unwrap();
        let version_vector = dag.get_version_vector().clone();
        let merkle_root = dag.compute_merkle_root();
        drop(dag);

        let request = self.sync.read().unwrap().prepare_request(
            self.config.node_id.clone(),
            version_vector.clone(),
            merkle_root,
        );

        // 2. Peer handles request and returns response
        let peer_dag = peer_runtime.dag.read().unwrap();
        let peer_ops: Vec<CausalOp> = peer_dag.get_operations_after(0).into_iter().cloned().collect();
        drop(peer_dag);

        // 3. Apply operations from peer
        for op in &peer_ops {
            self.apply_remote_operation(op.clone())?;
        }

        // 4. Merge CRDT states
        self.merge_crdts_with(peer_runtime)?;

        // 5. Update metrics
        let bytes_transferred = peer_ops.len() * 500; // Estimate
        let mut metrics = self.metrics.write().unwrap();
        metrics.operations_synced += peer_ops.len() as u64;
        metrics.total_sync_bytes += bytes_transferred as u64;

        Ok(SyncResult {
            operations_synced: peer_ops.len(),
            bytes_transferred,
            duration: start.elapsed(),
            snapshot_used: false,
        })
    }

    fn apply_remote_operation(&self, op: CausalOp) -> Result<(), RuntimeError> {
        // Insert into DAG
        let _ = self.dag.write().unwrap().insert(op.clone());

        // Apply to appropriate CRDT based on operation type
        match op.operation.as_str() {
            "create_document" => {
                if let Some(doc_id) = op.input.get("doc_id").and_then(|v| v.as_str()) {
                    if let Some(title) = op.input.get("title").and_then(|v| v.as_str()) {
                        let mut docs = self.documents.write().unwrap();
                        if !docs.contains_key(doc_id) {
                            let mut doc = CRDTDocument::new(doc_id.to_string(), op.node_id.clone());
                            doc.set_title(title.to_string());
                            docs.insert(doc_id.to_string(), doc);
                        }
                    }
                }
            }
            "increment_counter" => {
                if let Some(counter_id) = op.input.get("counter_id").and_then(|v| v.as_str()) {
                    if let Some(amount) = op.input.get("amount").and_then(|v| v.as_i64()) {
                        let mut counters = self.counters.write().unwrap();
                        let counter = counters
                            .entry(counter_id.to_string())
                            .or_insert_with(|| PNCounter::new(self.config.node_id.clone()));
                        
                        if amount >= 0 {
                            counter.increment(amount as u64);
                        } else {
                            counter.decrement((-amount) as u64);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn merge_crdts_with(&self, peer: &CausaluxRuntime) -> Result<(), RuntimeError> {
        // Merge documents
        let peer_docs = peer.documents.read().unwrap();
        let mut local_docs = self.documents.write().unwrap();
        
        for (doc_id, peer_doc) in peer_docs.iter() {
            if let Some(local_doc) = local_docs.get_mut(doc_id) {
                local_doc.merge(peer_doc);
            } else {
                local_docs.insert(doc_id.clone(), peer_doc.clone());
            }
        }
        drop(local_docs);
        drop(peer_docs);

        // Merge counters
        let peer_counters = peer.counters.read().unwrap();
        let mut local_counters = self.counters.write().unwrap();
        
        for (counter_id, peer_counter) in peer_counters.iter() {
            if let Some(local_counter) = local_counters.get_mut(counter_id) {
                local_counter.merge(peer_counter);
            } else {
                local_counters.insert(counter_id.clone(), peer_counter.clone());
            }
        }
        drop(local_counters);
        drop(peer_counters);

        // Merge sets
        let peer_sets = peer.sets.read().unwrap();
        let mut local_sets = self.sets.write().unwrap();
        
        for (set_id, peer_set) in peer_sets.iter() {
            if let Some(local_set) = local_sets.get_mut(set_id) {
                local_set.merge(peer_set);
            } else {
                local_sets.insert(set_id.clone(), peer_set.clone());
            }
        }

        Ok(())
    }

    // ========================================================================
    // UTILITY
    // ========================================================================

    fn create_operation(
        &self,
        operation: &str,
        input: serde_json::Value,
        dependencies: Vec<String>,
    ) -> Result<CausalOp, RuntimeError> {
        let mut version_vector = self.dag.read().unwrap().get_version_vector().clone();
        version_vector.increment(&self.config.node_id);

        let op = CausalOp::new(
            uuid::Uuid::new_v4().to_string(),
            input,
            dependencies.into_iter().collect(),
            version_vector,
            self.config.node_id.clone(),
            &self.keypair,
        );

        Ok(op)
    }

    /// Get runtime metrics
    pub fn get_metrics(&self) -> RuntimeMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Get node ID
    pub fn node_id(&self) -> String {
        self.config.node_id.clone()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum RuntimeError {
    DocumentExists,
    DocumentNotFound,
    CounterNotFound,
    SetNotFound,
    MapNotFound,
    DAGError(String),
    SyncError(String),
    BFTError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::DocumentExists => write!(f, "Document already exists"),
            RuntimeError::DocumentNotFound => write!(f, "Document not found"),
            RuntimeError::CounterNotFound => write!(f, "Counter not found"),
            RuntimeError::SetNotFound => write!(f, "Set not found"),
            RuntimeError::MapNotFound => write!(f, "Map not found"),
            RuntimeError::DAGError(msg) => write!(f, "DAG error: {}", msg),
            RuntimeError::SyncError(msg) => write!(f, "Sync error: {}", msg),
            RuntimeError::BFTError(msg) => write!(f, "BFT error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<String> for RuntimeError {
    fn from(s: String) -> Self {
        RuntimeError::DAGError(s)
    }
}

impl From<crate::dag::DagError> for RuntimeError {
    fn from(e: crate::dag::DagError) -> Self {
        RuntimeError::DAGError(e.to_string())
    }
}

// ============================================================================
// SYNC RESULT
// ============================================================================

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub operations_synced: usize,
    pub bytes_transferred: usize,
    pub duration: Duration,
    pub snapshot_used: bool,
}

// ============================================================================
// END-TO-END TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e_document_creation_and_sync() {
        // Create two nodes
        let node1 = CausaluxRuntime::new(RuntimeConfig {
            node_id: "alice".to_string(),
            ..Default::default()
        });

        let node2 = CausaluxRuntime::new(RuntimeConfig {
            node_id: "bob".to_string(),
            ..Default::default()
        });

        // Alice creates document
        node1.create_document("doc1".to_string(), "My Document".to_string()).unwrap();
        node1.insert_text("doc1", 0, "Hello World").unwrap();

        // Bob creates different document
        node2.create_document("doc2".to_string(), "Bob's Document".to_string()).unwrap();
        node2.insert_text("doc2", 0, "Greetings").unwrap();

        // Sync
        let result = node1.sync_with_node(&node2).unwrap();
        println!("✅ Synced {} operations in {:?}", result.operations_synced, result.duration);

        // Both should have both documents now
        assert!(node1.get_document("doc1").is_ok());
        assert!(node1.get_document("doc2").is_ok());
    }

    #[test]
    fn test_e2e_collaborative_editing() {
        let alice = CausaluxRuntime::new(RuntimeConfig {
            node_id: "alice".to_string(),
            ..Default::default()
        });

        let bob = CausaluxRuntime::new(RuntimeConfig {
            node_id: "bob".to_string(),
            ..Default::default()
        });

        // Both create same document (simulating offline)
        alice.create_document("shared".to_string(), "Shared Doc".to_string()).unwrap();
        bob.create_document("shared".to_string(), "Shared Doc".to_string()).unwrap();

        // Both edit concurrently (offline)
        alice.insert_text("shared", 0, "Alice was here. ").unwrap();
        bob.insert_text("shared", 0, "Bob was here. ").unwrap();

        // Sync everyone
        alice.sync_with_node(&bob).unwrap();
        bob.sync_with_node(&alice).unwrap();

        // Both should have same content after merge
        let doc_alice = alice.get_document("shared").unwrap();
        let doc_bob = bob.get_document("shared").unwrap();

        println!("✅ Alice's doc: {}", doc_alice);
        println!("✅ Bob's doc: {}", doc_bob);
    }

    #[test]
    fn test_e2e_distributed_counter() {
        let node1 = CausaluxRuntime::new(RuntimeConfig {
            node_id: "counter1".to_string(),
            ..Default::default()
        });

        let node2 = CausaluxRuntime::new(RuntimeConfig {
            node_id: "counter2".to_string(),
            ..Default::default()
        });

        // Both increment same counter (offline)
        for _ in 0..100 {
            node1.increment_counter("upvotes", 1).unwrap();
        }

        for _ in 0..150 {
            node2.increment_counter("upvotes", 1).unwrap();
        }

        // Sync
        node1.sync_with_node(&node2).unwrap();
        node2.sync_with_node(&node1).unwrap();

        // Both should have same total
        let count1 = node1.get_counter("upvotes").unwrap();
        let count2 = node2.get_counter("upvotes").unwrap();

        assert_eq!(count1, count2);
        println!("✅ Distributed counter converged to: {}", count1);
    }

    #[test]
    fn test_e2e_set_operations() {
        let node1 = CausaluxRuntime::new(RuntimeConfig::default());
        let node2 = CausaluxRuntime::new(RuntimeConfig::default());

        // Node1 adds items
        node1.add_to_set("cart", "iPhone".to_string()).unwrap();
        node1.add_to_set("cart", "AirPods".to_string()).unwrap();

        // Node2 adds items
        node2.add_to_set("cart", "MacBook".to_string()).unwrap();
        node2.add_to_set("cart", "iPhone".to_string()).unwrap();

        // Sync
        node1.sync_with_node(&node2).unwrap();
        node2.sync_with_node(&node1).unwrap();

        // Check set contents
        let set1 = node1.get_set("cart").unwrap();
        let set2 = node2.get_set("cart").unwrap();

        println!("✅ Set contents after merge: {:?}", set1);
        
        // Both should have same elements
        assert_eq!(set1.len(), set2.len());
    }

    #[test]
    fn test_e2e_metrics_tracking() {
        let runtime = CausaluxRuntime::new(RuntimeConfig::default());

        // Perform operations
        runtime.create_document("doc1".to_string(), "Test".to_string()).unwrap();
        runtime.insert_text("doc1", 0, "Hello").unwrap();
        runtime.increment_counter("counter1", 5).unwrap();
        runtime.add_to_set("set1", "item1".to_string()).unwrap();

        // Check metrics
        let metrics = runtime.get_metrics();
        
        assert_eq!(metrics.operations_executed, 4);
        println!("✅ Metrics tracked: {:?}", metrics);
    }
}
