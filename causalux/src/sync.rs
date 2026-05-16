// Hierarchical Sync Protocol for efficient long-partition recovery
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use crate::causal_op::CausalOp;
use crate::snapshot::Snapshot;
use crate::version_vector::VersionVector;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Sync request sent to peer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub node_id: String,
    pub version_vector: VersionVector,
    pub latest_snapshot_id: Option<String>,
    pub merkle_root: String,
}

/// Sync response from peer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub common_snapshot: Option<Snapshot>,
    pub snapshot_to_download: Option<String>,
    pub operations_after_snapshot: Vec<CausalOp>,
    pub total_operations: usize,
    pub estimated_size_bytes: usize,
}

/// Sync statistics
#[derive(Clone, Debug)]
pub struct SyncStats {
    pub snapshot_downloaded: bool,
    pub snapshot_size_bytes: usize,
    pub operations_applied: usize,
    pub operations_size_bytes: usize,
    pub total_bytes: usize,
    pub sync_duration: Duration,
}

/// Bandwidth savings calculation
#[derive(Debug, Clone)]
pub struct SyncSavings {
    pub full_replication_bytes: usize,
    pub hierarchical_bytes: usize,
    pub bytes_saved: usize,
    pub savings_percent: f64,
}

/// Sync strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStrategy {
    /// Short partition: just diff operations via Merkle tree
    MerkleDiff,
    /// Long partition: download snapshot + recent operations
    Hierarchical,
}

/// Error types for sync operations
#[derive(Debug, Clone)]
pub enum SyncError {
    MissingSnapshot,
    InvalidOperation,
    NetworkError(String),
    CompressionError,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::MissingSnapshot => write!(f, "Required snapshot not found"),
            SyncError::InvalidOperation => write!(f, "Invalid operation in sync"),
            SyncError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SyncError::CompressionError => write!(f, "Compression/decompression error"),
        }
    }
}

impl std::error::Error for SyncError {}

/// Hierarchical sync protocol implementation
pub struct HierarchicalSync {
    local_snapshots: Vec<Snapshot>,
    local_operations: Vec<CausalOp>,
    batch_size: usize,
    compression_enabled: bool,
}

impl HierarchicalSync {
    pub fn new(batch_size: usize, compression_enabled: bool) -> Self {
        Self {
            local_snapshots: vec![],
            local_operations: vec![],
            batch_size,
            compression_enabled,
        }
    }

    /// Add a snapshot to local storage
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.local_snapshots.push(snapshot);
    }

    /// Add operations to local storage
    pub fn add_operations(&mut self, ops: Vec<CausalOp>) {
        self.local_operations.extend(ops);
    }

    /// Prepare sync request
    pub fn prepare_sync_request(
        &self,
        node_id: String,
        version_vector: VersionVector,
        merkle_root: String,
    ) -> SyncRequest {
        SyncRequest {
            node_id,
            version_vector,
            latest_snapshot_id: self.local_snapshots.last().map(|s| s.id.clone()),
            merkle_root,
        }
    }

    /// Handle incoming sync request
    pub fn handle_sync_request(&self, request: SyncRequest) -> SyncResponse {
        let common_snapshot = self.find_common_snapshot(&request);

        if let Some(common) = common_snapshot {
            let ops_after = self.get_operations_after_snapshot(&common.id);
            
            SyncResponse {
                common_snapshot: Some(common.clone()),
                snapshot_to_download: None,
                operations_after_snapshot: ops_after.clone(),
                total_operations: ops_after.len(),
                estimated_size_bytes: self.estimate_size(&ops_after),
            }
        } else {
            let latest = self.local_snapshots.last();
            let ops = if let Some(snap) = latest {
                self.get_operations_after_snapshot(&snap.id)
            } else {
                self.local_operations.clone()
            };

            SyncResponse {
                common_snapshot: None,
                snapshot_to_download: latest.map(|s| s.id.clone()),
                operations_after_snapshot: ops.clone(),
                total_operations: ops.len(),
                estimated_size_bytes: self.estimate_snapshot_size(latest) + self.estimate_size(&ops),
            }
        }
    }

    /// Apply sync response
    pub fn apply_sync_response(
        &mut self,
        response: SyncResponse,
        snapshot_data: Option<Snapshot>,
    ) -> Result<SyncStats, SyncError> {
        let start = Instant::now();
        let mut stats = SyncStats {
            snapshot_downloaded: false,
            snapshot_size_bytes: 0,
            operations_applied: 0,
            operations_size_bytes: 0,
            total_bytes: 0,
            sync_duration: Duration::from_secs(0),
        };

        if response.snapshot_to_download.is_some() {
            if let Some(snapshot) = snapshot_data {
                self.restore_from_snapshot(snapshot.clone())?;
                stats.snapshot_downloaded = true;
                stats.snapshot_size_bytes = snapshot.compressed_size;
            } else {
                return Err(SyncError::MissingSnapshot);
            }
        }

        for op in &response.operations_after_snapshot {
            self.apply_operation(op.clone())?;
            stats.operations_applied += 1;
        }

        stats.operations_size_bytes = response.estimated_size_bytes - stats.snapshot_size_bytes;
        stats.total_bytes = stats.snapshot_size_bytes + stats.operations_size_bytes;
        stats.sync_duration = start.elapsed();

        Ok(stats)
    }

    fn find_common_snapshot(&self, request: &SyncRequest) -> Option<&Snapshot> {
        if let Some(req_snap_id) = &request.latest_snapshot_id {
            self.local_snapshots
                .iter()
                .rev()
                .find(|s| &s.id == req_snap_id)
        } else {
            None
        }
    }

    fn get_operations_after_snapshot(&self, _snapshot_id: &str) -> Vec<CausalOp> {
        // Return recent operations (simplified)
        self.local_operations.clone()
    }

    fn restore_from_snapshot(&mut self, snapshot: Snapshot) -> Result<(), SyncError> {
        self.local_operations.clear();
        self.local_snapshots.push(snapshot);
        Ok(())
    }

    fn apply_operation(&mut self, op: CausalOp) -> Result<(), SyncError> {
        self.local_operations.push(op);
        Ok(())
    }

    fn estimate_size(&self, ops: &[CausalOp]) -> usize {
        let json_size: usize = ops
            .iter()
            .map(|op| serde_json::to_string(op).unwrap().len())
            .sum();
        
        if self.compression_enabled {
            json_size / 5  // ~80% compression with gzip
        } else {
            json_size
        }
    }

    fn estimate_snapshot_size(&self, snapshot: Option<&Snapshot>) -> usize {
        snapshot.map(|s| s.compressed_size).unwrap_or(0)
    }

    /// Calculate bandwidth savings
    pub fn calculate_savings(&self, stats: &SyncStats, total_operations: usize) -> SyncSavings {
        let full_replication_size = total_operations * 500;
        let hierarchical_size = stats.total_bytes;
        
        let bandwidth_saved = full_replication_size.saturating_sub(hierarchical_size);
        let savings_percent = if full_replication_size > 0 {
            (bandwidth_saved as f64 / full_replication_size as f64) * 100.0
        } else {
            0.0
        };

        SyncSavings {
            full_replication_bytes: full_replication_size,
            hierarchical_bytes: hierarchical_size,
            bytes_saved: bandwidth_saved,
            savings_percent,
        }
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.local_snapshots.len()
    }

    /// Get operation count
    pub fn operation_count(&self) -> usize {
        self.local_operations.len()
    }
}

/// Adaptive sync chooses strategy based on partition duration
pub struct AdaptiveSync {
    hierarchical: HierarchicalSync,
    partition_threshold: Duration,
}

impl AdaptiveSync {
    pub fn new(batch_size: usize, partition_threshold: Duration) -> Self {
        Self {
            hierarchical: HierarchicalSync::new(batch_size, true),
            partition_threshold,
        }
    }

    /// Choose sync strategy based on partition duration
    pub fn sync_strategy(&self, last_sync: Instant) -> SyncStrategy {
        if last_sync.elapsed() > self.partition_threshold {
            SyncStrategy::Hierarchical
        } else {
            SyncStrategy::MerkleDiff
        }
    }

    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.hierarchical.add_snapshot(snapshot);
    }

    pub fn add_operations(&mut self, ops: Vec<CausalOp>) {
        self.hierarchical.add_operations(ops);
    }

    pub fn prepare_request(
        &self,
        node_id: String,
        version_vector: VersionVector,
        merkle_root: String,
    ) -> SyncRequest {
        self.hierarchical.prepare_sync_request(node_id, version_vector, merkle_root)
    }

    pub fn handle_request(&self, request: SyncRequest) -> SyncResponse {
        self.hierarchical.handle_sync_request(request)
    }

    pub fn apply_response(
        &mut self,
        response: SyncResponse,
        snapshot_data: Option<Snapshot>,
    ) -> Result<SyncStats, SyncError> {
        self.hierarchical.apply_sync_response(response, snapshot_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_strategy_selection() {
        let sync = AdaptiveSync::new(1000, Duration::from_secs(3600));

        // Short partition
        let recent = Instant::now() - Duration::from_secs(60);
        assert_eq!(sync.sync_strategy(recent), SyncStrategy::MerkleDiff);

        // Long partition
        let old = Instant::now() - Duration::from_secs(7200);
        assert_eq!(sync.sync_strategy(old), SyncStrategy::Hierarchical);
    }

    #[test]
    fn test_savings_calculation() {
        let sync = HierarchicalSync::new(1000, true);
        
        let stats = SyncStats {
            snapshot_downloaded: true,
            snapshot_size_bytes: 50_000_000,
            operations_applied: 1000,
            operations_size_bytes: 5_000_000,
            total_bytes: 55_000_000,
            sync_duration: Duration::from_secs(60),
        };

        let savings = sync.calculate_savings(&stats, 2_500_000);
        assert!(savings.savings_percent > 95.0);
    }
}
