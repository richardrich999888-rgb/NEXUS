//! Adaptive setpoint management.
//!
//! Setpoints can adapt over time (meta-homeostasis) to handle
//! changing conditions while maintaining stability.

use serde::{Deserialize, Serialize};
use crate::core::bounds::HardBounds;

/// Configuration for adaptive setpoint behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetpointConfig {
    /// Rate of setpoint adaptation (0 = fixed, 1 = immediate).
    pub adaptation_rate: f64,
    
    /// Minimum duration before setpoint can adapt.
    pub stability_threshold_steps: u64,
    
    /// Maximum rate of change per step.
    pub max_delta_per_step: f64,
    
    /// If true, setpoint adapts toward current value under sustained deviation.
    pub enable_allostatic_load: bool,
}

impl Default for SetpointConfig {
    fn default() -> Self {
        Self {
            adaptation_rate: 0.01,
            stability_threshold_steps: 100,
            max_delta_per_step: 0.001,
            enable_allostatic_load: false,
        }
    }
}

impl SetpointConfig {
    /// Creates a fixed setpoint configuration (no adaptation).
    pub fn fixed() -> Self {
        Self {
            adaptation_rate: 0.0,
            stability_threshold_steps: u64::MAX,
            max_delta_per_step: 0.0,
            enable_allostatic_load: false,
        }
    }
    
    /// Creates a slowly adapting setpoint.
    pub fn slow_adaptation() -> Self {
        Self {
            adaptation_rate: 0.001,
            stability_threshold_steps: 500,
            max_delta_per_step: 0.0001,
            enable_allostatic_load: true,
        }
    }
}

/// Adaptive setpoint that can adjust based on sustained conditions.
///
/// Models biological allostatic adaptation where chronic stress
/// shifts the baseline (bad) or recovery resets it (good).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveSetpoint {
    /// Current setpoint value.
    current: f64,
    
    /// Original setpoint (for reset/reference).
    original: f64,
    
    /// Valid bounds for setpoint itself.
    bounds: HardBounds,
    
    /// Configuration for adaptation behavior.
    config: SetpointConfig,
    
    /// Steps since last significant change.
    stable_steps: u64,
    
    /// Accumulated deviation (for allostatic load).
    accumulated_deviation: f64,
}

impl AdaptiveSetpoint {
    /// Creates a new adaptive setpoint.
    pub fn new(initial: f64, bounds: HardBounds, config: SetpointConfig) -> Self {
        let clamped = bounds.clamp(initial);
        Self {
            current: clamped,
            original: clamped,
            bounds,
            config,
            stable_steps: 0,
            accumulated_deviation: 0.0,
        }
    }
    
    /// Creates a fixed (non-adaptive) setpoint.
    pub fn fixed(value: f64, bounds: HardBounds) -> Self {
        Self::new(value, bounds, SetpointConfig::fixed())
    }
    
    /// Returns current setpoint value.
    pub fn value(&self) -> f64 {
        self.current
    }
    
    /// Returns original setpoint value.
    pub fn original(&self) -> f64 {
        self.original
    }
    
    /// Updates setpoint based on current metric value.
    /// Returns the change in setpoint (if any).
    pub fn update(&mut self, current_value: f64) -> f64 {
        if self.config.adaptation_rate == 0.0 {
            return 0.0;
        }
        
        let old = self.current;
        let deviation = current_value - self.current;
        
        // Accumulate deviation for allostatic load
        if self.config.enable_allostatic_load {
            self.accumulated_deviation += deviation.abs();
        }
        
        // Check stability threshold
        self.stable_steps += 1;
        if self.stable_steps < self.config.stability_threshold_steps {
            return 0.0;
        }
        
        // Compute adaptation
        let raw_delta = self.config.adaptation_rate * deviation;
        let clamped_delta = raw_delta.clamp(
            -self.config.max_delta_per_step,
            self.config.max_delta_per_step
        );
        
        // Apply adaptation
        let new_value = self.current + clamped_delta;
        self.current = self.bounds.clamp(new_value);
        
        self.current - old
    }
    
    /// Resets setpoint to original value.
    pub fn reset(&mut self) {
        self.current = self.original;
        self.accumulated_deviation = 0.0;
        self.stable_steps = 0;
    }
    
    /// Returns accumulated allostatic load.
    pub fn allostatic_load(&self) -> f64 {
        self.accumulated_deviation
    }
    
    /// Returns shift from original setpoint.
    pub fn drift(&self) -> f64 {
        self.current - self.original
    }
    
    /// Returns true if setpoint has drifted significantly from original.
    pub fn is_drifted(&self, threshold: f64) -> bool {
        self.drift().abs() > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixed_setpoint() {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        let mut sp = AdaptiveSetpoint::fixed(0.5, bounds);
        
        // Should not change
        for _ in 0..1000 {
            sp.update(0.8);
        }
        
        assert_eq!(sp.value(), 0.5);
    }
    
    #[test]
    fn test_adaptive_setpoint() {
        let bounds = HardBounds::new(0.0, 1.0).unwrap();
        let config = SetpointConfig {
            adaptation_rate: 0.1,
            stability_threshold_steps: 10,
            max_delta_per_step: 0.01,
            enable_allostatic_load: false,
        };
        let mut sp = AdaptiveSetpoint::new(0.5, bounds, config);
        
        // Drive toward 0.8
        for _ in 0..100 {
            sp.update(0.8);
        }
        
        // Should have moved toward 0.8
        assert!(sp.value() > 0.5);
        assert!(sp.drift() > 0.0);
    }
}
