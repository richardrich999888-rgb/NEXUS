//! Prometheus metrics for NEXUS protocol

use prometheus::{
    Registry, Counter, Histogram, Gauge, HistogramOpts, Opts,
    register_counter_with_registry, register_histogram_with_registry,
    register_gauge_with_registry,
};
use std::sync::Arc;
use tracing::{info, error};

/// NEXUS metrics collection
pub struct NexusMetrics {
    /// Registry for all metrics
    pub registry: Arc<Registry>,

    // PCU Execution Metrics
    /// Total PCU executions
    pub pcu_executions_total: Counter,
    /// PCU execution latency (seconds)
    pub pcu_execution_duration: Histogram,
    /// PCU execution failures
    pub pcu_execution_failures: Counter,
    /// PCU cache hits
    pub pcu_cache_hits: Counter,
    /// PCU cache misses
    pub pcu_cache_misses: Counter,

    // Network Metrics
    /// Total messages sent
    pub network_messages_sent: Counter,
    /// Total messages received
    pub network_messages_received: Counter,
    /// Message send latency (seconds)
    pub network_send_duration: Histogram,
    /// Message size (bytes)
    pub network_message_size: Histogram,
    /// Connection failures
    pub network_connection_failures: Counter,
    /// Rate limit rejections
    pub network_rate_limit_rejections: Counter,

    // Sync Metrics
    /// Total sync operations
    pub sync_operations_total: Counter,
    /// Sync operation latency (seconds)
    pub sync_operation_duration: Histogram,
    /// Sync conflicts detected
    pub sync_conflicts: Counter,
    /// Sync bytes transferred
    pub sync_bytes_transferred: Histogram,

    // Storage Metrics
    /// Storage read operations
    pub storage_reads: Counter,
    /// Storage write operations
    pub storage_writes: Counter,
    /// Storage read latency (seconds)
    pub storage_read_duration: Histogram,
    /// Storage write latency (seconds)
    pub storage_write_duration: Histogram,
    /// Storage size (bytes)
    pub storage_size_bytes: Gauge,

    // Resource Metrics
    /// Current active PCU executions
    pub active_pcu_executions: Gauge,
    /// Current memory usage (bytes)
    pub memory_usage_bytes: Gauge,
    /// Current CPU usage (percentage)
    pub cpu_usage_percent: Gauge,
}

impl NexusMetrics {
    /// Create new metrics registry
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // PCU Execution Metrics
        let pcu_executions_total = register_counter_with_registry!(
            Opts::new("nexus_pcu_executions_total", "Total number of PCU executions")
                .namespace("nexus"),
            registry
        )?;

        let pcu_execution_duration = register_histogram_with_registry!(
            HistogramOpts::new("nexus_pcu_execution_duration_seconds", "PCU execution duration in seconds")
                .namespace("nexus")
                .buckets(vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
            registry
        )?;

        let pcu_execution_failures = register_counter_with_registry!(
            Opts::new("nexus_pcu_execution_failures_total", "Total number of PCU execution failures")
                .namespace("nexus"),
            registry
        )?;

        let pcu_cache_hits = register_counter_with_registry!(
            Opts::new("nexus_pcu_cache_hits_total", "Total number of PCU cache hits")
                .namespace("nexus"),
            registry
        )?;

        let pcu_cache_misses = register_counter_with_registry!(
            Opts::new("nexus_pcu_cache_misses_total", "Total number of PCU cache misses")
                .namespace("nexus"),
            registry
        )?;

        // Network Metrics
        let network_messages_sent = register_counter_with_registry!(
            Opts::new("nexus_network_messages_sent_total", "Total number of network messages sent")
                .namespace("nexus"),
            registry
        )?;

        let network_messages_received = register_counter_with_registry!(
            Opts::new("nexus_network_messages_received_total", "Total number of network messages received")
                .namespace("nexus"),
            registry
        )?;

        let network_send_duration = register_histogram_with_registry!(
            HistogramOpts::new("nexus_network_send_duration_seconds", "Network message send duration in seconds")
                .namespace("nexus")
                .buckets(vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0]),
            registry
        )?;

        let network_message_size = register_histogram_with_registry!(
            HistogramOpts::new("nexus_network_message_size_bytes", "Network message size in bytes")
                .namespace("nexus")
                .buckets(vec![1024.0, 10240.0, 102400.0, 1024000.0, 10240000.0]),
            registry
        )?;

        let network_connection_failures = register_counter_with_registry!(
            Opts::new("nexus_network_connection_failures_total", "Total number of network connection failures")
                .namespace("nexus"),
            registry
        )?;

        let network_rate_limit_rejections = register_counter_with_registry!(
            Opts::new("nexus_network_rate_limit_rejections_total", "Total number of rate limit rejections")
                .namespace("nexus"),
            registry
        )?;

        // Sync Metrics
        let sync_operations_total = register_counter_with_registry!(
            Opts::new("nexus_sync_operations_total", "Total number of sync operations")
                .namespace("nexus"),
            registry
        )?;

        let sync_operation_duration = register_histogram_with_registry!(
            HistogramOpts::new("nexus_sync_operation_duration_seconds", "Sync operation duration in seconds")
                .namespace("nexus")
                .buckets(vec![0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0]),
            registry
        )?;

        let sync_conflicts = register_counter_with_registry!(
            Opts::new("nexus_sync_conflicts_total", "Total number of sync conflicts detected")
                .namespace("nexus"),
            registry
        )?;

        let sync_bytes_transferred = register_histogram_with_registry!(
            HistogramOpts::new("nexus_sync_bytes_transferred", "Sync bytes transferred")
                .namespace("nexus")
                .buckets(vec![1024.0, 10240.0, 102400.0, 1024000.0, 10240000.0, 104857600.0]),
            registry
        )?;

        // Storage Metrics
        let storage_reads = register_counter_with_registry!(
            Opts::new("nexus_storage_reads_total", "Total number of storage read operations")
                .namespace("nexus"),
            registry
        )?;

        let storage_writes = register_counter_with_registry!(
            Opts::new("nexus_storage_writes_total", "Total number of storage write operations")
                .namespace("nexus"),
            registry
        )?;

        let storage_read_duration = register_histogram_with_registry!(
            HistogramOpts::new("nexus_storage_read_duration_seconds", "Storage read duration in seconds")
                .namespace("nexus")
                .buckets(vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0]),
            registry
        )?;

        let storage_write_duration = register_histogram_with_registry!(
            HistogramOpts::new("nexus_storage_write_duration_seconds", "Storage write duration in seconds")
                .namespace("nexus")
                .buckets(vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0]),
            registry
        )?;

        let storage_size_bytes = register_gauge_with_registry!(
            Opts::new("nexus_storage_size_bytes", "Current storage size in bytes")
                .namespace("nexus"),
            registry
        )?;

        // Resource Metrics
        let active_pcu_executions = register_gauge_with_registry!(
            Opts::new("nexus_active_pcu_executions", "Current number of active PCU executions")
                .namespace("nexus"),
            registry
        )?;

        let memory_usage_bytes = register_gauge_with_registry!(
            Opts::new("nexus_memory_usage_bytes", "Current memory usage in bytes")
                .namespace("nexus"),
            registry
        )?;

        let cpu_usage_percent = register_gauge_with_registry!(
            Opts::new("nexus_cpu_usage_percent", "Current CPU usage percentage")
                .namespace("nexus"),
            registry
        )?;

        info!("NEXUS metrics initialized");

        Ok(Self {
            registry,
            pcu_executions_total,
            pcu_execution_duration,
            pcu_execution_failures,
            pcu_cache_hits,
            pcu_cache_misses,
            network_messages_sent,
            network_messages_received,
            network_send_duration,
            network_message_size,
            network_connection_failures,
            network_rate_limit_rejections,
            sync_operations_total,
            sync_operation_duration,
            sync_conflicts,
            sync_bytes_transferred,
            storage_reads,
            storage_writes,
            storage_read_duration,
            storage_write_duration,
            storage_size_bytes,
            active_pcu_executions,
            memory_usage_bytes,
            cpu_usage_percent,
        })
    }

    /// Get Prometheus metrics in text format
    pub fn gather_metrics(&self) -> Result<String, prometheus::Error> {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

impl Default for NexusMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default metrics")
    }
}


