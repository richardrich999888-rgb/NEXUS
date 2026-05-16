//! Multi-objective homeostasis controller.
//!
//! Balances multiple metrics simultaneously using constrained optimization.
//! Implements Pareto-optimal correction for conflicting objectives.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::bounds::HardBounds;
use crate::core::metric::{Metric, MetricId};

/// Multi-objective homeostasis controller.
///
/// Balances multiple metrics simultaneously using projected gradient descent
/// to find Pareto-optimal corrections that respect all bounds.
///
/// # Example
///
/// ```
/// use homeostasis_engine::core::bounds::HardBounds;
/// use homeostasis_engine::core::metric::{Metric, MetricId};
/// use homeostasis_engine::controller::multi_objective::MultiObjectiveController;
///
/// let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
/// let bounds = HardBounds::new(0.0, 1.0).unwrap();
///
/// controller.add_metric(Metric::new(MetricId(1), 0.0, 0.5, bounds, 0.5, 1.0).unwrap());
/// controller.add_metric(Metric::new(MetricId(2), 1.0, 0.3, bounds, 0.5, 1.0).unwrap());
///
/// let result = controller.converge();
/// assert!(result.converged);
/// ```
#[derive(Debug, Clone)]
pub struct MultiObjectiveController {
    metrics: HashMap<MetricId, Metric>,
    
    /// Learning rate for gradient descent.
    learning_rate: f64,
    
    /// Convergence threshold.
    tolerance: f64,
    
    /// Maximum iterations per solve.
    max_iterations: u32,
    
    /// Time step counter.
    step_count: u64,
}

impl MultiObjectiveController {
    /// Creates a new multi-objective controller.
    ///
    /// # Parameters
    ///
    /// - `learning_rate`: Step size for gradient descent (0 < lr <= 1)
    /// - `tolerance`: Convergence threshold for total error change
    /// - `max_iterations`: Maximum iterations per `converge()` call
    pub fn new(learning_rate: f64, tolerance: f64, max_iterations: u32) -> Self {
        assert!(learning_rate > 0.0 && learning_rate <= 1.0,
            "learning_rate must be in (0, 1]");
        assert!(tolerance > 0.0, "tolerance must be positive");
        
        Self {
            metrics: HashMap::new(),
            learning_rate,
            tolerance,
            max_iterations,
            step_count: 0,
        }
    }
    
    /// Adds a metric to the controller.
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.insert(metric.id, metric);
    }
    
    /// Gets a reference to a metric by ID.
    pub fn get_metric(&self, id: MetricId) -> Option<&Metric> {
        self.metrics.get(&id)
    }
    
    /// Gets a mutable reference to a metric by ID.
    pub fn get_metric_mut(&mut self, id: MetricId) -> Option<&mut Metric> {
        self.metrics.get_mut(&id)
    }
    
    /// Returns the number of metrics.
    pub fn metric_count(&self) -> usize {
        self.metrics.len()
    }
    
    /// Returns all metric IDs.
    pub fn metric_ids(&self) -> impl Iterator<Item = MetricId> + '_ {
        self.metrics.keys().copied()
    }
    
    /// Applies external changes to metrics (e.g., from cognition layer).
    ///
    /// Returns violations if any metric would exceed bounds.
    /// Changes are still applied (with clamping), but violations are reported.
    pub fn apply_external_changes(
        &mut self,
        changes: &[(MetricId, f64)],
    ) -> Vec<BoundsViolation> {
        let mut violations = Vec::new();
        
        for (id, delta) in changes {
            if let Some(metric) = self.metrics.get_mut(id) {
                let proposed = metric.value() + delta;
                
                if !metric.bounds.contains(proposed) {
                    violations.push(BoundsViolation {
                        metric_id: *id,
                        current_value: metric.value(),
                        proposed_value: proposed,
                        bounds: metric.bounds,
                        violation_magnitude: metric.bounds.violation(proposed),
                    });
                }
                
                // Apply with clamping
                metric.update(*delta);
            }
        }
        
        violations
    }
    
    /// Computes total weighted squared error.
    pub fn total_error(&self) -> f64 {
        self.metrics.values()
            .map(|m| m.weight * m.error() * m.error())
            .sum()
    }
    
    /// Computes one homeostatic correction step across all metrics.
    ///
    /// Uses projected gradient descent to find Pareto-optimal correction
    /// that respects all bounds.
    pub fn step(&mut self) -> MultiObjectiveResult {
        self.step_count += 1;
        
        let total_error_before = self.total_error();
        
        // Compute gradients and corrections. The gain is capped to avoid
        // high-priority metrics oscillating across the setpoint.
        let corrections: Vec<(MetricId, f64)> = self.metrics.values()
            .map(|m| {
                let gain = (2.0 * self.learning_rate * m.weight).clamp(0.0, 1.0);
                (m.id, -gain * m.error())
            })
            .collect();
        
        // Apply corrections with projection onto feasible region
        for (id, correction) in &corrections {
            if let Some(metric) = self.metrics.get_mut(id) {
                metric.update(*correction);
            }
        }
        
        // Compute error after correction
        let total_error_after = self.total_error();
        
        // Find metrics at bounds
        let bounds_active: Vec<MetricId> = self.metrics.values()
            .filter(|m| m.at_boundary())
            .map(|m| m.id)
            .collect();
        
        MultiObjectiveResult {
            step: self.step_count,
            total_error_before,
            total_error_after,
            error_reduction: total_error_before - total_error_after,
            bounds_active,
            converged: (total_error_before - total_error_after).abs() < self.tolerance,
        }
    }
    
    /// Runs until convergence or max iterations.
    pub fn converge(&mut self) -> ConvergenceResult {
        for i in 0..self.max_iterations {
            let result = self.step();
            
            if result.converged {
                return ConvergenceResult {
                    converged: true,
                    iterations: i + 1,
                    final_error: result.total_error_after,
                    bounds_active: result.bounds_active,
                };
            }
        }
        
        let bounds_active: Vec<MetricId> = self.metrics.values()
            .filter(|m| m.at_boundary())
            .map(|m| m.id)
            .collect();
        
        ConvergenceResult {
            converged: false,
            iterations: self.max_iterations,
            final_error: self.total_error(),
            bounds_active,
        }
    }
    
    /// Resets all metrics to their setpoints.
    pub fn reset_to_setpoints(&mut self) {
        for metric in self.metrics.values_mut() {
            metric.set_value(metric.setpoint);
        }
    }
    
    /// Returns system health assessment.
    pub fn health(&self) -> SystemHealth {
        let mut at_lower_bound = 0;
        let mut at_upper_bound = 0;
        let mut high_error = 0;
        let error_threshold = 0.1;
        
        for metric in self.metrics.values() {
            if metric.at_lower_bound() {
                at_lower_bound += 1;
            }
            if metric.at_upper_bound() {
                at_upper_bound += 1;
            }
            if metric.abs_error() > error_threshold {
                high_error += 1;
            }
        }
        
        SystemHealth {
            total_metrics: self.metrics.len(),
            at_lower_bound,
            at_upper_bound,
            within_tolerance: self.metrics.len() - high_error,
            high_error,
            total_error: self.total_error(),
            healthy: at_lower_bound == 0 && at_upper_bound == 0 && high_error == 0,
        }
    }
    
    /// Returns a snapshot of all metric values.
    pub fn snapshot(&self) -> HashMap<MetricId, f64> {
        self.metrics.iter()
            .map(|(id, m)| (*id, m.value()))
            .collect()
    }
}

/// Bounds violation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsViolation {
    pub metric_id: MetricId,
    pub current_value: f64,
    pub proposed_value: f64,
    pub bounds: HardBounds,
    pub violation_magnitude: f64,
}

/// Result of a single multi-objective step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiObjectiveResult {
    pub step: u64,
    pub total_error_before: f64,
    pub total_error_after: f64,
    pub error_reduction: f64,
    pub bounds_active: Vec<MetricId>,
    pub converged: bool,
}

/// Result of convergence attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceResult {
    pub converged: bool,
    pub iterations: u32,
    pub final_error: f64,
    pub bounds_active: Vec<MetricId>,
}

/// System health assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub total_metrics: usize,
    pub at_lower_bound: usize,
    pub at_upper_bound: usize,
    pub within_tolerance: usize,
    pub high_error: usize,
    pub total_error: f64,
    pub healthy: bool,
}

impl SystemHealth {
    /// Returns health as a score from 0 (critical) to 1 (perfect).
    pub fn score(&self) -> f64 {
        if self.total_metrics == 0 {
            return 1.0;
        }
        
        let bound_penalty = (self.at_lower_bound + self.at_upper_bound) as f64;
        let error_penalty = self.high_error as f64;
        let total_penalty = bound_penalty + error_penalty;
        
        (1.0 - total_penalty / (2.0 * self.total_metrics as f64)).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_bounds() -> HardBounds {
        HardBounds::new(0.0, 1.0).unwrap()
    }
    
    #[test]
    fn test_multi_metric_convergence() {
        let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
        
        controller.add_metric(Metric::new(
            MetricId(1), 0.0, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap());
        
        controller.add_metric(Metric::new(
            MetricId(2), 1.0, 0.3, test_bounds(), 0.5, 1.0
        ).unwrap());
        
        let result = controller.converge();
        
        assert!(result.converged);
        
        let m1 = controller.get_metric(MetricId(1)).unwrap();
        let m2 = controller.get_metric(MetricId(2)).unwrap();
        
        assert!((m1.value() - 0.5).abs() < 0.05);
        assert!((m2.value() - 0.3).abs() < 0.05);
    }
    
    #[test]
    fn test_external_change_violation() {
        let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
        
        controller.add_metric(Metric::new(
            MetricId(1), 0.5, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap());
        
        // Try to push beyond bounds
        let violations = controller.apply_external_changes(&[(MetricId(1), 2.0)]);
        
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].metric_id, MetricId(1));
        
        // Value should be clamped
        assert_eq!(controller.get_metric(MetricId(1)).unwrap().value(), 1.0);
    }
    
    #[test]
    fn test_health_assessment() {
        let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
        
        controller.add_metric(Metric::new(
            MetricId(1), 0.5, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap());
        
        let health = controller.health();
        
        assert!(health.healthy);
        assert_eq!(health.at_lower_bound, 0);
        assert_eq!(health.at_upper_bound, 0);
    }
}
