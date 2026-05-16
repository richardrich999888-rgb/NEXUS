//! Health assessment and monitoring.

use serde::{Deserialize, Serialize};
use crate::controller::multi_objective::MultiObjectiveController;

/// Overall health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All metrics within tolerance, no bounds active.
    Healthy,
    /// Some metrics have elevated error but within bounds.
    Stressed,
    /// One or more metrics at bounds.
    Constrained,
    /// Multiple metrics at bounds or high error.
    Critical,
}

/// Detailed health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Overall status.
    pub status: HealthStatus,
    /// Health score (0-1).
    pub score: f64,
    /// Number of metrics at lower bound.
    pub at_lower_bound: usize,
    /// Number of metrics at upper bound.
    pub at_upper_bound: usize,
    /// Number of metrics with high error.
    pub high_error_count: usize,
    /// Total weighted error.
    pub total_error: f64,
    /// Recommendations.
    pub recommendations: Vec<String>,
}

impl HealthCheck {
    /// Performs health check on a multi-objective controller.
    pub fn check(controller: &MultiObjectiveController) -> Self {
        let sys_health = controller.health();
        
        let score = sys_health.score();
        
        let status = if sys_health.healthy {
            HealthStatus::Healthy
        } else if sys_health.at_lower_bound + sys_health.at_upper_bound > 0 {
            if sys_health.high_error > sys_health.total_metrics / 2 {
                HealthStatus::Critical
            } else {
                HealthStatus::Constrained
            }
        } else if sys_health.high_error > 0 {
            HealthStatus::Stressed
        } else {
            HealthStatus::Healthy
        };
        
        let mut recommendations = Vec::new();
        
        if sys_health.at_lower_bound > 0 {
            recommendations.push(format!(
                "{} metric(s) at lower bound - consider relaxing constraints",
                sys_health.at_lower_bound
            ));
        }
        
        if sys_health.at_upper_bound > 0 {
            recommendations.push(format!(
                "{} metric(s) at upper bound - consider relaxing constraints",
                sys_health.at_upper_bound
            ));
        }
        
        if sys_health.high_error > 0 {
            recommendations.push(format!(
                "{} metric(s) with high error - consider adjusting setpoints",
                sys_health.high_error
            ));
        }
        
        HealthCheck {
            status,
            score,
            at_lower_bound: sys_health.at_lower_bound,
            at_upper_bound: sys_health.at_upper_bound,
            high_error_count: sys_health.high_error,
            total_error: sys_health.total_error,
            recommendations,
        }
    }
    
    /// Returns true if system is in a healthy state.
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }
    
    /// Returns true if system needs attention.
    pub fn needs_attention(&self) -> bool {
        matches!(self.status, HealthStatus::Stressed | HealthStatus::Constrained | HealthStatus::Critical)
    }
    
    /// Returns true if system is in critical state.
    pub fn is_critical(&self) -> bool {
        self.status == HealthStatus::Critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bounds::HardBounds;
    use crate::core::metric::{Metric, MetricId};
    
    #[test]
    fn test_healthy_system() {
        let mut controller = MultiObjectiveController::new(0.1, 1e-6, 100);
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        
        controller.add_metric(
            Metric::new(MetricId(1), 0.5, 0.5, bounds, 0.5, 1.0).unwrap()
        );
        
        let check = HealthCheck::check(&controller);
        
        assert!(check.is_healthy());
        assert_eq!(check.status, HealthStatus::Healthy);
    }
}
