// NEXUS Server - HTTP API for benchmarking and deployment
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

use causalux_v2::{ConflictPolicy, CausalDAG, VersionVector, CausalOp};
use nexus_pcu::{PrincipalId, USO};
use nexus_sync::NexusSyncEngine;

// ============================================================================
// Application State
// ============================================================================

struct AppState {
    sync_engine: RwLock<NexusSyncEngine>,
    node_id: String,
    start_time: Instant,
}

// ============================================================================
// API Types
// ============================================================================

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    node_id: String,
    uptime_seconds: u64,
    version: &'static str,
}

#[derive(Serialize)]
struct BenchmarkResponse {
    operation: String,
    duration_us: u64,
    operations_per_second: f64,
    timestamp: u64,
}

#[derive(Serialize)]
struct SyncStatsResponse {
    operation_count: usize,
    version_vector: std::collections::HashMap<String, u64>,
}

#[derive(Deserialize)]
struct CreateUsoRequest {
    data: String,
}

#[derive(Serialize)]
struct CreateUsoResponse {
    id: String,
    duration_us: u64,
}

// ============================================================================
// Handlers
// ============================================================================

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        node_id: state.node_id.clone(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn benchmark_causal_merge(State(state): State<Arc<AppState>>) -> Json<BenchmarkResponse> {
    let iterations = 1000;
    let start = Instant::now();
    
    // Benchmark version vector merges
    for _ in 0..iterations {
        let mut vv1 = VersionVector::new();
        let mut vv2 = VersionVector::new();
        
        vv1.increment("node1");
        vv1.increment("node1");
        vv2.increment("node2");
        vv2.increment("node2");
        vv2.increment("node2");
        
        let _merged = vv1.merge(&vv2);
    }
    
    let duration = start.elapsed();
    let ops_per_sec = (iterations as f64) / duration.as_secs_f64();
    
    Json(BenchmarkResponse {
        operation: "causal_merge".to_string(),
        duration_us: duration.as_micros() as u64,
        operations_per_second: ops_per_sec,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

async fn benchmark_uso_creation(State(state): State<Arc<AppState>>) -> Json<BenchmarkResponse> {
    let iterations = 1000;
    let start = Instant::now();
    
    // Benchmark USO creation
    for i in 0..iterations {
        let data = format!("test_data_{}", i);
        let _uso = USO::new(data.as_bytes().to_vec(), PrincipalId::generate());
    }
    
    let duration = start.elapsed();
    let ops_per_sec = (iterations as f64) / duration.as_secs_f64();
    
    Json(BenchmarkResponse {
        operation: "uso_creation".to_string(),
        duration_us: duration.as_micros() as u64,
        operations_per_second: ops_per_sec,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

async fn get_sync_stats(State(state): State<Arc<AppState>>) -> Json<SyncStatsResponse> {
    let engine = state.sync_engine.read().await;
    let vv = engine.version_vector();
    
    Json(SyncStatsResponse {
        operation_count: engine.operation_count(),
        version_vector: vv.versions.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect(),
    })
}

async fn create_uso(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUsoRequest>,
) -> Json<CreateUsoResponse> {
    let start = Instant::now();
    
    let uso = USO::new(req.data.as_bytes().to_vec(), PrincipalId::generate());
    let id = hex::encode(uso.id.as_bytes());
    
    // Register with sync engine
    let mut engine = state.sync_engine.write().await;
    engine.register_uso(uso);
    
    Json(CreateUsoResponse {
        id,
        duration_us: start.elapsed().as_micros() as u64,
    })
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| "nexus-node-1".to_string());
    
    info!("Starting NEXUS Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Node ID: {}", node_id);
    
    // Create sync engine
    let sync_engine = NexusSyncEngine::new(&node_id, ConflictPolicy::LastWriterWins);
    
    let state = Arc::new(AppState {
        sync_engine: RwLock::new(sync_engine),
        node_id: node_id.clone(),
        start_time: Instant::now(),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/api/benchmark/causal-merge", get(benchmark_causal_merge))
        .route("/api/benchmark/uso-creation", get(benchmark_uso_creation))
        .route("/api/sync/stats", get(get_sync_stats))
        .route("/api/uso", post(create_uso))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
