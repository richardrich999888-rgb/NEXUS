// NEXUS Storage: Provenance Log
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use crate::error::NexusStorageError;
use nexus_core::causal::{CausalTensor, CausalId};
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;

pub struct ProvenanceLog {
    db: Arc<DB>,
}

impl ProvenanceLog {
    /// Opens a provenance log at the specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NexusStorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let db = DB::open(&opts, path).map_err(|e| NexusStorageError::ConnectionFailed(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Appends a causal tensor to the log
    pub fn append(&self, tensor: &CausalTensor) -> Result<(), NexusStorageError> {
        let key = tensor.id.as_bytes();
        let value = bincode::serialize(tensor).map_err(|e| NexusStorageError::SerializationFailed(e.to_string()))?;
        
        self.db.put(key, value).map_err(|e| NexusStorageError::WriteFailed(e.to_string()))?;
        
        Ok(())
    }

    /// Appends multiple causal tensors atomically
    pub fn append_batch(&self, tensors: &[CausalTensor]) -> Result<(), NexusStorageError> {
        let mut batch = WriteBatch::default();
        
        for tensor in tensors {
            let key = tensor.id.as_bytes();
            let value = bincode::serialize(tensor).map_err(|e| NexusStorageError::SerializationFailed(e.to_string()))?;
            batch.put(key, value);
        }
        
        self.db.write(batch).map_err(|e| NexusStorageError::WriteFailed(e.to_string()))?;
        
        Ok(())
    }

    /// Retrieves a causal tensor by its ID
    pub fn get(&self, id: &CausalId) -> Result<Option<CausalTensor>, NexusStorageError> {
        let res = self.db.get(id.as_bytes()).map_err(|e| NexusStorageError::ReadFailed(e.to_string()))?;
        
        match res {
            Some(bytes) => {
                let tensor = bincode::deserialize(&bytes).map_err(|e| NexusStorageError::SerializationFailed(e.to_string()))?;
                Ok(Some(tensor))
            }
            None => Ok(None),
        }
    }

    /// Checks if a tensor exists in the log
    pub fn exists(&self, id: &CausalId) -> Result<bool, NexusStorageError> {
        let res = self.db.get_pinned(id.as_bytes()).map_err(|e| NexusStorageError::ReadFailed(e.to_string()))?;
        Ok(res.is_some())
    }

    /// Returns the total number of tensors in the log (approximate)
    pub fn count_approximate(&self) -> usize {
        self.db.property_int_value("rocksdb.estimate-num-keys").unwrap_or(Some(0)).unwrap_or(0) as usize
    }
}
