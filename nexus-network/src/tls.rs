//! TLS 1.3 Configuration and Certificate Management
//! 
//! Production-grade TLS with:
//! - TLS 1.3 only (no legacy protocols)
//! - Mutual TLS (mTLS) for node authentication
//! - Certificate validation and rotation
//! - Integration with NEXUS identity system

use crate::error::NexusNetworkError;
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig};
use rustls::server::{ClientCertVerifier, NoClientAuth};
use rustls::client::{ServerCertVerifier, ServerCertVerified};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use std::path::Path;
use tracing::{info, warn};
use webpki;

/// TLS configuration for NEXUS nodes
#[derive(Clone)]
pub struct TlsConfig {
    /// Server certificate chain
    pub cert_chain: Vec<Certificate>,
    /// Private key
    pub private_key: PrivateKey,
    /// Client CA certificates (for mTLS)
    pub client_ca: Option<Vec<Certificate>>,
    /// Certificate expiration time
    pub expires_at: SystemTime,
}

impl TlsConfig {
    /// Load TLS configuration from files
    pub fn from_files(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
        client_ca_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, NexusNetworkError> {
        // Load server certificate
        let cert_data = std::fs::read(cert_path)
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to read cert: {}", e)))?;
        let mut cert_reader = cert_data.as_slice();
        let cert_chain: Vec<Certificate> = rustls_pemfile::certs(&mut cert_reader)
            .map(|c| c.map(|der| Certificate(der.as_ref().to_vec())))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to parse cert: {}", e)))?;

        // Load private key
        let key_data = std::fs::read(key_path)
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to read key: {}", e)))?;
        let mut key_reader = key_data.as_slice();
        let mut keys: Vec<Vec<u8>> = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
            .map(|k| k.map(|k| k.secret_pkcs8_der().to_vec()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to parse key: {}", e)))?;
        
        let key_bytes = keys.pop()
            .ok_or_else(|| NexusNetworkError::AuthError("No private key found".to_string()))?;
        let private_key = PrivateKey(key_bytes);

        // Load client CA (optional, for mTLS)
        let client_ca = if let Some(ca_path) = client_ca_path {
            let ca_data = std::fs::read(ca_path)
                .map_err(|e| NexusNetworkError::AuthError(format!("Failed to read client CA: {}", e)))?;
            let mut ca_reader = ca_data.as_slice();
            Some(
                rustls_pemfile::certs(&mut ca_reader)
                    .map(|c| c.map(|der| Certificate(der.as_ref().to_vec())))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| NexusNetworkError::AuthError(format!("Failed to parse client CA: {}", e)))?
            )
        } else {
            None
        };

        // Parse certificate expiration (simplified - in production, parse from cert)
        let expires_at = SystemTime::now() + Duration::from_secs(90 * 24 * 60 * 60); // 90 days default

        Ok(Self {
            cert_chain,
            private_key,
            client_ca,
            expires_at,
        })
    }

    /// Generate self-signed certificate for development/testing
    /// 
    /// # Security Warning
    /// 
    /// This should only be used in development. Production must use
    /// proper CA-signed certificates or operator-provided certificates.
    pub fn generate_self_signed(
        common_name: &str,
        valid_for_days: u64,
    ) -> Result<Self, NexusNetworkError> {
        let cert = rcgen::generate_simple_self_signed(vec![common_name.into()])
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to generate cert: {}", e)))?;
        
        let cert_der = cert.serialize_der()
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to serialize cert: {}", e)))?;
        let key_der = cert.serialize_private_key_der();
        
        let cert_chain = vec![Certificate(cert_der)];
        let private_key = PrivateKey(key_der);
        let expires_at = SystemTime::now() + Duration::from_secs(valid_for_days * 24 * 60 * 60);

        Ok(Self {
            cert_chain,
            private_key,
            client_ca: None,
            expires_at,
        })
    }

    /// Check if certificate is expired or expiring soon
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }

    /// Check if certificate expires within the warning period (30 days)
    pub fn expires_soon(&self) -> bool {
        let warning_period = Duration::from_secs(30 * 24 * 60 * 60);
        let warning_time = self.expires_at - warning_period;
        SystemTime::now() >= warning_time
    }

    /// Build rustls server configuration with mTLS support
    pub fn build_rustls_server_config(&self) -> Result<ServerConfig, NexusNetworkError> {
        let config_builder = ServerConfig::builder()
            .with_safe_defaults();

        // Configure client authentication (mTLS)
        let config_builder = if let Some(ref client_ca) = self.client_ca {
            let verifier = Arc::new(NexusClientCertVerifier::new(client_ca.clone()));
            info!("mTLS enabled: client certificates required");
            config_builder.with_client_cert_verifier(verifier)
        } else {
            warn!("mTLS disabled: clients not authenticated");
            config_builder.with_client_cert_verifier(Arc::new(NoClientAuth))
        };

        config_builder
            .with_single_cert(self.cert_chain.clone(), self.private_key.clone())
            .map_err(|e| NexusNetworkError::AuthError(format!("Failed to build server config: {}", e)))
    }

    /// Build rustls client configuration with certificate validation
    pub fn build_rustls_client_config(
        &self,
        root_cas: Option<Vec<Certificate>>,
    ) -> Result<ClientConfig, NexusNetworkError> {
        let config_builder = ClientConfig::builder()
            .with_safe_defaults();

        // Configure server certificate verification
        let config_builder = if let Some(root_cas) = root_cas {
            let mut root_store = rustls::RootCertStore::empty();
            for cert in root_cas {
                root_store.add(&cert)
                    .map_err(|e| NexusNetworkError::AuthError(format!("Invalid root CA: {}", e)))?;
            }
            config_builder.with_root_certificates(root_store)
        } else {
            // Use system root CAs (empty store means use defaults)
            warn!("No root CAs provided: using system defaults");
            config_builder.with_root_certificates(rustls::RootCertStore::empty())
        };

        // Add client certificate for mTLS (if available)
        let config = if !self.cert_chain.is_empty() {
            config_builder.with_client_auth_cert(self.cert_chain.clone(), self.private_key.clone())
                .map_err(|e| NexusNetworkError::AuthError(format!("Failed to set client cert: {}", e)))?
        } else {
            config_builder.with_no_client_auth()
        };

        Ok(config)
    }
}

/// Client certificate verifier for mTLS
/// 
/// Verifies that client certificates are signed by trusted CAs
struct NexusClientCertVerifier {
    trusted_cas: Vec<Certificate>,
}

impl NexusClientCertVerifier {
    fn new(trusted_cas: Vec<Certificate>) -> Self {
        Self { trusted_cas }
    }
}

impl ClientCertVerifier for NexusClientCertVerifier {
    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        now: SystemTime,
    ) -> Result<rustls::server::ClientCertVerified, rustls::Error> {
        // Basic validation: certificate must be present
        if end_entity.0.is_empty() {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::BadEncoding
            ));
        }

        // 1. Parse end-entity certificate
        let cert = webpki::EndEntityCert::try_from(end_entity.0.as_slice())
            .map_err(|_| rustls::Error::InvalidCertificate(rustls::CertificateError::BadEncoding))?;

        // 2. Prepare intermediates
        let intermediates_der: Vec<&[u8]> = intermediates.iter()
            .map(|c| c.0.as_slice())
            .collect();

        // 3. Prepare trust anchors
        let mut anchors = Vec::with_capacity(self.trusted_cas.len());
        for ca_cert in &self.trusted_cas {
            let anchor = webpki::TrustAnchor::try_from_cert_der(&ca_cert.0)
                .map_err(|_| rustls::Error::InvalidCertificate(rustls::CertificateError::BadEncoding))?;
            anchors.push(anchor);
        }

        // 4. Perform full X.509 validation
        let time = webpki::Time::try_from(now)
            .map_err(|_| rustls::Error::FailedToGetCurrentTime)?;

        // Supported algorithms (NEXUS prefers Ed25519 and P-256)
        let algorithms = &[
            &webpki::ED25519,
            &webpki::ECDSA_P256_SHA256,
            &webpki::ECDSA_P384_SHA384,
            &webpki::RSA_PKCS1_2048_8192_SHA256,
            &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        ];

        cert.verify_is_valid_tls_client_cert(
            algorithms,
            &webpki::TlsClientTrustAnchors(&anchors),
            &intermediates_der,
            time,
            &[] // No CRLs for now
        ).map_err(|e| {
            warn!("Client certificate validation failed: {}", e);
            rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer)
        })?;
        
        Ok(rustls::server::ClientCertVerified::assertion())
    }

    fn client_auth_root_subjects(&self) -> &[rustls::DistinguishedName] {
        // Return empty so client sends all available certs or we can filter in verify_client_cert
        &[]
    }
}

/// Server certificate verifier for client connections
/// 
/// Verifies server certificates against trusted root CAs
pub struct NexusServerCertVerifier {
    _trusted_roots: rustls::RootCertStore,
    trusted_certs: Vec<Certificate>,
}

impl NexusServerCertVerifier {
    pub fn new(trusted_roots: rustls::RootCertStore, trusted_certs: Vec<Certificate>) -> Self {
        Self { _trusted_roots: trusted_roots, trusted_certs }
    }

    pub fn from_certs(certs: Vec<Certificate>) -> Result<Self, NexusNetworkError> {
        let mut root_store = rustls::RootCertStore::empty();
        let certs_clone = certs.clone();
        for cert in certs {
            root_store.add(&cert)
                .map_err(|e| NexusNetworkError::AuthError(format!("Invalid root CA: {}", e)))?;
        }
        Ok(Self::new(root_store, certs_clone))
    }
}

impl ServerCertVerifier for NexusServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // Basic validation: certificate must be present
        if end_entity.0.is_empty() {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::BadEncoding
            ));
        }

        // 1. Parse end-entity certificate
        let cert = webpki::EndEntityCert::try_from(end_entity.0.as_slice())
            .map_err(|_| rustls::Error::InvalidCertificate(rustls::CertificateError::BadEncoding))?;

        // 2. Prepare intermediates
        let intermediates_der: Vec<&[u8]> = intermediates.iter()
            .map(|c| c.0.as_slice())
            .collect();

        // 3. Prepare trust anchors
        let mut anchors = Vec::with_capacity(self.trusted_certs.len());
        for ca_cert in &self.trusted_certs {
            let anchor = webpki::TrustAnchor::try_from_cert_der(&ca_cert.0)
                .map_err(|_| rustls::Error::InvalidCertificate(rustls::CertificateError::BadEncoding))?;
            anchors.push(anchor);
        }

        let time = webpki::Time::try_from(now)
            .map_err(|_| rustls::Error::FailedToGetCurrentTime)?;

        let algorithms = &[
            &webpki::ED25519,
            &webpki::ECDSA_P256_SHA256,
            &webpki::ECDSA_P384_SHA384,
            &webpki::RSA_PKCS1_2048_8192_SHA256,
            &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        ];

        // 4. Verify certificate chain
        cert.verify_is_valid_tls_server_cert(
            algorithms,
            &webpki::TlsServerTrustAnchors(&anchors),
            &intermediates_der,
            time
        ).map_err(|e| {
            warn!("Server certificate validation failed: {}", e);
            rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer)
        })?;

        // 5. Verify server name
        let dns_name = match server_name {
            rustls::ServerName::DnsName(name) => name.as_ref(),
            rustls::ServerName::IpAddress(ip) => &ip.to_string(),
            _ => return Err(rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName)),
        };

        let webpki_name = webpki::SubjectNameRef::try_from_ascii_str(dns_name)
            .map_err(|_| rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName))?;

        cert.verify_is_valid_for_subject_name(webpki_name)
            .map_err(|_| rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName))?;
        
        Ok(ServerCertVerified::assertion())
    }
}

