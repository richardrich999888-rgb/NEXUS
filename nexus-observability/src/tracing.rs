//! Distributed tracing support (OpenTelemetry)

#[cfg(feature = "otel")]
use opentelemetry::global;
#[cfg(feature = "otel")]
use opentelemetry_sdk::{
    trace::{TracerProvider, TracerProviderBuilder},
    Resource,
};
#[cfg(feature = "otel")]
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
#[cfg(feature = "otel")]
use tracing::{info, error};
#[cfg(feature = "otel")]
use tracing_opentelemetry::OpenTelemetryLayer;
#[cfg(feature = "otel")]
use tracing_subscriber::layer::SubscriberExt;

/// Initialize OpenTelemetry tracing
#[cfg(feature = "otel")]
pub fn init_otel(service_name: &str, endpoint: Option<&str>) -> Result<(), anyhow::Error> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace::BatchConfig;

    let exporter = if let Some(endpoint) = endpoint {
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint)
    } else {
        // Default to local collector
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint("http://localhost:4317")
    };

    let resource = Resource::new(vec![
        SERVICE_NAME.string(service_name.to_string()),
    ]);

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, BatchConfig::default())
        .with_resource(resource)
        .build();

    global::set_tracer_provider(provider);

    let layer = OpenTelemetryLayer::new(global::tracer(service_name));
    
    tracing_subscriber::registry()
        .with(layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize OpenTelemetry: {}", e))?;

    info!(service = service_name, "OpenTelemetry tracing initialized");
    Ok(())
}

/// No-op implementation when OpenTelemetry is not enabled
#[cfg(not(feature = "otel"))]
pub fn init_otel(_service_name: &str, _endpoint: Option<&str>) -> Result<(), anyhow::Error> {
    Ok(())
}


