//! Resource limits and metering for NEXUS execution.
//!
//! Enforces bounded resource usage to ensure:
//! - Denial-of-service prevention
//! - Deterministic execution costs
//! - Fair resource allocation
//! - Predictable performance

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Resource limits for a PCU execution.
///
/// These limits are enforced by the executor runtime.
/// If any limit is exceeded, execution is terminated immediately.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Maximum execution time.
    pub max_time: Duration,

    /// Maximum fuel (CPU instructions).
    ///
    /// Fuel is the most deterministic way to measure CPU usage.
    pub max_fuel: u64,

    /// Maximum memory usage (bytes).
    pub max_memory: usize,

    /// Maximum output size (bytes).
    pub max_output: usize,

    /// Optional: Maximum stack depth.
    pub max_stack_depth: Option<u32>,
}

impl ExecutionLimits {
    /// Create new execution limits.
    pub fn new(max_time: Duration, max_fuel: u64, max_memory: usize, max_output: usize) -> Self {
        Self {
            max_time,
            max_fuel,
            max_memory,
            max_output,
            max_stack_depth: None,
        }
    }

    /// Default limits for a standard PCU.
    ///
    /// - 30 seconds max time
    /// - 1 billion instructions
    /// - 256 MB memory
    /// - 64 MB output
    pub const fn standard() -> Self {
        Self {
            max_time: Duration::from_secs(30),
            max_fuel: 1_000_000_000,
            max_memory: 256 * 1024 * 1024,
            max_output: 64 * 1024 * 1024,
            max_stack_depth: Some(1024),
        }
    }

    /// Restricted limits (for untrusted or low-priority tasks).
    pub const fn restricted() -> Self {
        Self {
            max_time: Duration::from_secs(5),
            max_fuel: 100_000_000,
            max_memory: 64 * 1024 * 1024,
            max_output: 1 * 1024 * 1024,
            max_stack_depth: Some(512),
        }
    }

    /// Minimal limits for testing.
    pub const fn minimal() -> Self {
        Self {
            max_time: Duration::from_millis(500),
            max_fuel: 1_000_000,
            max_memory: 1 * 1024 * 1024,
            max_output: 64 * 1024,
            max_stack_depth: Some(128),
        }
    }

    /// Enterprise limits (for high-performance tasks).
    pub const fn enterprise() -> Self {
        Self {
            max_time: Duration::from_secs(300),
            max_fuel: 10_000_000_000,
            max_memory: 2 * 1024 * 1024 * 1024, // 2 GB
            max_output: 512 * 1024 * 1024,
            max_stack_depth: Some(2048),
        }
    }
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self::standard()
    }
}
