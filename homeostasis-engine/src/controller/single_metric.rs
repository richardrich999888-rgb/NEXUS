//! Single-metric proportional controller with bounds enforcement.

use serde::{Deserialize, Serialize};
use crate::core::metric::Metric;

/// Single-metric proportional controller with bounds enforcement.
///
/// Applies negative feedback to drive a metric toward its setpoint
/// while respecting hard bounds.
///
/// # Example
///
/// ```
/// use homeostasis_engine::core::bounds::HardBounds;
/// use homeostasis_engine::core::metric::{Metric, MetricId};
/// use homeostasis_engine::controller::single_metric::SingleMetricController;
///
/// let bounds = HardBounds::new(0.0, 1.0).unwrap();
/// let mut metric = Metric::new(MetricId(1), 0.2, 0.5, bounds, 0.5, 1.0).unwrap();
/// let controller = SingleMetricController::new(0.1);
///
/// // Run 50 steps
/// for _ in 0..50 {
///     controller.step(&mut metric);
/// }
///
/// // Should be near setpoint
/// assert!((metric.value() - 0.5).abs() < 0.05);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleMetricController {
    /// Maximum correction per step (prevents oscillation).
    pub max_correction: f64,
    
    /// Damping factor to reduce oscillations (0-1).
    pub damping: f64,
}

impl SingleMetricController {
    /// Creates a new controller with the given maximum correction.
    ///
    /// # Panics
    ///
    /// Panics if `max_correction <= 0`.
    pub fn new(max_correction: f64) -> Self {
        assert!(max_correction > 0.0, "max_correction must be positive");
        Self {
            max_correction,
            damping: 0.0,
        }
    }
    
    /// Creates a controller with damping.
    pub fn with_damping(max_correction: f64, damping: f64) -> Self {
        assert!(max_correction > 0.0, "max_correction must be positive");
        assert!((0.0..=1.0).contains(&damping), "damping must be in [0, 1]");
        Self {
            max_correction,
            damping,
        }
    }
    
    /// Computes and applies one correction step.
    ///
    /// Returns detailed information about the correction applied.
    pub fn step(&self, metric: &mut Metric) -> CorrectionResult {
        let raw_correction = metric.correction_signal();
        
        // Apply damping
        let damped = raw_correction * (1.0 - self.damping);
        
        // Clamp correction magnitude to prevent oscillation
        let clamped = damped.clamp(-self.max_correction, self.max_correction);
        
        let applied = metric.update(clamped);
        
        CorrectionResult {
            requested: raw_correction,
            damped,
            clamped,
            applied,
            new_value: metric.value(),
            new_error: metric.error(),
            bounds_active: (applied - clamped).abs() > 1e-9,
        }
    }
    
    /// Runs until convergence or max steps.
    ///
    /// Returns the number of steps taken and whether convergence was achieved.
    pub fn converge(
        &self,
        metric: &mut Metric,
        tolerance: f64,
        max_steps: u32,
    ) -> (u32, bool) {
        for step in 0..max_steps {
            let result = self.step(metric);
            
            if result.new_error.abs() < tolerance {
                return (step + 1, true);
            }
        }
        
        (max_steps, false)
    }
}

impl Default for SingleMetricController {
    fn default() -> Self {
        Self::new(0.1)
    }
}

/// Result of a correction step.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CorrectionResult {
    /// Raw correction signal before damping/clamping.
    pub requested: f64,
    
    /// Correction after damping.
    pub damped: f64,
    
    /// Correction after magnitude clamping.
    pub clamped: f64,
    
    /// Actual change applied (may differ if hitting bounds).
    pub applied: f64,
    
    /// Value after correction.
    pub new_value: f64,
    
    /// Error after correction.
    pub new_error: f64,
    
    /// True if hard bounds prevented full correction.
    pub bounds_active: bool,
}

impl CorrectionResult {
    /// Returns true if the requested correction was fully applied.
    pub fn fully_applied(&self) -> bool {
        (self.requested - self.applied).abs() < 1e-9
    }
    
    /// Returns the fraction of requested correction that was applied.
    pub fn application_ratio(&self) -> f64 {
        if self.requested.abs() < 1e-9 {
            1.0
        } else {
            self.applied / self.requested
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bounds::HardBounds;
    use crate::core::metric::MetricId;
    
    fn test_metric(initial: f64, setpoint: f64) -> Metric {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        Metric::new(MetricId(1), initial, setpoint, bounds, 0.5, 1.0).unwrap()
    }
    
    #[test]
    fn test_convergence_to_setpoint() {
        let mut metric = test_metric(0.0, 0.5);
        let controller = SingleMetricController::new(0.1);
        
        let (steps, converged) = controller.converge(&mut metric, 0.01, 100);
        
        assert!(converged);
        assert!(steps < 100);
        assert!((metric.value() - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_respects_upper_bound() {
        let mut metric = test_metric(0.9, 0.5);
        metric.set_value(0.95);
        
        let controller = SingleMetricController::new(0.5);
        
        // Force upward (wrong direction for stability test)
        metric.update(0.1);
        
        assert_eq!(metric.value(), 1.0); // Should hit bound
    }
    
    #[test]
    fn test_damping_reduces_oscillation() {
        let mut metric1 = test_metric(0.0, 0.5);
        let mut metric2 = test_metric(0.0, 0.5);
        
        let controller_no_damp = SingleMetricController::new(0.5);
        let controller_damped = SingleMetricController::with_damping(0.5, 0.3);
        
        // First step correction
        let result1 = controller_no_damp.step(&mut metric1);
        let result2 = controller_damped.step(&mut metric2);
        
        // Damped should have smaller correction
        assert!(result2.applied.abs() < result1.applied.abs());
    }
}
