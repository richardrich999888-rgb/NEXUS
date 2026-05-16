// NEXUS Storage: Query Patterns
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use crate::AlgebraicIndex;
use crate::error::NexusStorageError;
use nexus_core::causal::CausalId;

#[derive(Debug, Clone)]
pub enum QueryPattern {
    All,
    ByNode(u64),
    AtDepth(u64),
    DepthRange(u64, u64),
    Specific(CausalId),
}

impl QueryPattern {
    /// Executes the query against the provided index
    pub fn execute(&self, index: &AlgebraicIndex) -> Result<Vec<CausalId>, NexusStorageError> {
        match self {
            QueryPattern::All => {
                // In a real implementation, this might be very large, so we might want pagination
                // For now, we'll return all IDs from the data CF
                // (Note: This is a simplified version)
                Ok(index.get_by_depth(0)?) // Start with depth 0 as a placeholder for "all" is not trivial without a full scan
            }
            QueryPattern::ByNode(node_id) => index.get_by_node(*node_id),
            QueryPattern::AtDepth(depth) => index.get_by_depth(*depth),
            QueryPattern::Specific(id) => {
                if index.get_tensor(id)?.is_some() {
                    Ok(vec![*id])
                } else {
                    Ok(vec![])
                }
            }
            QueryPattern::DepthRange(start, end) => {
                let mut results = Vec::new();
                for d in *start..=*end {
                    results.extend(index.get_by_depth(d)?);
                }
                Ok(results)
            }
        }
    }
}
