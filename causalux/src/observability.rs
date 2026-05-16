// Observability Layer - Production-grade logging and metrics
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug, instrument};
use serde::{Deserialize, Serialize};

#[cfg(feature = "observability")]
use prometheus::{
    Registry, Counter, Histogram, HistogramOpts, Opts,
    register_counter_with_registry, register_histogram_with_registry,
};

// ============================================================================
// AUDIT LOGGING
// ============================================================================

/// Audit log entry for compliance and forensics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp (microseconds since epoch)
    pub timestamp: u64,
    
    /// User/node that performed the operation
    pub actor_id: String,
    
    /// Operation type
    pub operation: AuditOperation,
    
    /// Operation ID (for correlation)
    pub operation_id: String,
    
    /// Document / resource affected
    pub resource_id: String,
    
    /// Result (success, failure)
    pub result: AuditResult,
    
    /// Additional metadata
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOperation {
    ReadDocument,
    WriteDocument,
    DeleteDocument,
    ShareDocument,
    RevokeSharing,
    SyncOperation,
    CreateSnapshot,
    RestoreSnapshot,
    EncryptData,
    DecryptData,
    BFTValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { reason: String },
}

impl AuditLogEntry {
    pub fn new(
        actor_id: String,
        operation: AuditOperation,
        operation_id: String,
        resource_id: String,
        result: AuditResult,
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            actor_id,
            operation,
            operation_id,
            resource_id,
            result,
            metadata: serde_json::json!({}),
        }
    }

    /// Log this entry to structured logger
    pub fn log(&self) {
        match &self.result {
            AuditResult::Success => {
                info!(
                    actor = %self.actor_id,
                    operation = ?self.operation,
                    operation_id = %self.operation_id,
                    resource_id = %self.resource_id,
                    "Audit: Operation succeeded"
                );
            }
            AuditResult::Failure { reason } => {
                warn!(
                    actor = %self.actor_id,
                    operation = ?self.operation,
                    operation_id = %self.operation_id,
                    resource_id = %self.resource_id,
                    reason = %reason,
                    "Audit: Operation failed"
                );
            }
        }
    }

    /// Export as JSON (for compliance reports)
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// ============================================================================
// METRICS
// ============================================================================

#[cfg(feature = "observability")]
pub struct CausaluxMetrics {
    /// Total operations processed
    pub operations_total: Counter,
    
    /// Operation latency histogram
    pub operation_latency: Histogram,
    
    /// Conflicts detected
    pub conflicts_total: Counter,
    
    /// Snapshots created
    pub snapshots_total: Counter,
    
    /// Sync operations
    pub sync_operations_total: Counter,
    
    /// Sync bandwidth (bytes)
    pub sync_bandwidth_bytes: Histogram,
    
    /// BFT validations
    pub bft_validations_total: Counter,
    
    /// Active documents
    pub active_documents: prometheus::Gauge,
}

#[cfg(feature = "observability")]
impl CausaluxMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let operations_total = register_counter_with_registry!(
            "causalux_operations_total",
            "Total number of operations processed",
            registry
        )?;

        let operation_latency = register_histogram_with_registry!(
            "causalux_operation_latency_seconds",
            "Operation processing latency in seconds",
            vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0],
            registry
        )?;

        let conflicts_total = register_counter_with_registry!(
            "causalux_conflicts_total",
            "Total number of conflicts detected",
            registry
        )?;

        let snapshots_total = register_counter_with_registry!(
            "causalux_snapshots_total",
            "Total number of snapshots created",
            registry
        )?;

        let sync_operations_total = register_counter_with_registry!(
            "causalux_sync_operations_total",
            "Total number of sync operations",
            registry
        )?;

        let sync_bandwidth_bytes = register_histogram_with_registry!(
            "causalux_sync_bandwidth_bytes",
            "Sync bandwidth in bytes",
            vec![1024.0, 10240.0, 102400.0, 1024000.0, 10240000.0],
            registry
        )?;

        let bft_validations_total = register_counter_with_registry!(
            "causalux_bft_validations_total",
            "Total number of BFT validations",
            registry
        )?;

        let active_documents = prometheus::register_gauge_with_registry!(
            "causalux_active_documents",
            "Number of active documents",
            registry
        )?;

        Ok(Self {
            operations_total,
            operation_latency,
            conflicts_total,
            snapshots_total,
            sync_operations_total,
            sync_bandwidth_bytes,
            bft_validations_total,
            active_documents,
        })
    }
}

// ============================================================================
// HEALTH CHECKS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall status
    pub status: HealthState,
    
    /// Component statuses
    pub components: Vec<ComponentHealth>,
    
    /// Timestamp
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthState,
    pub message: Option<String>,
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            status: HealthState::Healthy,
            components: vec![],
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn add_component(&mut self, name: String, status: HealthState, message: Option<String>) {
        self.components.push(ComponentHealth {
            name,
            status: status.clone(),
            message,
        });

        // Overall status is the worst of all components
        if status == HealthState::Unhealthy {
            self.status = HealthState::Unhealthy;
        } else if status == HealthState::Degraded && self.status != HealthState::Unhealthy {
            self.status = HealthState::Degraded;
        }
    }

    /// Check if system is ready to serve traffic
    pub fn is_ready(&self) -> bool {
        self.status == HealthState::Healthy || self.status == HealthState::Degraded
    }

    /// Check if system is alive (for liveness probe)
    pub fn is_alive(&self) -> bool {
        self.status != HealthState::Unhealthy
    }
}

// ============================================================================
// INITIALIZATION
// ============================================================================

/// Initialize observability stack (logging + metrics)
pub fn init_observability() {
    // Initialize tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("CAUSALUX observability initialized");
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry() {
        let entry = AuditLogEntry::new(
            "alice".to_string(),
            AuditOperation::WriteDocument,
            "op123".to_string(),
            "doc456".to_string(),
            AuditResult::Success,
        );

        assert_eq!(entry.actor_id, "alice");
        assert!(entry.timestamp > 0);

        let json = entry.to_json();
        assert!(json.contains("alice"));
        assert!(json.contains("WriteDocument"));
    }

    #[test]
    fn test_health_status() {
        let mut health = HealthStatus::new();
        assert_eq!(health.status, HealthState::Healthy);
        assert!(health.is_ready());

        health.add_component(
            "storage".to_string(),
            HealthState::Degraded,
            Some("Disk usage at 80%".to_string()),
        );
        assert_eq!(health.status, HealthState::Degraded);
        assert!(health.is_ready());

        health.add_component(
            "network".to_string(),
            HealthState::Unhealthy,
            Some("Connection lost".to_string()),
        );
        assert_eq!(health.status, HealthState::Unhealthy);
        assert!(!health.is_ready());
    }

    #[cfg(feature = "observability")]
    #[test]
    fn test_metrics() {
        let registry = Registry::new();
        let metrics = CausaluxMetrics::new(&registry).unwrap();

        metrics.operations_total.inc();
        assert_eq!(metrics.operations_total.get(), 1.0);

        metrics.conflicts_total.inc();
        assert_eq!(metrics.conflicts_total.get(), 1.0);
    }
}
