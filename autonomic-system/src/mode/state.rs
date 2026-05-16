//! Autonomic mode state definitions.

use serde::{Deserialize, Serialize};

/// Primary operating modes (ACT vs CALM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomicMode {
    /// Action mode: High arousal, fast responses, risk tolerant.
    /// Biological analogy: Sympathetic nervous system (fight/flight).
    Act,
    /// Contemplation mode: Low arousal, deliberate, safety focused.
    /// Biological analogy: Parasympathetic nervous system (rest/digest).
    Calm,
    /// Emergency mode: Maximum arousal, survival-only actions.
    Emergency,
    /// Recovery mode: Post-emergency, transitioning to CALM.
    Recovery,
}

impl AutonomicMode {
    /// Returns the base arousal level for this mode.
    pub fn base_arousal(&self) -> f64 {
        match self {
            AutonomicMode::Act => 0.7,
            AutonomicMode::Calm => 0.3,
            AutonomicMode::Emergency => 1.0,
            AutonomicMode::Recovery => 0.4,
        }
    }
    
    /// Returns the risk tolerance for this mode [0, 1].
    pub fn risk_tolerance(&self) -> f64 {
        match self {
            AutonomicMode::Act => 0.6,
            AutonomicMode::Calm => 0.2,
            AutonomicMode::Emergency => 0.1, // Very conservative in emergency
            AutonomicMode::Recovery => 0.3,
        }
    }
    
    /// Returns the processing speed multiplier.
    pub fn speed_factor(&self) -> f64 {
        match self {
            AutonomicMode::Act => 2.0,
            AutonomicMode::Calm => 0.5,
            AutonomicMode::Emergency => 3.0,
            AutonomicMode::Recovery => 0.7,
        }
    }
    
    /// Returns whether reflexes are enabled.
    pub fn reflexes_enabled(&self) -> bool {
        matches!(self, AutonomicMode::Act | AutonomicMode::Emergency)
    }
    
    /// Returns a human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            AutonomicMode::Act => "ACT",
            AutonomicMode::Calm => "CALM",
            AutonomicMode::Emergency => "EMERGENCY",
            AutonomicMode::Recovery => "RECOVERY",
        }
    }
}

impl Default for AutonomicMode {
    fn default() -> Self {
        AutonomicMode::Calm // Default to safe mode
    }
}

/// Current arousal level with dynamics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Arousal {
    /// Current arousal level [0, 1].
    level: f64,
    /// Rate of change per tick.
    velocity: f64,
    /// Target arousal (mode's base).
    target: f64,
}

impl Arousal {
    /// Time constant for arousal dynamics.
    const TAU: f64 = 10.0;
    
    /// Creates a new arousal state.
    pub fn new(level: f64) -> Self {
        Self {
            level: level.clamp(0.0, 1.0),
            velocity: 0.0,
            target: level.clamp(0.0, 1.0),
        }
    }
    
    /// Returns current arousal level.
    pub fn level(&self) -> f64 {
        self.level
    }
    
    /// Returns current velocity.
    pub fn velocity(&self) -> f64 {
        self.velocity
    }
    
    /// Sets target arousal.
    pub fn set_target(&mut self, target: f64) {
        self.target = target.clamp(0.0, 1.0);
    }
    
    /// Applies an immediate stimulus.
    pub fn stimulate(&mut self, amount: f64) {
        self.level = (self.level + amount).clamp(0.0, 1.0);
        self.velocity += amount * 0.5;
    }
    
    /// Updates arousal state (first-order dynamics toward target).
    pub fn update(&mut self, dt: f64) {
        let error = self.target - self.level;
        self.velocity = error / Self::TAU;
        self.level = (self.level + self.velocity * dt).clamp(0.0, 1.0);
    }
    
    /// Returns true if arousal is high.
    pub fn is_high(&self) -> bool {
        self.level > 0.7
    }
    
    /// Returns true if arousal is low.
    pub fn is_low(&self) -> bool {
        self.level < 0.3
    }
    
    /// Returns true if arousal is critically high.
    pub fn is_critical(&self) -> bool {
        self.level > 0.9
    }
}

impl Default for Arousal {
    fn default() -> Self {
        Self::new(AutonomicMode::default().base_arousal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mode_defaults() {
        assert_eq!(AutonomicMode::default(), AutonomicMode::Calm);
    }
    
    #[test]
    fn test_arousal_stimulation() {
        let mut arousal = Arousal::new(0.5);
        arousal.stimulate(0.3);
        assert!(arousal.level() > 0.7);
    }
    
    #[test]
    fn test_arousal_convergence() {
        let mut arousal = Arousal::new(0.9);
        arousal.set_target(0.3);
        
        for _ in 0..100 {
            arousal.update(1.0);
        }
        
        assert!((arousal.level() - 0.3).abs() < 0.1);
    }
}
