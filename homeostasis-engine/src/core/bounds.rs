//! Hard bounds that cannot be violated.
//! 
//! If a metric reaches a hard bound, the system must correct.
//! These represent absolute constraints that maintain system safety.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for bounds operations.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BoundsError {
    #[error("Invalid range: lower ({lower}) must be less than upper ({upper})")]
    InvalidRange { lower: f64, upper: f64 },
    
    #[error("Bounds must be finite values")]
    NonFinite,
    
    #[error("Range too narrow: span ({span}) below minimum ({min})")]
    RangeTooNarrow { span: f64, min: f64 },
}

/// Hard bounds that cannot be violated.
/// 
/// These represent absolute constraints on metric values.
/// The system must always maintain values within these bounds.
/// 
/// # Example
/// 
/// ```
/// use homeostasis_engine::core::bounds::HardBounds;
/// 
/// let bounds = HardBounds::new(0.0, 1.0).unwrap();
/// assert!(bounds.contains(0.5));
/// assert!(!bounds.contains(1.5));
/// assert_eq!(bounds.clamp(1.5), 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HardBounds {
    /// Lower bound (inclusive)
    pub lower: f64,
    /// Upper bound (inclusive)
    pub upper: f64,
}

impl HardBounds {
    /// Minimum allowed range between lower and upper bounds.
    pub const MIN_RANGE: f64 = 1e-10;
    
    /// Creates new bounds with validation.
    /// 
    /// # Errors
    /// 
    /// Returns error if:
    /// - `lower >= upper`
    /// - Either bound is non-finite (NaN or infinity)
    /// - Range is below minimum threshold
    pub fn new(lower: f64, upper: f64) -> Result<Self, BoundsError> {
        // Check for finite values
        if !lower.is_finite() || !upper.is_finite() {
            return Err(BoundsError::NonFinite);
        }
        
        // Check ordering
        if lower >= upper {
            return Err(BoundsError::InvalidRange { lower, upper });
        }
        
        // Check minimum range
        let span = upper - lower;
        if span < Self::MIN_RANGE {
            return Err(BoundsError::RangeTooNarrow { 
                span, 
                min: Self::MIN_RANGE 
            });
        }
        
        Ok(Self { lower, upper })
    }
    
    /// Creates bounds for a unit interval [0, 1].
    pub fn unit() -> Self {
        Self { lower: 0.0, upper: 1.0 }
    }
    
    /// Creates symmetric bounds [-limit, +limit].
    pub fn symmetric(limit: f64) -> Result<Self, BoundsError> {
        Self::new(-limit.abs(), limit.abs())
    }
    
    /// Clamps value to bounds.
    #[inline]
    pub fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.lower, self.upper)
    }
    
    /// Returns violation magnitude. Zero if within bounds.
    /// 
    /// For values above upper bound, returns `value - upper`.
    /// For values below lower bound, returns `lower - value`.
    #[inline]
    pub fn violation(&self, value: f64) -> f64 {
        if value > self.upper {
            value - self.upper
        } else if value < self.lower {
            self.lower - value
        } else {
            0.0
        }
    }
    
    /// Returns signed violation (positive if above, negative if below).
    #[inline]
    pub fn signed_violation(&self, value: f64) -> f64 {
        if value > self.upper {
            value - self.upper
        } else if value < self.lower {
            value - self.lower
        } else {
            0.0
        }
    }
    
    /// Returns true if value is within bounds (inclusive).
    #[inline]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }
    
    /// Returns the span (range) of the bounds.
    #[inline]
    pub fn span(&self) -> f64 {
        self.upper - self.lower
    }
    
    /// Returns the midpoint of the bounds.
    #[inline]
    pub fn midpoint(&self) -> f64 {
        (self.lower + self.upper) / 2.0
    }
    
    /// Normalizes a value to [0, 1] within these bounds.
    #[inline]
    pub fn normalize(&self, value: f64) -> f64 {
        (value - self.lower) / self.span()
    }
    
    /// Denormalizes a [0, 1] value to this range.
    #[inline]
    pub fn denormalize(&self, normalized: f64) -> f64 {
        self.lower + normalized * self.span()
    }
    
    /// Returns which edge the value is closer to, if at boundary.
    pub fn boundary_edge(&self, value: f64) -> Option<BoundaryEdge> {
        const EPSILON: f64 = 1e-9;
        
        if (value - self.lower).abs() < EPSILON {
            Some(BoundaryEdge::Lower)
        } else if (value - self.upper).abs() < EPSILON {
            Some(BoundaryEdge::Upper)
        } else {
            None
        }
    }
}

/// Which boundary edge a value is at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryEdge {
    Lower,
    Upper,
}

impl Default for HardBounds {
    fn default() -> Self {
        Self::unit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_bounds() {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        assert_eq!(bounds.lower, 0.0);
        assert_eq!(bounds.upper, 1.0);
    }
    
    #[test]
    fn test_invalid_range() {
        assert!(matches!(
            HardBounds::new(1.0, 0.0),
            Err(BoundsError::InvalidRange { .. })
        ));
    }
    
    #[test]
    fn test_equal_bounds() {
        assert!(matches!(
            HardBounds::new(0.5, 0.5),
            Err(BoundsError::InvalidRange { .. })
        ));
    }
    
    #[test]
    fn test_non_finite() {
        assert!(matches!(
            HardBounds::new(f64::NAN, 1.0),
            Err(BoundsError::NonFinite)
        ));
        assert!(matches!(
            HardBounds::new(0.0, f64::INFINITY),
            Err(BoundsError::NonFinite)
        ));
    }
    
    #[test]
    fn test_clamp() {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        assert_eq!(bounds.clamp(-0.5), 0.0);
        assert_eq!(bounds.clamp(0.5), 0.5);
        assert_eq!(bounds.clamp(1.5), 1.0);
    }
    
    #[test]
    fn test_violation() {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        assert_eq!(bounds.violation(0.5), 0.0);
        assert_eq!(bounds.violation(1.5), 0.5);
        assert_eq!(bounds.violation(-0.3), 0.3);
    }
    
    #[test]
    fn test_contains() {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        assert!(bounds.contains(0.0));
        assert!(bounds.contains(0.5));
        assert!(bounds.contains(1.0));
        assert!(!bounds.contains(-0.1));
        assert!(!bounds.contains(1.1));
    }
    
    #[test]
    fn test_normalize_denormalize() {
        let bounds = HardBounds::new(10.0, 20.0).unwrap();
        assert_eq!(bounds.normalize(15.0), 0.5);
        assert_eq!(bounds.denormalize(0.5), 15.0);
    }
}
