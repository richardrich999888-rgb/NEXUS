// NEXUS Core: Production-Grade Causal Tensor Algebra
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX
// Inventor: Katta Naga Sri Ganesh

pub mod causal;
pub mod cost_optimizer;
pub mod crypto;
pub mod error;
#[cfg(feature = "migration")]
pub mod migration;
pub mod tenancy;

// Re-export core types
pub use causal::{
    CausalId, CausalTensor, VectorClock, Provenance,
    TensorMetadata, TensorFlags, ConsistencyLevel,
};
pub use error::{NexusError, Result};
pub use tenancy::{TenantManager, TenantId, Tenant, TenantQuotas, TenantUsage, QuotaOperation, TenancyError};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lib_exports() {
        // Verify all core types are accessible
        let _clock: VectorClock = VectorClock::new();
    }
}

