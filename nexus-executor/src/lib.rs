//! # NEXUS Executor
//!
//! Production-grade WASM executor for NEXUS Portable Computation Units (PCUs).
//!
//! ## Overview
//!
//! The NEXUS Executor provides secure, sandboxed execution of WebAssembly modules
//! with the following guarantees:
//!
//! - **Deterministic execution**: Same inputs always produce same outputs
//! - **Resource bounded**: CPU (fuel), memory, and time limits enforced
//! - **Identity-aware**: Capability-based access control embedded in execution
//! - **Provable**: Cryptographic proofs of correct execution
//! - **Secure**: WASM sandboxing prevents unauthorized system access
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     NEXUS EXECUTOR                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
//! │  │    PCU      │  │  Identity   │  │   Proof     │        │
//! │  │  Executor   │──│  Verifier   │──│  Generator  │        │
//! │  └─────────────┘  └─────────────┘  └─────────────┘        │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │              WASMTIME RUNTIME                        │   │
//! │  │  ┌─────────┐  ┌─────────┐  ┌─────────┐            │   │
//! │  │  │  Fuel   │  │ Memory  │  │  Host   │            │   │
//! │  │  │ Metering│  │ Limits  │  │Functions│            │   │
//! │  │  └─────────┘  └─────────┘  └─────────┘            │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │              RESULT CACHE                            │   │
//! │  │         (Content-addressed memoization)              │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexus_executor::{PcuExecutor, PCU, ExecutionContext, ExecutionLimits};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create executor
//!     let executor = PcuExecutor::new()?;
//!
//!     // Create PCU from WASM bytecode
//!     let pcu = PCU::new(wasm_bytes, identity, constraints)?;
//!
//!     // Execute with limits
//!     let context = ExecutionContext::new(inputs, identity, ExecutionLimits::default());
//!     let result = executor.execute(&pcu, context).await?;
//!
//!     // Result includes output and cryptographic proof
//!     println!("Output hash: {}", result.output_hash);
//!     println!("Proof valid: {}", result.proof.verify().is_ok());
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod cache;
pub mod error;
pub mod executor;
pub mod guard;
pub mod guards;
pub mod host_functions;
pub mod limits;
pub mod proof;
pub mod semantic_cache;
pub mod types;

// Re-exports for convenience
pub use cache::ResultCache;
pub use error::{ExecutorError, ExecutorResult};
pub use executor::{ExecutorBuilder, PcuExecutor};
pub use guard::{ExecutionGuard, GuardDecision};
pub use guards::{NervousSystemGuard, ImmuneGuard, CompositeGuard};
pub use limits::ExecutionLimits;
pub use proof::{ExecutionProof, NodeAttestation};
pub use nexus_pcu::NodeId;
pub use semantic_cache::{RoutingDecision, SemanticCache, SemanticCacheStats, SemanticKey};
pub use types::{ExecutionContext, ExecutionResult};

// Re-export core types from nexus-pcu (single source of truth)
// NOTE: All PCU, Identity, and ContentHash types come from nexus-pcu
pub use nexus_pcu::{
    ContentHash, ContentHasher,
    IdentityContext, PrincipalId, Capability, CapabilitySet, DelegationChain,
    PCU, WasmModule, ExecutionConstraints,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum supported WASM module size (16 MB)
pub const MAX_MODULE_SIZE: usize = 16 * 1024 * 1024;

/// Maximum supported output size (64 MB)
pub const MAX_OUTPUT_SIZE: usize = 64 * 1024 * 1024;

/// Interface for PCU interaction with the host environment.
pub trait NexusHost: Send + Sync {
    /// Read a Universal State Object (USO) by its content hash.
    fn uso_get(&self, id: &ContentHash) -> Result<Option<Vec<u8>>, ExecutorError>;

    /// Create or update a USO.
    fn uso_put(&self, data: &[u8]) -> Result<ContentHash, ExecutorError>;

    /// Apply a CRDT operation to a USO.
    fn uso_apply_op(&self, id: &ContentHash, op: &[u8]) -> Result<ContentHash, ExecutorError>;

    /// Log a message from the PCU.
    fn log(&self, level: u32, message: &str);

    /// Get current system time.
    fn get_time(&self) -> u64;

    /// Spawn a child PCU.
    fn spawn_pcu(&self, pcu: &PCU, inputs: Vec<(ContentHash, Vec<u8>)>) -> Result<ContentHash, ExecutorError>;
}

/// Fallback implementation of NexusHost that does nothing.
pub struct NoopHost;

impl NexusHost for NoopHost {
    fn uso_get(&self, _id: &ContentHash) -> Result<Option<Vec<u8>>, ExecutorError> {
        Ok(None)
    }

    fn uso_put(&self, data: &[u8]) -> Result<ContentHash, ExecutorError> {
        Ok(ContentHash::compute(data))
    }

    fn uso_apply_op(&self, id: &ContentHash, _op: &[u8]) -> Result<ContentHash, ExecutorError> {
        Ok(*id)
    }

    fn log(&self, _level: u32, _message: &str) {}

    fn get_time(&self) -> u64 {
        0
    }

    fn spawn_pcu(&self, _pcu: &PCU, _inputs: Vec<(ContentHash, Vec<u8>)>) -> Result<ContentHash, ExecutorError> {
        Ok(ContentHash::zero())
    }
}

/// Prelude module for common imports
pub mod prelude {
    pub use crate::cache::ResultCache;
    pub use crate::error::{ExecutorError, ExecutorResult};
    pub use crate::executor::{ExecutorBuilder, PcuExecutor};
    pub use crate::limits::ExecutionLimits;
    pub use crate::proof::{ExecutionProof, NodeAttestation};
    pub use nexus_pcu::NodeId;
    pub use crate::types::{ExecutionContext, ExecutionResult};
    
    // Re-export from nexus-pcu
    pub use nexus_pcu::{
        ContentHash, ContentHasher,
        IdentityContext, PrincipalId, Capability, CapabilitySet,
        PCU, WasmModule, ExecutionConstraints,
    };
}
