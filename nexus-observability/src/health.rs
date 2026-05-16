//! Health check system for NEXUS

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

/// Health status of a component or system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy and ready
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is unhealthy and should not serve traffic
    Unhealthy,
}

impl HealthStatus {
    /// Check if system can serve traffic
    pub fn is_ready(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Check if system is alive (for liveness probes)
    pub fn is_alive(&self) -> bool {
        self != &HealthStatus::Unhealthy
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Optional message
    pub message: Option<String>,
    /// Last check timestamp
    pub checked_at: u64,
}

impl ComponentHealth {
    pub fn new(name: String, status: HealthStatus) -> Self {
        Self {
            name,
            status,
            message: None,
            checked_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

/// System-wide health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Overall system status
    pub status: HealthStatus,
    /// Individual component statuses
    pub components: Vec<ComponentHealth>,
    /// Timestamp of health check
    pub timestamp: u64,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Healthy,
            components: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Add component health
    pub fn add_component(&mut self, component: ComponentHealth) {
        // Update overall status based on worst component
        match component.status {
            HealthStatus::Unhealthy => {
                self.status = HealthStatus::Unhealthy;
                warn!(component = component.name, "Component is unhealthy");
            }
            HealthStatus::Degraded => {
                if self.status == HealthStatus::Healthy {
                    self.status = HealthStatus::Degraded;
                }
            }
            HealthStatus::Healthy => {
                // Only update if all components are healthy
                if self.components.iter().all(|c| c.status == HealthStatus::Healthy) {
                    self.status = HealthStatus::Healthy;
                }
            }
        }
        self.components.push(component);
    }

    /// Check if system is ready to serve traffic
    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }

    /// Check if system is alive
    pub fn is_alive(&self) -> bool {
        self.status.is_alive()
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}


