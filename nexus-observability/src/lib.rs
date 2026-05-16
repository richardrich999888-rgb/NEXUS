//! # NEXUS Observability
//!
//! Production-grade observability for NEXUS protocol:
//! - Metrics (Prometheus)
//! - Distributed tracing (OpenTelemetry)
//! - Structured logging
//! - Health checks

pub mod metrics;
#[cfg(feature = "otel")]
pub mod tracing;
pub mod health;
pub mod logging;

pub use metrics::NexusMetrics;
pub use health::{HealthCheck, HealthStatus, ComponentHealth};
pub use logging::init_logging;

/// Initialize full observability stack
pub fn init(service_name: &str) -> Result<NexusMetrics, anyhow::Error> {
    logging::init_logging(service_name)?;
    let metrics = metrics::NexusMetrics::new()?;
    Ok(metrics)
}

#[cfg(feature = "otel")]
pub fn init_with_otel(service_name: &str, otel_endpoint: Option<&str>) -> Result<NexusMetrics, anyhow::Error> {
    logging::init_logging(service_name)?;
    crate::tracing::init_otel(service_name, otel_endpoint)?;
    let metrics = metrics::NexusMetrics::new()?;
    Ok(metrics)
}

