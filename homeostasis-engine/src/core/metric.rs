//! Metric definition with setpoint and bounds.
//!
//! A metric is the fundamental unit of homeostatic control - an observable
//! value that the system maintains within bounds and around a setpoint.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::core::bounds::HardBounds;

/// Error type for metric operations.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MetricError {
    #[error("Invalid gain: {0} (must be positive)")]
    InvalidGain(f64),
    
    #[error("Invalid weight: {0} (must be non-negative)")]
    InvalidWeight(f64),
    
    #[error("Setpoint {setpoint} is outside bounds [{}, {}]", .bounds.lower, .bounds.upper)]
    SetpointOutOfBounds { setpoint: f64, bounds: HardBounds },
    
    #[error("Initial value {value} is outside bounds [{}, {}]", .bounds.lower, .bounds.upper)]
    InitialValueOutOfBounds { value: f64, bounds: HardBounds },
}

/// Unique identifier for a metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricId(pub u32);

impl MetricId {
    /// Creates a new metric ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<u32> for MetricId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// A single homeostatic metric with setpoint and bounds.
///
/// # Fields
///
/// - `id`: Unique identifier
/// - `value`: Current value (always within bounds)
/// - `setpoint`: Target value the system maintains
/// - `bounds`: Hard limits that cannot be violated
/// - `gain`: Proportional gain for correction (higher = faster, more oscillation)
/// - `weight`: Priority weight in multi-objective optimization
///
/// # Example
///
/// ```
/// use homeostasis_engine::core::bounds::HardBounds;
/// use homeostasis_engine::core::metric::{Metric, MetricId};
///
/// let bounds = HardBounds::new(0.0, 1.0).unwrap();
/// let mut metric = Metric::new(
///     MetricId(1),
///     0.2,    // initial value
///     0.5,    // setpoint
///     bounds,
///     0.5,    // gain
///     1.0,    // weight
/// ).unwrap();
///
/// assert_eq!(metric.error(), -0.3); // below setpoint
/// assert!(metric.correction_signal() > 0.0); // should increase
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Unique identifier for this metric.
    pub id: MetricId,
    
    /// Current value (private to enforce bounds).
    value: f64,
    
    /// Target value the system tries to maintain.
    pub setpoint: f64,
    
    /// Hard limits that must not be violated.
    pub bounds: HardBounds,
    
    /// Proportional gain for correction signal.
    /// Higher = faster response, more oscillation risk.
    pub gain: f64,
    
    /// Weight in multi-objective optimization.
    /// Higher = this metric is prioritized.
    pub weight: f64,
    
    /// Human-readable name (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Metric {
    /// Creates a new metric with validation.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - `gain <= 0.0`
    /// - `weight < 0.0`
    /// - `setpoint` is outside bounds
    pub fn new(
        id: MetricId,
        initial_value: f64,
        setpoint: f64,
        bounds: HardBounds,
        gain: f64,
        weight: f64,
    ) -> Result<Self, MetricError> {
        // Validate gain
        if gain <= 0.0 || !gain.is_finite() {
            return Err(MetricError::InvalidGain(gain));
        }
        
        // Validate weight
        if weight < 0.0 || !weight.is_finite() {
            return Err(MetricError::InvalidWeight(weight));
        }
        
        // Setpoint must be within bounds
        if !bounds.contains(setpoint) {
            return Err(MetricError::SetpointOutOfBounds { setpoint, bounds });
        }
        
        // Clamp initial value to bounds (warning: silent correction)
        let clamped_value = bounds.clamp(initial_value);
        
        Ok(Self {
            id,
            value: clamped_value,
            setpoint,
            bounds,
            gain,
            weight,
            name: None,
        })
    }
    
    /// Creates a metric with a name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Returns current value.
    #[inline]
    pub fn value(&self) -> f64 {
        self.value
    }
    
    /// Updates value, enforcing hard bounds.
    /// Returns the actual change applied (may differ from delta if bounds hit).
    pub fn update(&mut self, delta: f64) -> f64 {
        let old = self.value;
        let new_unclamped = self.value + delta;
        self.value = self.bounds.clamp(new_unclamped);
        self.value - old
    }
    
    /// Sets value directly, enforcing hard bounds.
    pub fn set_value(&mut self, value: f64) {
        self.value = self.bounds.clamp(value);
    }
    
    /// Computes error from setpoint.
    /// Positive if above setpoint, negative if below.
    #[inline]
    pub fn error(&self) -> f64 {
        self.value - self.setpoint
    }
    
    /// Computes absolute error from setpoint.
    #[inline]
    pub fn abs_error(&self) -> f64 {
        (self.value - self.setpoint).abs()
    }
    
    /// Computes normalized error (error / span).
    #[inline]
    pub fn normalized_error(&self) -> f64 {
        self.error() / self.bounds.span()
    }
    
    /// Computes correction signal (negative feedback).
    /// Returns the delta that should be applied to move toward setpoint.
    #[inline]
    pub fn correction_signal(&self) -> f64 {
        -self.gain * self.error()
    }
    
    /// Returns bounds violation magnitude (0 if within bounds).
    #[inline]
    pub fn violation(&self) -> f64 {
        self.bounds.violation(self.value)
    }
    
    /// Returns true if value equals lower or upper bound.
    #[inline]
    pub fn at_boundary(&self) -> bool {
        self.bounds.boundary_edge(self.value).is_some()
    }
    
    /// Returns true if value is at lower bound.
    #[inline]
    pub fn at_lower_bound(&self) -> bool {
        (self.value - self.bounds.lower).abs() < 1e-9
    }
    
    /// Returns true if value is at upper bound.
    #[inline]
    pub fn at_upper_bound(&self) -> bool {
        (self.value - self.bounds.upper).abs() < 1e-9
    }
    
    /// Returns the distance from current value to setpoint as fraction of span.
    #[inline]
    pub fn relative_distance_to_setpoint(&self) -> f64 {
        self.abs_error() / self.bounds.span()
    }
    
    /// Updates setpoint with validation.
    pub fn set_setpoint(&mut self, new_setpoint: f64) -> Result<(), MetricError> {
        if !self.bounds.contains(new_setpoint) {
            return Err(MetricError::SetpointOutOfBounds { 
                setpoint: new_setpoint, 
                bounds: self.bounds 
            });
        }
        self.setpoint = new_setpoint;
        Ok(())
    }
    
    /// Returns summary of metric state.
    pub fn summary(&self) -> MetricSummary {
        MetricSummary {
            id: self.id,
            value: self.value,
            setpoint: self.setpoint,
            error: self.error(),
            at_lower_bound: self.at_lower_bound(),
            at_upper_bound: self.at_upper_bound(),
        }
    }
}

/// Summary of metric state.
#[derive(Debug, Clone, Copy)]
pub struct MetricSummary {
    pub id: MetricId,
    pub value: f64,
    pub setpoint: f64,
    pub error: f64,
    pub at_lower_bound: bool,
    pub at_upper_bound: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_bounds() -> HardBounds {
        HardBounds::new(0.0, 1.0).unwrap()
    }
    
    #[test]
    fn test_creation() {
        let metric = Metric::new(
            MetricId(1),
            0.5,
            0.5,
            test_bounds(),
            0.5,
            1.0,
        ).unwrap();
        
        assert_eq!(metric.id.0, 1);
        assert_eq!(metric.value(), 0.5);
        assert_eq!(metric.setpoint, 0.5);
    }
    
    #[test]
    fn test_invalid_gain() {
        let result = Metric::new(
            MetricId(1), 0.5, 0.5, test_bounds(), -0.5, 1.0
        );
        assert!(matches!(result, Err(MetricError::InvalidGain(_))));
    }
    
    #[test]
    fn test_setpoint_out_of_bounds() {
        let result = Metric::new(
            MetricId(1), 0.5, 2.0, test_bounds(), 0.5, 1.0
        );
        assert!(matches!(result, Err(MetricError::SetpointOutOfBounds { .. })));
    }
    
    #[test]
    fn test_initial_value_clamped() {
        let metric = Metric::new(
            MetricId(1), 2.0, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap();
        
        // Value should be clamped to upper bound
        assert_eq!(metric.value(), 1.0);
    }
    
    #[test]
    fn test_error_calculation() {
        let metric = Metric::new(
            MetricId(1), 0.3, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap();
        
        assert_eq!(metric.error(), -0.2); // below setpoint
    }
    
    #[test]
    fn test_correction_signal() {
        let metric = Metric::new(
            MetricId(1), 0.3, 0.5, test_bounds(), 1.0, 1.0
        ).unwrap();
        
        // Error is -0.2, gain is 1.0
        // Correction should be -(-0.2) * 1.0 = +0.2
        assert_eq!(metric.correction_signal(), 0.2);
    }
    
    #[test]
    fn test_update_within_bounds() {
        let mut metric = Metric::new(
            MetricId(1), 0.5, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap();
        
        let applied = metric.update(0.2);
        assert!((applied - 0.2).abs() < 1e-9);
        assert!((metric.value() - 0.7).abs() < 1e-9);
    }
    
    #[test]
    fn test_update_hits_bound() {
        let mut metric = Metric::new(
            MetricId(1), 0.9, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap();
        
        let applied = metric.update(0.5);
        assert!((applied - 0.1).abs() < 1e-9); // Only 0.1 applied due to upper bound
        assert!((metric.value() - 1.0).abs() < 1e-9);
    }
    
    #[test]
    fn test_at_boundary() {
        let mut metric = Metric::new(
            MetricId(1), 0.0, 0.5, test_bounds(), 0.5, 1.0
        ).unwrap();
        
        assert!(metric.at_lower_bound());
        assert!(!metric.at_upper_bound());
        
        metric.set_value(1.0);
        assert!(!metric.at_lower_bound());
        assert!(metric.at_upper_bound());
    }
}
