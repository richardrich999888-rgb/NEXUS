//! Transparent Result Cache for NEXUS Executor.
//!
//! This module provides a simple wrapper for caching execution results.

use crate::types::ExecutionResult;
use lru::LruCache;
use std::num::NonZeroUsize;
use parking_lot::RwLock;
use nexus_pcu::ContentHash;

/// A simple result cache for NEXUS computations.
pub struct ResultCache {
    inner: RwLock<LruCache<ContentHash, ExecutionResult>>,
}

impl ResultCache {
    /// Create a new result cache with specified capacity.
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity)
            .or_else(|| NonZeroUsize::new(100))
            .expect("Failed to create cache: capacity must be > 0");
        Self {
            inner: RwLock::new(LruCache::new(cap)),
        }
    }

    /// Get a result from the cache.
    pub fn get(&self, key: &ContentHash) -> Option<ExecutionResult> {
        self.inner.write().get(key).cloned()
    }

    /// Insert a result into the cache.
    pub fn insert(&self, key: ContentHash, result: ExecutionResult) {
        self.inner.write().put(key, result);
    }
}

impl Default for ResultCache {
    fn default() -> Self {
        Self::new(1000)
    }
}
