//! Structured logging initialization

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing::{info};

/// Initialize structured logging
pub fn init_logging(service_name: &str) -> Result<(), anyhow::Error> {
    // Use RUST_LOG environment variable if set, otherwise default to INFO
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::Layer::default()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
                .json() // Structured JSON logging for production
        )
        .init();

    info!(service = service_name, "Logging initialized");
    Ok(())
}


