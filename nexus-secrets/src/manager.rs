//! Secret manager with automatic backend selection

use crate::backend::{SecretBackend, SecretBackendType, LocalBackend};
use crate::error::{SecretError, SecretResult};
use crate::Secret;
use std::sync::Arc;

#[cfg(feature = "vault")]
use crate::backend::vault_backend::VaultBackend;

#[cfg(feature = "aws")]
use crate::backend::aws_backend::AwsBackend;

#[cfg(feature = "k8s")]
use crate::backend::k8s_backend::KubernetesBackend;

/// Secret manager with automatic backend selection
pub struct SecretManager {
    backend: Arc<dyn SecretBackend>,
    backend_type: SecretBackendType,
}

impl SecretManager {
    /// Create manager with local encrypted file backend (development)
    pub fn local(base_path: impl AsRef<std::path::Path>, encryption_key: [u8; 32]) -> Self {
        let backend = Arc::new(LocalBackend::new(base_path, encryption_key));
        Self {
            backend,
            backend_type: SecretBackendType::Local,
        }
    }

    /// Create manager with HashiCorp Vault backend
    #[cfg(feature = "vault")]
    pub async fn vault(
        address: &str,
        token: &str,
        mount: Option<String>,
    ) -> SecretResult<Self> {
        let backend = Arc::new(VaultBackend::new(address, token, mount).await?);
        Ok(Self {
            backend,
            backend_type: SecretBackendType::Vault,
        })
    }

    /// Create manager with AWS Secrets Manager backend
    #[cfg(feature = "aws")]
    pub async fn aws(config: Option<aws_config::SdkConfig>) -> SecretResult<Self> {
        let backend = Arc::new(AwsBackend::new(config).await?);
        Ok(Self {
            backend,
            backend_type: SecretBackendType::Aws,
        })
    }

    /// Create manager with Kubernetes Secrets backend
    #[cfg(feature = "k8s")]
    pub async fn kubernetes(namespace: Option<String>) -> SecretResult<Self> {
        let backend = Arc::new(KubernetesBackend::new(namespace).await?);
        Ok(Self {
            backend,
            backend_type: SecretBackendType::Kubernetes,
        })
    }

    /// Auto-detect backend from environment
    pub async fn auto() -> SecretResult<Self> {
        // Check environment variables in priority order
        if let Ok(vault_addr) = std::env::var("VAULT_ADDR") {
            if let Ok(vault_token) = std::env::var("VAULT_TOKEN") {
                #[cfg(feature = "vault")]
                {
                    return Self::vault(&vault_addr, &vault_token, None).await;
                }
            }
        }

        // Check AWS credentials
        if std::env::var("AWS_REGION").is_ok() || std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
            #[cfg(feature = "aws")]
            {
                return Self::aws(None).await;
            }
        }

        // Check Kubernetes
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            #[cfg(feature = "k8s")]
            {
                let namespace = std::env::var("POD_NAMESPACE").ok();
                return Self::kubernetes(namespace).await;
            }
        }

        // Fallback to local (development)
        let base_path = std::env::var("NEXUS_SECRETS_PATH")
            .unwrap_or_else(|_| "/tmp/nexus-secrets".to_string());
        let key = crate::encryption::generate_key();
        
        Ok(Self::local(base_path, key))
    }

    /// Get a secret
    pub async fn get(&self, path: &str) -> SecretResult<Secret> {
        self.backend.get(path).await
    }

    /// Store a secret
    pub async fn put(&self, path: &str, secret: &Secret) -> SecretResult<()> {
        self.backend.put(path, secret).await
    }

    /// Delete a secret
    pub async fn delete(&self, path: &str) -> SecretResult<()> {
        self.backend.delete(path).await
    }

    /// Check if secret exists
    pub async fn exists(&self, path: &str) -> SecretResult<bool> {
        self.backend.exists(path).await
    }

    /// List secrets
    pub async fn list(&self, path: &str) -> SecretResult<Vec<String>> {
        self.backend.list(path).await
    }

    /// Get backend type
    pub fn backend_type(&self) -> SecretBackendType {
        self.backend_type
    }
}

/// Common secret paths for NEXUS
pub mod paths {
    /// TLS certificate private key
    pub const TLS_PRIVATE_KEY: &str = "tls/private_key";
    
    /// TLS certificate
    pub const TLS_CERTIFICATE: &str = "tls/certificate";
    
    /// Node signing key
    pub const NODE_SIGNING_KEY: &str = "node/signing_key";
    
    /// Database connection string
    pub const DATABASE_URL: &str = "database/url";
    
    /// API keys
    pub const API_KEY: &str = "api/key";
    
    /// Encryption keys
    pub const ENCRYPTION_KEY: &str = "encryption/key";
}


