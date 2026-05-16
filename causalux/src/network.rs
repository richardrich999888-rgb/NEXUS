// Network Layer - HTTP API and WebSocket for sync
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd

#[cfg(feature = "network")]
use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
#[cfg(feature = "network")]
use tower_http::trace::TraceLayer;
#[cfg(feature = "network")]
use std::sync::Arc;

use crate::sync::{SyncRequest, SyncResponse};
use crate::causal_op::CausalOp;
use crate::observability::HealthStatus;
use serde::{Deserialize, Serialize};

// ============================================================================
// API TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[cfg(feature = "network")]
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "INVALID_REQUEST" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "CONFLICT" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    pub operation: CausalOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    pub success: bool,
    pub operation_id: String,
}

// ============================================================================
// APP STATE
// ============================================================================

#[cfg(feature = "network")]
#[derive(Clone)]
pub struct AppState {
    // In production, this would contain DAG, storage, etc.
    pub node_id: String,
}

// ============================================================================
// HTTP API HANDLERS
// ============================================================================

#[cfg(feature = "network")]
async fn health_handler() -> Json<HealthStatus> {
    let mut health = HealthStatus::new();
    health.add_component(
        "api".to_string(),
        crate::observability::HealthState::Healthy,
        None,
    );
    Json(health)
}

#[cfg(feature = "network")]
async fn ready_handler() -> impl IntoResponse {
    // Check if system is ready to serve traffic
    let health = HealthStatus::new();
    
    if health.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

#[cfg(feature = "network")]
async fn submit_operation_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OperationRequest>,
) -> Result<Json<OperationResponse>, ApiError> {
    // In production: validate, insert into DAG, broadcast to peers
    tracing::info!(
        operation_id = %req.operation.id,
        node_id = %state.node_id,
        "Operation submitted"
    );
    
    Ok(Json(OperationResponse {
        success: true,
        operation_id: req.operation.id.clone(),
    }))
}

#[cfg(feature = "network")]
async fn sync_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, ApiError> {
    // In production: compute diff, return operations
    tracing::info!(
        node_id = %state.node_id,
        peer_version = ?req.version_vector,
        "Sync request received"
    );
    
    Ok(Json(SyncResponse {
        operations: vec![],
        snapshots: vec![],
        final_version: req.version_vector.clone(),
    }))
}

#[cfg(feature = "network")]
async fn metrics_handler() -> String {
    #[cfg(feature = "observability")]
    {
        use prometheus::TextEncoder;
        let encoder = TextEncoder::new();
        let registry = prometheus::default_registry();
        let metric_families = registry.gather();
        encoder.encode_to_string(&metric_families).unwrap()
    }
    
    #[cfg(not(feature = "observability"))]
    {
        "Metrics not enabled. Compile with --features observability".to_string()
    }
}

// ============================================================================
// HTTP SERVER
// ============================================================================

#[cfg(feature = "network")]
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health checks (Kubernetes probes)
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        
        // Operations API
        .route("/api/v1/operations", post(submit_operation_handler))
        
        // Sync API
        .route("/api/v1/sync", post(sync_handler))
        
        // Metrics (Prometheus)
        .route("/metrics", get(metrics_handler))
        
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}

#[cfg(feature = "network")]
pub async fn start_server(node_id: String, port: u16) -> Result<(), std::io::Error> {
    use tokio::net::TcpListener;
    
    let state = AppState { node_id: node_id.clone() };
    let app = create_router(state);
    
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting CAUSALUX HTTP server on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(all(test, feature = "network"))]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = AppState {
            node_id: "test-node".to_string(),
        };
        let app = create_router(state);
        
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let state = AppState {
            node_id: "test-node".to_string(),
        };
        let app = create_router(state);
        
        let response = app
            .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
}
