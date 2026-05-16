//! # NEXUS Secret Management
//!
//! Production-grade secret management with support for:
//! - HashiCorp Vault
//! - AWS Secrets Manager
//! - Kubernetes Secrets
//! - Encrypted local storage (development)
//!
//! All secrets are encrypted at rest and never logged.

pub mod backend;
pub mod encryption;
pub mod manager;
pub mod error;

pub use backend::{SecretBackend, SecretBackendType};
pub use manager::SecretManager;
pub use error::{SecretError, SecretResult};

/// Secret value (zeroized on drop)
#[derive(Clone)]
pub struct Secret {
    inner: Vec<u8>,
}

impl Secret {
    pub fn new(data: Vec<u8>) -> Self {
        Self { inner: data }
    }

    pub fn from_str(s: &str) -> Self {
        Self::new(s.as_bytes().to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.inner).to_string()
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.inner.zeroize();
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}


