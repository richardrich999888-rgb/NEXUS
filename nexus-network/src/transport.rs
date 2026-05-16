// NEXUS Network: QUIC Transport with TLS 1.3
// Copyright (c) 2025 SYNTRIASS Labs Pvt Ltd
// Patent Pending: IN202501XXXXX

use crate::error::NexusNetworkError;
use crate::message::CausalMessage;
use crate::tls::TlsConfig;
use crate::rate_limit::RateLimiter;
use std::net::SocketAddr;
use std::sync::Arc;
use quinn::{Endpoint, ServerConfig, ClientConfig};
use futures::io::AsyncWriteExt;

pub struct QuicTransport {
    endpoint: Endpoint,
    tls_config: Arc<TlsConfig>,
    rate_limiter: Arc<RateLimiter>,
    metrics: Option<Arc<nexus_observability::NexusMetrics>>,
}

impl QuicTransport {
    /// Initialize a new QUIC endpoint with TLS configuration
    pub fn new(
        addr: SocketAddr,
        tls_config: TlsConfig,
    ) -> Result<Self, NexusNetworkError> {
        let rustls_config = tls_config.build_rustls_server_config()?;
        let server_config = ServerConfig::with_crypto(Arc::new(rustls_config));
        let endpoint = Endpoint::server(server_config, addr)
            .map_err(|e| NexusNetworkError::ConnectionFailed(e.to_string()))?;
            
        Ok(Self {
            endpoint,
            tls_config: Arc::new(tls_config),
            rate_limiter: Arc::new(RateLimiter::new()),
            metrics: None,
        })
    }

    /// Create transport with self-signed certificate (development only)
    pub fn new_dev(addr: SocketAddr, common_name: &str) -> Result<Self, NexusNetworkError> {
        let tls_config = TlsConfig::generate_self_signed(common_name, 90)?;
        Self::new(addr, tls_config)
    }

    /// Create transport with certificate files (production)
    pub fn new_with_certs(
        addr: SocketAddr,
        cert_path: impl AsRef<std::path::Path>,
        key_path: impl AsRef<std::path::Path>,
        client_ca_path: Option<impl AsRef<std::path::Path>>,
    ) -> Result<Self, NexusNetworkError> {
        let tls_config = TlsConfig::from_files(cert_path, key_path, client_ca_path)?;
        Self::new(addr, tls_config)
    }

    /// Get rate limiter reference
    pub fn rate_limiter(&self) -> &Arc<RateLimiter> {
        &self.rate_limiter
    }

    /// Set metrics for observability
    pub fn with_metrics(mut self, metrics: Arc<nexus_observability::NexusMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Connect to a remote node with proper TLS verification
    pub async fn connect(
        &self,
        addr: SocketAddr,
        root_cas: Option<Vec<rustls::Certificate>>,
    ) -> Result<quinn::Connection, NexusNetworkError> {
        // Check rate limit
        if !self.rate_limiter.allow_connection(addr) {
            if let Some(ref metrics) = self.metrics {
                metrics.network_rate_limit_rejections.inc();
            }
            return Err(NexusNetworkError::ConnectionFailed(
                "Rate limit: too many connections".to_string()
            ));
        }

        // Build client config with proper certificate verification
        let rustls_config = self.tls_config.build_rustls_client_config(root_cas)?;
        let client_config = ClientConfig::new(Arc::new(rustls_config));
            
        let connection = self.endpoint.connect_with(client_config, addr, "nexus-p2p")
            .map_err(|e| NexusNetworkError::ConnectionFailed(e.to_string()))?
            .await
            .map_err(|e| NexusNetworkError::ConnectionFailed(e.to_string()))?;
            
        Ok(connection)
    }

    /// Listen for incoming streams and process messages with rate limiting
    pub async fn listen<F, Fut>(&self, handler: F) -> Result<(), NexusNetworkError>
    where
        F: Fn(CausalMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let handler = Arc::new(handler);
        let rate_limiter = Arc::clone(&self.rate_limiter);
        let metrics = self.metrics.clone();
        
        while let Some(connecting) = self.endpoint.accept().await {
            let handler = handler.clone();
            let rate_limiter = rate_limiter.clone();
            let metrics = metrics.clone();
            
            tokio::spawn(async move {
                let connection = match connecting.await {
                    Ok(c) => c,
                    Err(_) => return,
                };

                // Get peer address for rate limiting
                let peer_addr = connection.remote_address();
                
                // Check rate limit for new connection
                if !rate_limiter.allow_connection(peer_addr) {
                    if let Some(ref m) = metrics {
                        m.network_rate_limit_rejections.inc();
                    }
                    tracing::warn!("Rate limit: rejecting connection from {}", peer_addr);
                    return;
                }

                // Clean up on disconnect
                let _guard = ConnectionGuard {
                    rate_limiter: rate_limiter.clone(),
                    peer_addr,
                };

                while let Ok(mut stream) = connection.accept_uni().await {
                    let h = handler.clone();
                    let rate_limiter = rate_limiter.clone();
                    let peer_addr = peer_addr;
                    let metrics = metrics.clone();
                    
                    tokio::spawn(async move {
                        if let Ok(buffer) = stream.read_to_end(1024 * 1024).await {
                            // Check message rate limit
                            if !rate_limiter.allow_message(peer_addr, buffer.len()) {
                                if let Some(ref m) = metrics {
                                    m.network_rate_limit_rejections.inc();
                                }
                                tracing::warn!("Rate limit: dropping message from {}", peer_addr);
                                return;
                            }

                            // Record received message
                            if let Some(ref m) = metrics {
                                m.network_messages_received.inc();
                                m.network_message_size.observe(buffer.len() as f64);
                            }

                            if let Ok(msg) = CausalMessage::from_bytes(&buffer) {
                                h(msg).await;
                            }
                        }
                    });
                }
            });
        }
        Ok(())
    }

    /// Send a message over a QUIC stream with rate limiting
    pub async fn send(&self, conn: &quinn::Connection, msg: &CausalMessage) -> Result<(), NexusNetworkError> {
        let start = std::time::Instant::now();
        let peer_addr = conn.remote_address();
        let bytes = msg.to_bytes().map_err(|e| NexusNetworkError::SerializationError(e.to_string()))?;
        
        // Check rate limit before sending
        if !self.rate_limiter.allow_message(peer_addr, bytes.len()) {
            if let Some(ref metrics) = self.metrics {
                metrics.network_rate_limit_rejections.inc();
            }
            return Err(NexusNetworkError::TransportError(
                "Rate limit: message rate exceeded".to_string()
            ));
        }

        let mut send_stream = conn.open_uni()
            .await
            .map_err(|e| {
                if let Some(ref metrics) = self.metrics {
                    metrics.network_connection_failures.inc();
                }
                NexusNetworkError::TransportError(e.to_string())
            })?;
        
        send_stream.write_all(&bytes)
            .await
            .map_err(|e| NexusNetworkError::TransportError(e.to_string()))?;
        
        send_stream.finish()
            .await
            .map_err(|e| NexusNetworkError::TransportError(e.to_string()))?;
        
        // Record metrics
        if let Some(ref metrics) = self.metrics {
            metrics.network_messages_sent.inc();
            metrics.network_message_size.observe(bytes.len() as f64);
            metrics.network_send_duration.observe(start.elapsed().as_secs_f64());
        }
        
        Ok(())
    }
}

/// RAII guard to clean up rate limiter state on connection close
struct ConnectionGuard {
    rate_limiter: Arc<RateLimiter>,
    peer_addr: SocketAddr,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.rate_limiter.record_disconnect(self.peer_addr);
    }
}
