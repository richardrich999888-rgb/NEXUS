// NEXUS Storage: Algebraic Index
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use crate::error::NexusStorageError;
use nexus_core::causal::{CausalId, CausalTensor};
use rocksdb::{DB, ColumnFamilyDescriptor, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;

pub const CF_DATA: &str = "tensors";
pub const CF_NODE_INDEX: &str = "node_index";
pub const CF_DEPTH_INDEX: &str = "depth_index";

pub struct AlgebraicIndex {
    db: Arc<DB>,
}

impl AlgebraicIndex {
    /// Opens the index at the specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NexusStorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new(CF_DATA, Options::default()),
            ColumnFamilyDescriptor::new(CF_NODE_INDEX, Options::default()),
            ColumnFamilyDescriptor::new(CF_DEPTH_INDEX, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| NexusStorageError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Indexes a tensor into all relevant column families
    pub fn index_tensor(&self, tensor: &CausalTensor) -> Result<(), NexusStorageError> {
        let mut batch = WriteBatch::default();
        let data_cf = self.db.cf_handle(CF_DATA).ok_or_else(|| NexusStorageError::IndexCorruption("Data CF missing".into()))?;
        let node_cf = self.db.cf_handle(CF_NODE_INDEX).ok_or_else(|| NexusStorageError::IndexCorruption("Node CF missing".into()))?;
        let depth_cf = self.db.cf_handle(CF_DEPTH_INDEX).ok_or_else(|| NexusStorageError::IndexCorruption("Depth CF missing".into()))?;

        let tensor_id = tensor.id.as_bytes();
        let tensor_data = bincode::serialize(tensor).map_err(|e| NexusStorageError::SerializationFailed(e.to_string()))?;

        // 1. Store main data
        batch.put_cf(&data_cf, tensor_id, tensor_data);

        // 2. Index by Node ID: key = node_id (u64) | tensor_id
        let mut node_key = Vec::with_capacity(8 + 32);
        node_key.extend_from_slice(&tensor.metadata.node_id.to_be_bytes());
        node_key.extend_from_slice(tensor_id);
        batch.put_cf(&node_cf, node_key, b"");

        // 3. Index by Depth: key = depth (u64) | tensor_id
        let mut depth_key = Vec::with_capacity(8 + 32);
        depth_key.extend_from_slice(&tensor.provenance.depth.to_be_bytes());
        depth_key.extend_from_slice(tensor_id);
        batch.put_cf(&depth_cf, depth_key, b"");

        self.db.write(batch).map_err(|e| NexusStorageError::WriteFailed(e.to_string()))?;
        Ok(())
    }

    /// Retrieves a tensor by ID
    pub fn get_tensor(&self, id: &CausalId) -> Result<Option<CausalTensor>, NexusStorageError> {
        let data_cf = self.db.cf_handle(CF_DATA).ok_or_else(|| NexusStorageError::IndexCorruption("Data CF missing".into()))?;
        let res = self.db.get_cf(&data_cf, id.as_bytes()).map_err(|e| NexusStorageError::ReadFailed(e.to_string()))?;
        
        match res {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes).map_err(|e| NexusStorageError::SerializationFailed(e.to_string()))?)),
            None => Ok(None),
        }
    }

    /// Returns all tensor IDs for a specific node
    pub fn get_by_node(&self, node_id: u64) -> Result<Vec<CausalId>, NexusStorageError> {
        let node_cf = self.db.cf_handle(CF_NODE_INDEX).ok_or_else(|| NexusStorageError::IndexCorruption("Node CF missing".into()))?;
        let prefix = node_id.to_be_bytes();
        
        let mut results = Vec::new();
        let iter = self.db.prefix_iterator_cf(&node_cf, prefix);
        
        for item in iter {
            let (key, _) = item.map_err(|e| NexusStorageError::ReadFailed(e.to_string()))?;
            if !key.starts_with(&prefix) { break; }
            if key.len() >= 40 {
                let id_bytes: [u8; 32] = key[8..40].try_into().map_err(|_| NexusStorageError::IndexCorruption("Invalid key length".into()))?;
                results.push(CausalId::from_bytes(id_bytes));
            }
        }
        
        Ok(results)
    }

    /// Returns all tensor IDs at a specific depth
    pub fn get_by_depth(&self, depth: u64) -> Result<Vec<CausalId>, NexusStorageError> {
        let depth_cf = self.db.cf_handle(CF_DEPTH_INDEX).ok_or_else(|| NexusStorageError::IndexCorruption("Depth CF missing".into()))?;
        let prefix = depth.to_be_bytes();
        
        let mut results = Vec::new();
        let iter = self.db.prefix_iterator_cf(&depth_cf, prefix);
        
        for item in iter {
            let (key, _) = item.map_err(|e| NexusStorageError::ReadFailed(e.to_string()))?;
            if !key.starts_with(&prefix) { break; }
            if key.len() >= 40 {
                let id_bytes: [u8; 32] = key[8..40].try_into().map_err(|_| NexusStorageError::IndexCorruption("Invalid key length".into()))?;
                results.push(CausalId::from_bytes(id_bytes));
            }
        }
        
        Ok(results)
    }
}
