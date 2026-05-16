//! Secret backend implementations

use async_trait::async_trait;
use crate::error::{SecretError, SecretResult};
use crate::Secret;

/// Type of secret backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretBackendType {
    /// HashiCorp Vault
    Vault,
    /// AWS Secrets Manager
    Aws,
    /// Kubernetes Secrets
    Kubernetes,
    /// Encrypted local file (development only)
    Local,
}

/// Trait for secret backends
#[async_trait]
pub trait SecretBackend: Send + Sync {
    /// Get a secret by path/key
    async fn get(&self, path: &str) -> SecretResult<Secret>;

    /// Store a secret
    async fn put(&self, path: &str, secret: &Secret) -> SecretResult<()>;

    /// Delete a secret
    async fn delete(&self, path: &str) -> SecretResult<()>;

    /// Check if secret exists
    async fn exists(&self, path: &str) -> SecretResult<bool>;

    /// List secrets under a path
    async fn list(&self, path: &str) -> SecretResult<Vec<String>>;
}

/// Local encrypted file backend (development/testing)
pub struct LocalBackend {
    base_path: std::path::PathBuf,
    encryption_key: [u8; 32],
}

impl LocalBackend {
    pub fn new(base_path: impl AsRef<std::path::Path>, encryption_key: [u8; 32]) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            encryption_key,
        }
    }

    fn secret_path(&self, path: &str) -> std::path::PathBuf {
        // Sanitize path to prevent directory traversal
        let sanitized = path.replace("..", "").replace("/", "_");
        self.base_path.join(format!("{}.enc", sanitized))
    }
}

#[async_trait]
impl SecretBackend for LocalBackend {
    async fn get(&self, path: &str) -> SecretResult<Secret> {
        use tokio::fs;
        
        let file_path = self.secret_path(path);
        let encrypted = fs::read(&file_path)
            .await
            .map_err(|e| SecretError::NotFound(format!("Failed to read secret {}: {}", path, e)))?;

        let decrypted = crate::encryption::decrypt(&self.encryption_key, &encrypted)?;
        Ok(Secret::new(decrypted))
    }

    async fn put(&self, path: &str, secret: &Secret) -> SecretResult<()> {
        use tokio::fs;
        
        let encrypted = crate::encryption::encrypt(&self.encryption_key, secret.as_bytes())?;
        let file_path = self.secret_path(path);
        
        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| SecretError::Backend(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(&file_path, &encrypted)
            .await
            .map_err(|e| SecretError::Backend(format!("Failed to write secret: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, path: &str) -> SecretResult<()> {
        use tokio::fs;
        
        let file_path = self.secret_path(path);
        fs::remove_file(&file_path)
            .await
            .map_err(|e| SecretError::Backend(format!("Failed to delete secret: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, path: &str) -> SecretResult<bool> {
        use tokio::fs;
        
        let file_path = self.secret_path(path);
        Ok(file_path.exists())
    }

    async fn list(&self, path: &str) -> SecretResult<Vec<String>> {
        use tokio::fs;
        
        let dir: std::path::PathBuf = if path.is_empty() {
            self.base_path.clone()
        } else {
            self.base_path.join(path.replace("/", "_"))
        };

        let mut entries = Vec::new();
        if dir.exists() {
            let mut read_dir = fs::read_dir(&dir)
                .await
                .map_err(|e| SecretError::Backend(format!("Failed to list secrets: {}", e)))?;

            while let Some(entry) = read_dir.next_entry()
                .await
                .map_err(|e| SecretError::Backend(format!("Failed to read directory: {}", e)))?
            {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".enc") {
                        entries.push(name.trim_end_matches(".enc").to_string());
                    }
                }
            }
        }

        Ok(entries)
    }
}

#[cfg(feature = "vault")]
pub mod vault_backend {
    use super::*;
    use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
    use vaultrs::kv2;

    pub struct VaultBackend {
        client: VaultClient,
        mount: String,
    }

    impl VaultBackend {
        pub async fn new(
            address: &str,
            token: &str,
            mount: Option<String>,
        ) -> SecretResult<Self> {
            let settings = VaultClientSettingsBuilder::default()
                .address(address)
                .token(token)
                .build()
                .map_err(|e| SecretError::Backend(format!("Failed to create Vault client: {}", e)))?;

            let client = VaultClient::new(settings)
                .map_err(|e| SecretError::Backend(format!("Failed to connect to Vault: {}", e)))?;

            Ok(Self {
                client,
                mount: mount.unwrap_or_else(|| "secret".to_string()),
            })
        }
    }

    #[async_trait]
    impl SecretBackend for VaultBackend {
        async fn get(&self, path: &str) -> SecretResult<Secret> {
            let data = kv2::read(&self.client, &self.mount, path)
                .await
                .map_err(|e| SecretError::Backend(format!("Vault read error: {}", e)))?;

            // Extract the secret value (Vault returns JSON)
            let value = data
                .data
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| SecretError::InvalidFormat("Missing 'value' field in Vault response".to_string()))?;

            Ok(Secret::from_str(value))
        }

        async fn put(&self, path: &str, secret: &Secret) -> SecretResult<()> {
            let value = secret.to_string_lossy();
            let mut data = std::collections::HashMap::new();
            data.insert("value".to_string(), serde_json::Value::String(value));

            kv2::set(&self.client, &self.mount, path, &data)
                .await
                .map_err(|e| SecretError::Backend(format!("Vault write error: {}", e)))?;

            Ok(())
        }

        async fn delete(&self, path: &str) -> SecretResult<()> {
            kv2::delete(&self.client, &self.mount, path, None)
                .await
                .map_err(|e| SecretError::Backend(format!("Vault delete error: {}", e)))?;

            Ok(())
        }

        async fn exists(&self, path: &str) -> SecretResult<bool> {
            match self.get(path).await {
                Ok(_) => Ok(true),
                Err(SecretError::NotFound(_)) => Ok(false),
                Err(e) => Err(e),
            }
        }

        async fn list(&self, path: &str) -> SecretResult<Vec<String>> {
            let paths = kv2::list(&self.client, &self.mount, path)
                .await
                .map_err(|e| SecretError::Backend(format!("Vault list error: {}", e)))?;

            Ok(paths.keys)
        }
    }
}

#[cfg(feature = "aws")]
pub mod aws_backend {
    use super::*;
    use aws_sdk_secretsmanager::Client as SecretsManagerClient;
    use aws_config::SdkConfig;

    pub struct AwsBackend {
        client: SecretsManagerClient,
    }

    impl AwsBackend {
        pub async fn new(config: Option<SdkConfig>) -> SecretResult<Self> {
            let sdk_config = config.unwrap_or_else(|| {
                // Use default AWS config from environment
                tokio::runtime::Handle::current().block_on(aws_config::load_from_env())
            });

            let client = SecretsManagerClient::new(&sdk_config);

            Ok(Self { client })
        }
    }

    #[async_trait]
    impl SecretBackend for AwsBackend {
        async fn get(&self, path: &str) -> SecretResult<Secret> {
            let response = self.client
                .get_secret_value()
                .secret_id(path)
                .send()
                .await
                .map_err(|e| SecretError::Backend(format!("AWS Secrets Manager error: {}", e)))?;

            let secret_string = response.secret_string()
                .ok_or_else(|| SecretError::InvalidFormat("Missing secret string in AWS response".to_string()))?;

            Ok(Secret::from_str(secret_string))
        }

        async fn put(&self, path: &str, secret: &Secret) -> SecretResult<()> {
            let value = secret.to_string_lossy();
            
            self.client
                .create_secret()
                .name(path)
                .secret_string(&value)
                .send()
                .await
                .or_else(|e| {
                    // If secret exists, try updating
                    if e.to_string().contains("already exists") {
                        // Update existing secret
                        Box::pin(async move {
                            self.client
                                .update_secret()
                                .secret_id(path)
                                .secret_string(&value)
                                .send()
                                .await
                                .map_err(|e| SecretError::Backend(format!("AWS update error: {}", e)))
                        })
                    } else {
                        Box::pin(async move {
                            Err(SecretError::Backend(format!("AWS create error: {}", e)))
                        })
                    }
                })?;

            Ok(())
        }

        async fn delete(&self, path: &str) -> SecretResult<()> {
            self.client
                .delete_secret()
                .secret_id(path)
                .recovery_window_in_days(7) // Allow recovery for 7 days
                .send()
                .await
                .map_err(|e| SecretError::Backend(format!("AWS delete error: {}", e)))?;

            Ok(())
        }

        async fn exists(&self, path: &str) -> SecretResult<bool> {
            match self.get(path).await {
                Ok(_) => Ok(true),
                Err(SecretError::NotFound(_)) => Ok(false),
                Err(e) => Err(e),
            }
        }

        async fn list(&self, _path: &str) -> SecretResult<Vec<String>> {
            let mut names = Vec::new();
            let mut paginator = self.client
                .list_secrets()
                .into_paginator()
                .page_size(100)
                .send();

            while let Some(page) = paginator.next().await {
                let page = page.map_err(|e| SecretError::Backend(format!("AWS list error: {}", e)))?;
                if let Some(secrets) = page.secret_list() {
                    for secret in secrets {
                        if let Some(name) = secret.name() {
                            names.push(name.to_string());
                        }
                    }
                }
            }

            Ok(names)
        }
    }
}

#[cfg(feature = "k8s")]
pub mod k8s_backend {
    use super::*;
    use kube::{Api, Client};
    use k8s_openapi::api::core::v1::Secret as K8sSecret;

    pub struct KubernetesBackend {
        client: Client,
        namespace: String,
    }

    impl KubernetesBackend {
        pub async fn new(namespace: Option<String>) -> SecretResult<Self> {
            let client = Client::try_default()
                .await
                .map_err(|e| SecretError::Backend(format!("Failed to create K8s client: {}", e)))?;

            let namespace = namespace.unwrap_or_else(|| "default".to_string());

            Ok(Self { client, namespace })
        }
    }

    #[async_trait]
    impl SecretBackend for KubernetesBackend {
        async fn get(&self, path: &str) -> SecretResult<Secret> {
            let api: Api<K8sSecret> = Api::namespaced(self.client.clone(), &self.namespace);
            
            let secret = api.get(path)
                .await
                .map_err(|e| SecretError::NotFound(format!("K8s secret not found: {}", e)))?;

            // Extract first data entry (K8s secrets are key-value maps)
            let data = secret.data
                .and_then(|d| d.into_iter().next())
                .map(|(_, v)| v.0)
                .ok_or_else(|| SecretError::InvalidFormat("K8s secret has no data".to_string()))?;

            Ok(Secret::new(data))
        }

        async fn put(&self, path: &str, secret: &Secret) -> SecretResult<()> {
            use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
            use std::collections::BTreeMap;

            let api: Api<K8sSecret> = Api::namespaced(self.client.clone(), &self.namespace);

            let k8s_secret = K8sSecret {
                metadata: ObjectMeta {
                    name: Some(path.to_string()),
                    namespace: Some(self.namespace.clone()),
                    ..Default::default()
                },
                data: Some({
                    let mut map = BTreeMap::new();
                    map.insert("value".to_string(), k8s_openapi::ByteString(secret.as_bytes().to_vec()));
                    map
                }),
                ..Default::default()
            };

            // Try to create first
            match api.create(&Default::default(), &k8s_secret).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    // If exists, try update
                    if e.to_string().contains("already exists") {
                        api.replace(path, &Default::default(), &k8s_secret)
                            .await
                            .map_err(|e| SecretError::Backend(format!("K8s update error: {}", e)))
                    } else {
                        Err(SecretError::Backend(format!("K8s create error: {}", e)))
                    }
                }
            }?;

            Ok(())
        }

        async fn delete(&self, path: &str) -> SecretResult<()> {
            let api: Api<K8sSecret> = Api::namespaced(self.client.clone(), &self.namespace);
            
            api.delete(path, &Default::default())
                .await
                .map_err(|e| SecretError::Backend(format!("K8s delete error: {}", e)))?;

            Ok(())
        }

        async fn exists(&self, path: &str) -> SecretResult<bool> {
            match self.get(path).await {
                Ok(_) => Ok(true),
                Err(SecretError::NotFound(_)) => Ok(false),
                Err(e) => Err(e),
            }
        }

        async fn list(&self, _path: &str) -> SecretResult<Vec<String>> {
            let api: Api<K8sSecret> = Api::namespaced(self.client.clone(), &self.namespace);
            
            let secrets = api.list(&Default::default())
                .await
                .map_err(|e| SecretError::Backend(format!("K8s list error: {}", e)))?;

            Ok(secrets
                .iter()
                .filter_map(|s| s.metadata.name.clone())
                .collect())
        }
    }
}

