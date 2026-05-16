# NEXUS Protocol API Reference

**Version:** 1.0  
**Last Updated:** 2025-01-18

## Overview

This document provides a complete API reference for the NEXUS protocol components.

---

## Table of Contents

1. [Core Types](#core-types)
2. [PCU Execution](#pcu-execution)
3. [Network Transport](#network-transport)
4. [Storage](#storage)
5. [Observability](#observability)
6. [Secrets Management](#secrets-management)
7. [Multi-tenancy](#multi-tenancy)
8. [Backup & Recovery](#backup--recovery)

---

## Core Types

### PCU (Portable Computation Unit)

```rust
pub struct PCU {
    pub id: ContentHash,
    pub code: WasmModule,
    pub inputs: Vec<ContentHash>,
    pub identity: IdentityContext,
    pub constraints: ExecutionConstraints,
}

impl PCU {
    /// Create a new PCU
    pub fn new(
        code: WasmModule,
        inputs: Vec<ContentHash>,
        identity: IdentityContext,
        constraints: ExecutionConstraints,
    ) -> Self;

    /// Validate PCU structure and constraints
    pub fn validate(&self) -> Result<(), PcuValidationError>;

    /// Check if identity has required capabilities
    pub fn check_capabilities(&self) -> Result<(), Capability>;

    /// Compute content hash
    pub fn content_hash(&self) -> ContentHash;

    /// Compute semantic hash (for caching)
    pub fn semantic_hash(&self) -> ContentHash;

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error>;

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error>;
}
```

### IdentityContext

```rust
pub struct IdentityContext {
    pub principal: PrincipalId,
    pub capabilities: CapabilitySet,
    pub delegation: Option<DelegationChain>,
}

impl IdentityContext {
    /// Create anonymous identity (no capabilities)
    pub fn anonymous() -> Self;

    /// Check if identity has capability
    pub fn has_capability(&self, cap: &Capability) -> bool;

    /// Compute content hash
    pub fn content_hash(&self) -> ContentHash;
}
```

### ContentHash

```rust
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Compute BLAKE3 hash of data
    pub fn compute(data: &[u8]) -> Self;

    /// Convert to hex string
    pub fn to_hex(&self) -> String;

    /// Create from hex string
    pub fn from_hex(s: &str) -> Result<Self, ParseError>;
}
```

---

## PCU Execution

### PcuExecutor

```rust
pub struct PcuExecutor {
    // ...
}

impl PcuExecutor {
    /// Create new executor
    pub fn new(
        config: ExecutorConfig,
        metrics: Option<Arc<NexusMetrics>>,
    ) -> Self;

    /// Execute a PCU
    pub async fn execute(&self, pcu: &PCU) -> Result<ExecutionResult, ExecutionError>;
}
```

**Example:**
```rust
use nexus_executor::{PcuExecutor, ExecutorConfig};
use nexus_pcu::{PCU, WasmModule, IdentityContext, ExecutionConstraints};

let executor = PcuExecutor::new(ExecutorConfig::default(), None);
let wasm = WasmModule::new(wasm_bytes);
let pcu = PCU::new(
    wasm,
    vec![],
    IdentityContext::anonymous(),
    ExecutionConstraints::default(),
);

let result = executor.execute(&pcu).await?;
```

---

## Network Transport

### QuicTransport

```rust
pub struct QuicTransport {
    // ...
}

impl QuicTransport {
    /// Create transport with TLS configuration
    pub fn new(
        addr: SocketAddr,
        tls_config: TlsConfig,
    ) -> Result<Self, NexusNetworkError>;

    /// Create development transport (self-signed cert)
    pub fn new_dev(addr: SocketAddr, common_name: &str) -> Result<Self, NexusNetworkError>;

    /// Create production transport (certificate files)
    pub fn new_with_certs(
        addr: SocketAddr,
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
        client_ca_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, NexusNetworkError>;

    /// Connect to remote node
    pub async fn connect(
        &self,
        addr: SocketAddr,
        root_cas: Option<Vec<rustls::Certificate>>,
    ) -> Result<quinn::Connection, NexusNetworkError>;

    /// Listen for incoming connections
    pub async fn listen<F, Fut>(
        &self,
        handler: F,
    ) -> Result<(), NexusNetworkError>
    where
        F: Fn(CausalMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send;
}
```

**Example:**
```rust
use nexus_network::{QuicTransport, TlsConfig};
use std::time::Duration;

// Development
let transport = QuicTransport::new_dev("127.0.0.1:8080".parse()?, "nexus-node")?;

// Production
let tls = TlsConfig::from_files("cert.pem", "key.pem", Some("ca.pem"))?;
let transport = QuicTransport::new("127.0.0.1:8080".parse()?, tls)?;
```

### RateLimiter

```rust
pub struct RateLimiter {
    // ...
}

impl RateLimiter {
    /// Check if connection is allowed
    pub fn allow_connection(&self, peer: SocketAddr) -> bool;

    /// Check if message is allowed
    pub fn allow_message(&self, peer: SocketAddr) -> bool;

    /// Record connection disconnect
    pub fn record_disconnect(&self, peer: SocketAddr);

    /// Cleanup old entries
    pub fn cleanup(&self);
}
```

---

## Storage

### ProvenanceLog

```rust
pub struct ProvenanceLog {
    // ...
}

impl ProvenanceLog {
    /// Open provenance log
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NexusStorageError>;

    /// Append causal tensor
    pub fn append(&self, tensor: &CausalTensor) -> Result<(), NexusStorageError>;

    /// Append batch atomically
    pub fn append_batch(&self, tensors: &[CausalTensor]) -> Result<(), NexusStorageError>;

    /// Get tensor by ID
    pub fn get(&self, id: &CausalId) -> Result<Option<CausalTensor>, NexusStorageError>;
}
```

### BackupManager

```rust
pub struct BackupManager {
    // ...
}

impl BackupManager {
    /// Create full backup
    pub async fn create_full_backup(&self) -> Result<BackupMetadata, NexusStorageError>;

    /// Create incremental backup
    pub async fn create_incremental_backup(
        &self,
        base_backup_id: &str,
    ) -> Result<BackupMetadata, NexusStorageError>;

    /// Restore from backup
    pub async fn restore(
        &self,
        backup_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<(), NexusStorageError>;

    /// List all backups
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>, NexusStorageError>;

    /// Cleanup old backups
    pub async fn cleanup_old_backups(&self, keep_count: usize) -> Result<(), NexusStorageError>;
}
```

---

## Observability

### NexusMetrics

```rust
pub struct NexusMetrics {
    // PCU metrics
    pub pcu_executions_total: Counter,
    pub pcu_execution_duration: Histogram,
    pub pcu_cache_hits: Counter,
    pub pcu_cache_misses: Counter,
    pub pcu_execution_failures: Counter,
    pub pcu_active_executions: Gauge,

    // Network metrics
    pub network_messages_sent: Counter,
    pub network_messages_received: Counter,
    pub network_message_size: Histogram,
    pub network_send_duration: Histogram,
    pub network_connection_failures: Counter,
    pub network_rate_limit_rejections: Counter,

    // Storage metrics
    pub storage_reads_total: Counter,
    pub storage_writes_total: Counter,
    pub storage_read_duration: Histogram,
    pub storage_write_duration: Histogram,
    pub storage_errors_total: Counter,

    // Resource metrics
    pub resource_cpu_usage: Gauge,
    pub resource_memory_usage: Gauge,
}

impl NexusMetrics {
    /// Create new metrics instance
    pub fn new() -> Result<Self, anyhow::Error>;
}
```

**Usage:**
```rust
use nexus_observability::NexusMetrics;

let metrics = NexusMetrics::new()?;
metrics.pcu_executions_total.inc();
metrics.pcu_execution_duration.observe(0.5);
```

### Health Checks

```rust
pub struct HealthStatus {
    pub state: HealthState,
    pub components: Vec<ComponentHealth>,
}

pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

pub struct ComponentHealth {
    pub name: String,
    pub state: HealthState,
    pub message: Option<String>,
}

impl HealthStatus {
    /// Check if service is ready
    pub fn is_ready(&self) -> bool;

    /// Check if service is alive
    pub fn is_alive(&self) -> bool;
}
```

---

## Secrets Management

### SecretManager

```rust
pub struct SecretManager {
    // ...
}

impl SecretManager {
    /// Auto-detect backend from environment
    pub async fn auto() -> Result<Self, SecretError>;

    /// Create with local encrypted storage
    pub fn local(base_path: impl AsRef<Path>, encryption_key: [u8; 32]) -> Self;

    /// Create with HashiCorp Vault
    #[cfg(feature = "vault")]
    pub async fn vault(
        address: &str,
        token: &str,
        mount: Option<String>,
    ) -> Result<Self, SecretError>;

    /// Create with AWS Secrets Manager
    #[cfg(feature = "aws")]
    pub async fn aws(config: Option<aws_config::SdkConfig>) -> Result<Self, SecretError>;

    /// Get secret
    pub async fn get(&self, path: &str) -> Result<Secret, SecretError>;

    /// Store secret
    pub async fn put(&self, path: &str, secret: &Secret) -> Result<(), SecretError>;

    /// Delete secret
    pub async fn delete(&self, path: &str) -> Result<(), SecretError>;

    /// Check if secret exists
    pub async fn exists(&self, path: &str) -> Result<bool, SecretError>;
}
```

**Standard Secret Paths:**
```rust
pub mod paths {
    pub const TLS_PRIVATE_KEY: &str = "tls/private_key";
    pub const TLS_CERTIFICATE: &str = "tls/certificate";
    pub const NODE_SIGNING_KEY: &str = "node/signing_key";
    pub const DATABASE_URL: &str = "database/url";
    pub const API_KEY: &str = "api/key";
    pub const ENCRYPTION_KEY: &str = "encryption/key";
}
```

---

## Multi-tenancy

### TenantManager

```rust
pub struct TenantManager {
    // ...
}

impl TenantManager {
    /// Register new tenant
    pub fn register_tenant(
        &self,
        tenant_id: TenantId,
        name: String,
        principal_id: PrincipalId,
        quotas: Option<TenantQuotas>,
    ) -> Result<(), TenancyError>;

    /// Get tenant
    pub fn get_tenant(&self, tenant_id: TenantId) -> Option<Tenant>;

    /// Check quota before operation
    pub fn check_quota(
        &self,
        tenant_id: TenantId,
        operation: &QuotaOperation,
    ) -> Result<(), TenancyError>;

    /// Release resources after operation
    pub fn release_resources(
        &self,
        tenant_id: TenantId,
        operation: &QuotaOperation,
    );

    /// Get current usage
    pub fn get_usage(&self, tenant_id: TenantId) -> Option<TenantUsage>;

    /// Update quotas
    pub fn update_quotas(
        &self,
        tenant_id: TenantId,
        quotas: TenantQuotas,
    ) -> Result<(), TenancyError>;
}
```

**Quota Operations:**
```rust
pub enum QuotaOperation {
    ExecutePCU,
    AllocateMemory(u64),
    AllocateStorage(u64),
    NetworkTransfer(u64),
}
```

---

## Error Types

### ExecutionError

```rust
pub enum ExecutionError {
    ValidationFailed(String),
    CapabilityDenied(Capability),
    ResourceExceeded(String),
    WasmError(String),
    Timeout,
    Interrupted,
}
```

### NexusNetworkError

```rust
pub enum NexusNetworkError {
    ConnectionFailed(String),
    TlsError(String),
    RateLimitExceeded,
    MessageTooLarge,
    InvalidMessage,
}
```

### NexusStorageError

```rust
pub enum NexusStorageError {
    ConnectionFailed(String),
    WriteFailed(String),
    ReadFailed(String),
    SerializationFailed(String),
    NotFound(String),
}
```

---

## Best Practices

1. **Always validate PCUs** before execution:
   ```rust
   pcu.validate()?;
   pcu.check_capabilities()?;
   ```

2. **Use structured logging**:
   ```rust
   tracing::info!("Executing PCU", pcu_id = %pcu.id);
   ```

3. **Handle errors gracefully**:
   ```rust
   match executor.execute(&pcu).await {
       Ok(result) => { /* ... */ },
       Err(ExecutionError::ValidationFailed(e)) => { /* ... */ },
       Err(e) => { /* ... */ },
   }
   ```

4. **Monitor metrics**:
   ```rust
   metrics.pcu_execution_duration.observe(duration.as_secs_f64());
   ```

5. **Use secrets management** for sensitive data:
   ```rust
   let manager = SecretManager::auto().await?;
   let key = manager.get(nexus_secrets::paths::TLS_PRIVATE_KEY).await?;
   ```

---

*For detailed examples, see the [examples](../examples) directory.*


