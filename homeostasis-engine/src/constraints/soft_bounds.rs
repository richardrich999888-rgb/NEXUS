//! Soft bounds - preference regions with penalties.
//!
//! Unlike hard bounds, soft bounds can be exceeded but incur a penalty.
//! Used for optimization objectives rather than safety constraints.

use serde::{Deserialize, Serialize};

/// Soft bounds with configurable penalty.
///
/// Values outside soft bounds are penalized but not prevented.
/// Useful for expressing preferences while maintaining flexibility.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SoftBounds {
    /// Preferred lower limit.
    pub lower: f64,
    /// Preferred upper limit.
    pub upper: f64,
    /// Penalty multiplier for violations.
    pub penalty_rate: f64,
    /// Penalty type.
    pub penalty_type: PenaltyType,
}

/// Type of penalty function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenaltyType {
    /// Linear penalty: penalty = rate * |violation|
    Linear,
    /// Quadratic penalty: penalty = rate * violation^2
    Quadratic,
    /// Exponential penalty: penalty = rate * (exp(violation) - 1)
    Exponential,
}

impl SoftBounds {
    /// Creates new soft bounds.
    pub fn new(lower: f64, upper: f64, penalty_rate: f64) -> Self {
        assert!(lower < upper, "lower must be less than upper");
        assert!(penalty_rate >= 0.0, "penalty_rate must be non-negative");
        
        Self {
            lower,
            upper,
            penalty_rate,
            penalty_type: PenaltyType::Quadratic,
        }
    }
    
    /// Creates soft bounds with specified penalty type.
    pub fn with_penalty_type(mut self, penalty_type: PenaltyType) -> Self {
        self.penalty_type = penalty_type;
        self
    }
    
    /// Calculates penalty for a value.
    pub fn penalty(&self, value: f64) -> f64 {
        let violation = self.violation(value);
        
        if violation <= 0.0 {
            return 0.0;
        }
        
        match self.penalty_type {
            PenaltyType::Linear => self.penalty_rate * violation,
            PenaltyType::Quadratic => self.penalty_rate * violation * violation,
            PenaltyType::Exponential => self.penalty_rate * (violation.exp() - 1.0),
        }
    }
    
    /// Returns violation magnitude (0 if within bounds).
    pub fn violation(&self, value: f64) -> f64 {
        if value > self.upper {
            value - self.upper
        } else if value < self.lower {
            self.lower - value
        } else {
            0.0
        }
    }
    
    /// Returns true if value is within soft bounds.
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }
    
    /// Returns gradient of penalty function.
    pub fn penalty_gradient(&self, value: f64) -> f64 {
        if value > self.upper {
            let v = value - self.upper;
            match self.penalty_type {
                PenaltyType::Linear => self.penalty_rate,
                PenaltyType::Quadratic => 2.0 * self.penalty_rate * v,
                PenaltyType::Exponential => self.penalty_rate * v.exp(),
            }
        } else if value < self.lower {
            let v = self.lower - value;
            match self.penalty_type {
                PenaltyType::Linear => -self.penalty_rate,
                PenaltyType::Quadratic => -2.0 * self.penalty_rate * v,
                PenaltyType::Exponential => -self.penalty_rate * v.exp(),
            }
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_penalty_within_bounds() {
        let bounds = SoftBounds::new(0.0, 1.0, 1.0);
        assert_eq!(bounds.penalty(0.5), 0.0);
    }
    
    #[test]
    fn test_linear_penalty() {
        let bounds = SoftBounds::new(0.0, 1.0, 2.0)
            .with_penalty_type(PenaltyType::Linear);
        
        assert_eq!(bounds.penalty(1.5), 1.0); // 2.0 * 0.5
        assert_eq!(bounds.penalty(-0.5), 1.0); // 2.0 * 0.5
    }
    
    #[test]
    fn test_quadratic_penalty() {
        let bounds = SoftBounds::new(0.0, 1.0, 1.0);
        
        assert_eq!(bounds.penalty(1.5), 0.25); // 1.0 * 0.5^2
    }
}
