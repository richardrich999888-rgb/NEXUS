// Storage Layer - Persistent storage with RocksDB
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

#[cfg(feature = "storage")]
use rocksdb::{DB, Options, IteratorMode, WriteBatch};
use crate::causal_op::CausalOp;
use crate::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

// ============================================================================
// STORAGE ERRORS
// ============================================================================

#[derive(Debug, Clone)]
pub enum StorageError {
    DatabaseError(String),
    SerializationError(String),
    NotFound(String),
    InvalidKey(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::NotFound(key) => write!(f, "Key not found: {}", key),
            StorageError::InvalidKey(key) => write!(f, "Invalid key: {}", key),
        }
    }
}

impl std::error::Error for StorageError {}

// ============================================================================
// ROCKSDB STORAGE BACKEND
// ============================================================================

#[cfg(feature = "storage")]
pub struct RocksDBStorage {
    db: Arc<DB>,
}

#[cfg(feature = "storage")]
impl RocksDBStorage {
    /// Open or create a RocksDB database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let db = DB::open(&opts, path)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Store an operation
    pub fn put_operation(&self, op: &CausalOp) -> Result<(), StorageError> {
        let key = format!("op:{}", op.id);
        let value = serde_json::to_vec(op)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.put(key.as_bytes(), value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Get an operation by ID
    pub fn get_operation(&self, op_id: &str) -> Result<CausalOp, StorageError> {
        let key = format!("op:{}", op_id);
        let value = self.db.get(key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .ok_or_else(|| StorageError::NotFound(op_id.to_string()))?;
        
        let op: CausalOp = serde_json::from_slice(&value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        Ok(op)
    }

    /// Store a snapshot
    pub fn put_snapshot(&self, snapshot_id: &str, snapshot: &Snapshot) -> Result<(), StorageError> {
        let key = format!("snapshot:{}", snapshot_id);
        let value = serde_json::to_vec(snapshot)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.put(key.as_bytes(), value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Get a snapshot by ID
    pub fn get_snapshot(&self, snapshot_id: &str) -> Result<Snapshot, StorageError> {
        let key = format!("snapshot:{}", snapshot_id);
        let value = self.db.get(key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .ok_or_else(|| StorageError::NotFound(snapshot_id.to_string()))?;
        
        let snapshot: Snapshot = serde_json::from_slice(&value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        Ok(snapshot)
    }

    /// List all snapshot IDs
    pub fn list_snapshots(&self) -> Result<Vec<String>, StorageError> {
        let prefix = b"snapshot:";
        let mut snapshot_ids = Vec::new();
        
        let iter = self.db.iterator(IteratorMode::From(prefix, rocksdb::Direction::Forward));
        
        for item in iter {
            let (key, _) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let key_str = String::from_utf8_lossy(&key);
            
            if !key_str.starts_with("snapshot:") {
                break;
            }
            
            if let Some(id) = key_str.strip_prefix("snapshot:") {
                snapshot_ids.push(id.to_string());
            }
        }
        
        Ok(snapshot_ids)
    }

    /// Delete an operation
    pub fn delete_operation(&self, op_id: &str) -> Result<(), StorageError> {
        let key = format!("op:{}", op_id);
        self.db.delete(key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&self, snapshot_id: &str) -> Result<(), StorageError> {
        let key = format!("snapshot:{}", snapshot_id);
        self.db.delete(key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Batch write operations (atomic)
    pub fn batch_write_operations(&self, ops: &[CausalOp]) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();
        
        for op in ops {
            let key = format!("op:{}", op.id);
            let value = serde_json::to_vec(op)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            batch.put(key.as_bytes(), value);
        }
        
        self.db.write(batch)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Compact database to reclaim space
    pub fn compact(&self) -> Result<(), StorageError> {
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Option<String> {
        self.db.property_value("rocksdb.stats")
    }
}

// ============================================================================
// IN-MEMORY STORAGE (for testing without rocksdb feature)
// ============================================================================

#[cfg(not(feature = "storage"))]
pub struct InMemoryStorage {
    operations: std::collections::HashMap<String, String>,
    snapshots: std::collections::HashMap<String, String>,
}

#[cfg(not(feature = "storage"))]
impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            operations: std::collections::HashMap::new(),
            snapshots: std::collections::HashMap::new(),
        }
    }

    pub fn put_operation(&mut self, op: &CausalOp) -> Result<(), StorageError> {
        let value = serde_json::to_string(op)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.operations.insert(op.id.clone(), value);
        Ok(())
    }

    pub fn get_operation(&self, op_id: &str) -> Result<CausalOp, StorageError> {
        let value = self.operations.get(op_id)
            .ok_or_else(|| StorageError::NotFound(op_id.to_string()))?;
        
        let op: CausalOp = serde_json::from_str(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        Ok(op)
    }

    pub fn put_snapshot(&mut self, snapshot_id: &str, snapshot: &Snapshot) -> Result<(), StorageError> {
        let value = serde_json::to_string(snapshot)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.snapshots.insert(snapshot_id.to_string(), value);
        Ok(())
    }

    pub fn get_snapshot(&self, snapshot_id: &str) -> Result<Snapshot, StorageError> {
        let value = self.snapshots.get(snapshot_id)
            .ok_or_else(|| StorageError::NotFound(snapshot_id.to_string()))?;
        
        let snapshot: Snapshot = serde_json::from_str(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        Ok(snapshot)
    }

    pub fn list_snapshots(&self) -> Result<Vec<String>, StorageError> {
        Ok(self.snapshots.keys().cloned().collect())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version_vector::VersionVector;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use std::collections::BTreeSet;

    fn create_test_op(id: &str) -> CausalOp {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        let keypair = SigningKey::from_bytes(&bytes);
        let mut vv = VersionVector::new();
        vv.increment("node1");
        
        CausalOp::new(
            id.to_string(),
            serde_json::json!({"data": "test"}),
            BTreeSet::new(),
            vv,
            "node1".to_string(),
            &keypair,
        )
    }

    #[cfg(feature = "storage")]
    #[test]
    fn test_rocksdb_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = RocksDBStorage::open(temp_dir.path()).unwrap();
        
        let op = create_test_op("op1");
        
        // Put and get
        storage.put_operation(&op).unwrap();
        let retrieved = storage.get_operation("op1").unwrap();
        assert_eq!(retrieved.id, "op1");
        
        // Delete
        storage.delete_operation("op1").unwrap();
        assert!(storage.get_operation("op1").is_err());
    }

    #[cfg(feature = "storage")]
    #[test]
    fn test_batch_write() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = RocksDBStorage::open(temp_dir.path()).unwrap();
        
        let ops = vec![
            create_test_op("op1"),
            create_test_op("op2"),
            create_test_op("op3"),
        ];
        
        storage.batch_write_operations(&ops).unwrap();
        
        assert!(storage.get_operation("op1").is_ok());
        assert!(storage.get_operation("op2").is_ok());
        assert!(storage.get_operation("op3").is_ok());
    }

    #[cfg(not(feature = "storage"))]
    #[test]
    fn test_in_memory_storage() {
        let mut storage = InMemoryStorage::new();
        
        let op = create_test_op("op1");
        storage.put_operation(&op).unwrap();
        
        let retrieved = storage.get_operation("op1").unwrap();
        assert_eq!(retrieved.id, "op1");
    }
}
