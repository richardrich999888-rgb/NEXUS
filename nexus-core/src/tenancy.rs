//! Multi-tenancy support for NEXUS
//! Tenant isolation, resource quotas, and access control

// PrincipalId is defined in nexus-pcu, but nexus-core doesn't depend on it
// Use a simple wrapper for now, or add nexus-pcu as dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrincipalId([u8; 32]);

impl PrincipalId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Tenant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(u64);

impl TenantId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Resource quotas per tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    pub max_pcu_executions_per_second: u64,
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_network_bandwidth_bytes_per_second: u64,
    pub max_concurrent_executions: u64,
}

impl Default for TenantQuotas {
    fn default() -> Self {
        Self {
            max_pcu_executions_per_second: 100,
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            max_network_bandwidth_bytes_per_second: 100 * 1024 * 1024, // 100MB/s
            max_concurrent_executions: 10,
        }
    }
}

/// Tenant metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub principal_id: PrincipalId,
    pub quotas: TenantQuotas,
    pub created_at: u64,
    pub active: bool,
}

/// Resource usage tracking per tenant
#[derive(Debug, Clone)]
pub struct TenantUsage {
    pub pcu_executions_this_second: u64,
    pub memory_bytes_used: u64,
    pub storage_bytes_used: u64,
    pub network_bytes_this_second: u64,
    pub concurrent_executions: u64,
    pub last_reset: std::time::Instant,
}

impl Default for TenantUsage {
    fn default() -> Self {
        Self {
            pcu_executions_this_second: 0,
            memory_bytes_used: 0,
            storage_bytes_used: 0,
            network_bytes_this_second: 0,
            concurrent_executions: 0,
            last_reset: std::time::Instant::now(),
        }
    }
}

impl TenantUsage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_if_needed(&mut self) {
        if self.last_reset.elapsed().as_secs() >= 1 {
            self.pcu_executions_this_second = 0;
            self.network_bytes_this_second = 0;
            self.last_reset = std::time::Instant::now();
        }
    }
}

/// Multi-tenant manager
pub struct TenantManager {
    tenants: Arc<RwLock<HashMap<TenantId, Tenant>>>,
    usage: Arc<RwLock<HashMap<TenantId, TenantUsage>>>,
}

impl TenantManager {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new tenant
    pub fn register_tenant(
        &self,
        tenant_id: TenantId,
        name: String,
        principal_id: PrincipalId,
        quotas: Option<TenantQuotas>,
    ) -> Result<(), TenancyError> {
        let mut tenants = self.tenants.write();
        
        if tenants.contains_key(&tenant_id) {
            return Err(TenancyError::TenantExists(tenant_id));
        }

        let tenant = Tenant {
            id: tenant_id,
            name,
            principal_id,
            quotas: quotas.unwrap_or_default(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            active: true,
        };

        tenants.insert(tenant_id, tenant);
        
        // Initialize usage tracking
        let mut usage = self.usage.write();
        usage.insert(tenant_id, TenantUsage::new());

        Ok(())
    }

    /// Get tenant by ID
    pub fn get_tenant(&self, tenant_id: TenantId) -> Option<Tenant> {
        self.tenants.read().get(&tenant_id).cloned()
    }

    /// Check if operation is allowed for tenant (quota check)
    pub fn check_quota(
        &self,
        tenant_id: TenantId,
        operation: &QuotaOperation,
    ) -> Result<(), TenancyError> {
        let tenants = self.tenants.read();
        let tenant = tenants.get(&tenant_id)
            .ok_or_else(|| TenancyError::TenantNotFound(tenant_id))?;

        if !tenant.active {
            return Err(TenancyError::TenantInactive(tenant_id));
        }

        let mut usage = self.usage.write();
        let usage_entry = usage.entry(tenant_id).or_insert_with(TenantUsage::new);
        usage_entry.reset_if_needed();

        match operation {
            QuotaOperation::ExecutePCU => {
                if usage_entry.pcu_executions_this_second >= tenant.quotas.max_pcu_executions_per_second {
                    return Err(TenancyError::QuotaExceeded("PCU executions per second".to_string()));
                }
                if usage_entry.concurrent_executions >= tenant.quotas.max_concurrent_executions {
                    return Err(TenancyError::QuotaExceeded("Concurrent executions".to_string()));
                }
                usage_entry.pcu_executions_this_second += 1;
                usage_entry.concurrent_executions += 1;
            }
            QuotaOperation::AllocateMemory(bytes) => {
                if usage_entry.memory_bytes_used + bytes > tenant.quotas.max_memory_bytes {
                    return Err(TenancyError::QuotaExceeded("Memory".to_string()));
                }
                usage_entry.memory_bytes_used += bytes;
            }
            QuotaOperation::AllocateStorage(bytes) => {
                if usage_entry.storage_bytes_used + bytes > tenant.quotas.max_storage_bytes {
                    return Err(TenancyError::QuotaExceeded("Storage".to_string()));
                }
                usage_entry.storage_bytes_used += bytes;
            }
            QuotaOperation::NetworkTransfer(bytes) => {
                if usage_entry.network_bytes_this_second + bytes > tenant.quotas.max_network_bandwidth_bytes_per_second {
                    return Err(TenancyError::QuotaExceeded("Network bandwidth".to_string()));
                }
                usage_entry.network_bytes_this_second += bytes;
            }
        }

        Ok(())
    }

    /// Release resources (e.g., when execution completes)
    pub fn release_resources(&self, tenant_id: TenantId, operation: &QuotaOperation) {
        let mut usage = self.usage.write();
        if let Some(usage_entry) = usage.get_mut(&tenant_id) {
            match operation {
                QuotaOperation::ExecutePCU => {
                    usage_entry.concurrent_executions = usage_entry.concurrent_executions.saturating_sub(1);
                }
                QuotaOperation::AllocateMemory(bytes) => {
                    usage_entry.memory_bytes_used = usage_entry.memory_bytes_used.saturating_sub(*bytes);
                }
                QuotaOperation::AllocateStorage(bytes) => {
                    usage_entry.storage_bytes_used = usage_entry.storage_bytes_used.saturating_sub(*bytes);
                }
                QuotaOperation::NetworkTransfer(_) => {
                    // Network usage resets per second, no need to release
                }
            }
        }
    }

    /// Get current usage for tenant
    pub fn get_usage(&self, tenant_id: TenantId) -> Option<TenantUsage> {
        self.usage.read().get(&tenant_id).cloned()
    }

    /// Update tenant quotas
    pub fn update_quotas(
        &self,
        tenant_id: TenantId,
        quotas: TenantQuotas,
    ) -> Result<(), TenancyError> {
        let mut tenants = self.tenants.write();
        let tenant = tenants.get_mut(&tenant_id)
            .ok_or_else(|| TenancyError::TenantNotFound(tenant_id))?;
        
        tenant.quotas = quotas;
        Ok(())
    }

    /// Deactivate tenant
    pub fn deactivate_tenant(&self, tenant_id: TenantId) -> Result<(), TenancyError> {
        let mut tenants = self.tenants.write();
        let tenant = tenants.get_mut(&tenant_id)
            .ok_or_else(|| TenancyError::TenantNotFound(tenant_id))?;
        
        tenant.active = false;
        Ok(())
    }
}

/// Quota operation types
#[derive(Debug, Clone)]
pub enum QuotaOperation {
    ExecutePCU,
    AllocateMemory(u64),
    AllocateStorage(u64),
    NetworkTransfer(u64),
}

/// Tenancy errors
#[derive(Debug, thiserror::Error)]
pub enum TenancyError {
    #[error("Tenant not found: {0:?}")]
    TenantNotFound(TenantId),
    
    #[error("Tenant already exists: {0:?}")]
    TenantExists(TenantId),
    
    #[error("Tenant is inactive: {0:?}")]
    TenantInactive(TenantId),
    
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

