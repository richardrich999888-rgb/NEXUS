// Snapshot management for garbage collection and efficient sync

use crate::version_vector::VersionVector;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

/// A point-in-time snapshot of the system state.
/// 
/// Snapshots enable:
/// - Constant memory footprint (via garbage collection)
/// - Fast sync after long partitions (download snapshot, not full history)
/// - Efficient state reconstruction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique snapshot ID
    pub id: String,
    
    /// Complete state at this point (serialized)
    pub state: serde_json::Value,
    
    /// Merkle root of all operations up to this snapshot
    pub merkle_root: String,
    
    /// Version vector at snapshot time
    pub version_vector: VersionVector,
    
    /// Unix timestamp when created
    pub timestamp: u64,
    
    /// Number of operations included
    pub operation_count: u64,
    
    /// Compressed size in bytes
    pub compressed_size: usize,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(
        state: serde_json::Value,
        merkle_root: String,
        version_vector: VersionVector,
        operation_count: u64,
    ) -> Self {
        let serialized = serde_json::to_string(&state).unwrap();
        let compressed = Self::compress_state(&serialized);
        let compressed_size = compressed.len();

        // Generate unique ID from content hash
        let id = Self::generate_id(&merkle_root, operation_count);

        Self {
            id,
            state,
            merkle_root,
            version_vector,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operation_count,
            compressed_size,
        }
    }

    /// Compress state for storage/transfer
    pub fn compress_state(state: &str) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(state.as_bytes()).unwrap();
        encoder.finish().unwrap()
    }

    /// Decompress state
    pub fn decompress_state(compressed: &[u8]) -> String {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        decompressed
    }

    /// Get compressed bytes for transfer
    pub fn to_compressed_bytes(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self.state).unwrap();
        Self::compress_state(&json)
    }

    /// Generate snapshot ID from content
    fn generate_id(merkle_root: &str, operation_count: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(merkle_root.as_bytes());
        hasher.update(operation_count.to_le_bytes());
        format!("snap_{:x}", hasher.finalize())[..24].to_string()
    }
}

/// Manages snapshots with automatic garbage collection.
/// 
/// Strategy:
/// - Create snapshot every N operations (default: 10,000)
/// - Keep last M snapshots in memory (default: 100)
/// - Archive older snapshots to cold storage
/// - Trim operations older than 2 snapshots ago
pub struct SnapshotManager {
    /// Recent snapshots (most recent at back)
    snapshots: VecDeque<Snapshot>,
    
    /// Maximum snapshots to keep in memory
    max_snapshots: usize,
    
    /// Operations per snapshot
    snapshot_interval: usize,
    
    /// Total operations processed
    total_operations: u64,
    
    /// Archived snapshot IDs (for cold storage lookup)
    archived_ids: Vec<String>,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(max_snapshots: usize, snapshot_interval: usize) -> Self {
        Self {
            snapshots: VecDeque::new(),
            max_snapshots,
            snapshot_interval,
            total_operations: 0,
            archived_ids: Vec::new(),
        }
    }

    /// Check if we should create a new snapshot
    pub fn should_snapshot(&self) -> bool {
        self.total_operations > 0
            && self.total_operations % (self.snapshot_interval as u64) == 0
    }

    /// Create and store a new snapshot
    pub fn create_snapshot(
        &mut self,
        state: serde_json::Value,
        merkle_root: String,
        version_vector: VersionVector,
    ) -> &Snapshot {
        let snapshot = Snapshot::new(
            state,
            merkle_root,
            version_vector,
            self.total_operations,
        );

        self.snapshots.push_back(snapshot);

        // Evict old snapshots to cold storage
        while self.snapshots.len() > self.max_snapshots {
            let old = self.snapshots.pop_front().unwrap();
            self.archive_snapshot(&old);
        }

        self.snapshots.back().unwrap()
    }

    /// Archive a snapshot to cold storage
    fn archive_snapshot(&mut self, snapshot: &Snapshot) {
        // In production: write to S3, GCS, or local disk
        eprintln!("📦 Archived snapshot: {} ({} bytes)", 
            snapshot.id, snapshot.compressed_size);
        self.archived_ids.push(snapshot.id.clone());
    }

    /// Increment operation count
    pub fn increment_operation_count(&mut self) {
        self.total_operations += 1;
    }

    /// Get threshold for trimmable operations (older than 2 snapshots ago)
    pub fn get_trimable_threshold(&self) -> Option<u64> {
        if self.snapshots.len() >= 2 {
            Some(self.snapshots[self.snapshots.len() - 2].operation_count)
        } else {
            None
        }
    }

    /// Get the latest snapshot
    pub fn get_latest(&self) -> Option<&Snapshot> {
        self.snapshots.back()
    }

    /// Get snapshot by ID
    pub fn get_by_id(&self, id: &str) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    /// Get all snapshot IDs (for sync negotiation)
    pub fn get_snapshot_ids(&self) -> Vec<String> {
        self.snapshots.iter().map(|s| s.id.clone()).collect()
    }

    /// Find most recent common snapshot with a peer
    pub fn find_common_snapshot(&self, peer_snapshot_ids: &[String]) -> Option<&Snapshot> {
        // Iterate from most recent to oldest
        for snapshot in self.snapshots.iter().rev() {
            if peer_snapshot_ids.contains(&snapshot.id) {
                return Some(snapshot);
            }
        }
        None
    }

    /// Get memory footprint estimate
    pub fn memory_footprint(&self) -> usize {
        self.snapshots.iter().map(|s| s.compressed_size).sum()
    }

    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.total_operations
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let vv = VersionVector::new();
        let snapshot = Snapshot::new(
            serde_json::json!({"counter": 42}),
            "merkle_root_123".to_string(),
            vv,
            1000,
        );

        assert!(!snapshot.id.is_empty());
        assert!(snapshot.compressed_size > 0);
        assert_eq!(snapshot.operation_count, 1000);
    }

    #[test]
    fn test_compression() {
        let original = r#"{"key": "value", "nested": {"a": 1, "b": 2}}"#;
        let compressed = Snapshot::compress_state(original);
        let decompressed = Snapshot::decompress_state(&compressed);

        assert_eq!(original, decompressed);
        // Compression should reduce size for larger inputs
    }

    #[test]
    fn test_snapshot_manager() {
        let mut manager = SnapshotManager::new(5, 100);

        // Simulate 100 operations
        for _ in 0..100 {
            manager.increment_operation_count();
        }

        assert!(manager.should_snapshot());

        // Create snapshot
        manager.create_snapshot(
            serde_json::json!({"state": "test"}),
            "merkle_123".to_string(),
            VersionVector::new(),
        );

        assert_eq!(manager.snapshot_count(), 1);
    }

    #[test]
    fn test_snapshot_eviction() {
        let mut manager = SnapshotManager::new(3, 100);

        // Create 5 snapshots (should evict 2)
        for i in 0..5 {
            for _ in 0..100 {
                manager.increment_operation_count();
            }
            manager.create_snapshot(
                serde_json::json!({"i": i}),
                format!("merkle_{}", i),
                VersionVector::new(),
            );
        }

        assert_eq!(manager.snapshot_count(), 3);
        assert_eq!(manager.archived_ids.len(), 2);
    }

    #[test]
    fn test_find_common_snapshot() {
        let mut manager = SnapshotManager::new(10, 100);

        for i in 0..3 {
            for _ in 0..100 {
                manager.increment_operation_count();
            }
            manager.create_snapshot(
                serde_json::json!({}),
                format!("merkle_{}", i),
                VersionVector::new(),
            );
        }

        let snapshot_ids = manager.get_snapshot_ids();
        let peer_has = vec![snapshot_ids[0].clone(), snapshot_ids[1].clone()];

        let common = manager.find_common_snapshot(&peer_has);
        assert!(common.is_some());
        assert_eq!(common.unwrap().id, snapshot_ids[1]);
    }
}
