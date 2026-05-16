//! Feedback loop primitives.
//!
//! Provides configurable feedback mechanisms for homeostatic control.

use serde::{Deserialize, Serialize};

/// Configuration for a feedback loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// Type of feedback (negative stabilizes, positive amplifies).
    pub feedback_type: FeedbackType,
    
    /// Proportional gain (K_p).
    pub proportional_gain: f64,
    
    /// Integral gain (K_i) for accumulated error.
    pub integral_gain: f64,
    
    /// Derivative gain (K_d) for rate of change.
    pub derivative_gain: f64,
    
    /// Maximum output magnitude.
    pub max_output: f64,
    
    /// Dead zone: errors below this are ignored.
    pub dead_zone: f64,
}

/// Type of feedback loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// Negative feedback: opposes deviation from setpoint.
    /// This is the default for homeostatic control.
    Negative,
    
    /// Positive feedback: amplifies deviation.
    /// Used sparingly (e.g., immune response escalation).
    Positive,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            feedback_type: FeedbackType::Negative,
            proportional_gain: 0.5,
            integral_gain: 0.0,
            derivative_gain: 0.0,
            max_output: 1.0,
            dead_zone: 0.0,
        }
    }
}

impl FeedbackConfig {
    /// Creates a simple proportional controller.
    pub fn proportional(gain: f64) -> Self {
        Self {
            proportional_gain: gain,
            ..Default::default()
        }
    }
    
    /// Creates a PID controller.
    pub fn pid(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            proportional_gain: kp,
            integral_gain: ki,
            derivative_gain: kd,
            ..Default::default()
        }
    }
}

/// A feedback loop controller with state.
#[derive(Debug, Clone)]
pub struct FeedbackLoop {
    config: FeedbackConfig,
    
    /// Accumulated integral error.
    integral: f64,
    
    /// Previous error for derivative calculation.
    prev_error: f64,
    
    /// Whether first step (no derivative yet).
    first_step: bool,
}

impl FeedbackLoop {
    /// Creates a new feedback loop with the given configuration.
    pub fn new(config: FeedbackConfig) -> Self {
        Self {
            config,
            integral: 0.0,
            prev_error: 0.0,
            first_step: true,
        }
    }
    
    /// Creates a simple proportional feedback loop.
    pub fn proportional(gain: f64) -> Self {
        Self::new(FeedbackConfig::proportional(gain))
    }
    
    /// Computes the feedback correction for the given error.
    pub fn compute(&mut self, error: f64) -> f64 {
        // Apply dead zone
        let effective_error = if error.abs() < self.config.dead_zone {
            0.0
        } else {
            error
        };
        
        // Proportional term
        let p_term = self.config.proportional_gain * effective_error;
        
        // Integral term
        self.integral += effective_error;
        let i_term = self.config.integral_gain * self.integral;
        
        // Derivative term
        let d_term = if self.first_step {
            self.first_step = false;
            0.0
        } else {
            self.config.derivative_gain * (effective_error - self.prev_error)
        };
        self.prev_error = effective_error;
        
        // Total output
        let mut output = p_term + i_term + d_term;
        
        // Apply feedback sign
        if self.config.feedback_type == FeedbackType::Negative {
            output = -output;
        }
        
        // Clamp output
        output.clamp(-self.config.max_output, self.config.max_output)
    }
    
    /// Resets the feedback loop state.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.first_step = true;
    }
    
    /// Returns accumulated integral.
    pub fn integral(&self) -> f64 {
        self.integral
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proportional() {
        let mut fb = FeedbackLoop::proportional(1.0);
        
        // Positive error should give negative correction
        let correction = fb.compute(0.5);
        assert_eq!(correction, -0.5);
        
        // Negative error should give positive correction
        let correction = fb.compute(-0.5);
        assert_eq!(correction, 0.5);
    }
    
    #[test]
    fn test_dead_zone() {
        let config = FeedbackConfig {
            dead_zone: 0.1,
            ..FeedbackConfig::proportional(1.0)
        };
        let mut fb = FeedbackLoop::new(config);
        
        // Small error within dead zone
        let correction = fb.compute(0.05);
        assert_eq!(correction, 0.0);
        
        // Error outside dead zone
        let correction = fb.compute(0.2);
        assert_eq!(correction, -0.2);
    }
    
    #[test]
    fn test_max_output() {
        let config = FeedbackConfig {
            max_output: 0.1,
            ..FeedbackConfig::proportional(10.0)
        };
        let mut fb = FeedbackLoop::new(config);
        
        // Large error should be clamped
        let correction = fb.compute(1.0);
        assert_eq!(correction, -0.1);
    }
}
