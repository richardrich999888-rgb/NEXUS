//! # Homeostasis & Feedback Control
//!
//! PATENT CLAIM 10: Biological Feedback Loops for Self-Regulation
//!
//! Implements homeostatic control mechanisms:
//! - Negative feedback loops
//! - Allostasis (adaptive equilibrium)
//! - Circadian rhythm modulation
//! - Set-point regulation

use crate::endocrine::{Hormone, EndocrineState, HormoneLevel};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Set-point for a hormone (target level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPoint {
    /// Target hormone level
    pub target: f64,
    /// Tolerance band (acceptable deviation)
    pub tolerance: f64,
    /// Adaptation rate (how fast set-point changes)
    pub adaptation_rate: f64,
}

impl Default for SetPoint {
    fn default() -> Self {
        Self {
            target: 0.5,
            tolerance: 0.1,
            adaptation_rate: 0.01,
        }
    }
}

impl SetPoint {
    pub fn new(target: f64) -> Self {
        Self {
            target: target.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Check if level is within tolerance
    pub fn in_tolerance(&self, level: f64) -> bool {
        (level - self.target).abs() <= self.tolerance
    }

    /// Calculate error from set-point
    pub fn error(&self, level: f64) -> f64 {
        level - self.target
    }

    /// Adapt set-point towards current level (allostasis)
    pub fn adapt(&mut self, current_level: f64) {
        // Slowly move target towards chronically elevated/depressed levels
        let error = current_level - self.target;
        self.target += error * self.adaptation_rate;
        self.target = self.target.clamp(0.2, 0.8); // Keep in safe range
    }
}

/// Feedback loop types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// Negative feedback: high levels → reduce production
    Negative,
    /// Positive feedback: high levels → increase production (rare, unstable)
    Positive,
}

/// Feedback loop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackLoop {
    /// Type of feedback
    pub feedback_type: FeedbackType,
    /// Sensitivity (how strongly feedback affects secretion)
    pub sensitivity: f64,
    /// Delay before feedback kicks in (seconds)
    pub delay: f64,
    /// Maximum feedback effect
    pub max_effect: f64,
}

impl Default for FeedbackLoop {
    fn default() -> Self {
        Self {
            feedback_type: FeedbackType::Negative,
            sensitivity: 1.0,
            delay: 60.0, // 1 minute delay
            max_effect: 0.8,
        }
    }
}

impl FeedbackLoop {
    /// Calculate feedback factor [0.0, 2.0]
    /// 
    /// For negative feedback: >0.5 means level too high → reduce
    /// For positive feedback: >0.5 means level too high → increase more
    pub fn calculate(&self, current_level: f64, set_point: &SetPoint) -> f64 {
        let error = set_point.error(current_level);

        match self.feedback_type {
            FeedbackType::Negative => {
                // High level → strong inhibition
                if error > 0.0 {
                    let inhibition = (error * self.sensitivity).min(self.max_effect);
                    1.0 - inhibition
                } else {
                    // Low level → slight stimulation
                    let stimulation = (error.abs() * self.sensitivity * 0.5).min(0.3);
                    1.0 + stimulation
                }
            }
            FeedbackType::Positive => {
                // Positive feedback (rare, use carefully)
                if error > 0.0 {
                    let amplification = (error * self.sensitivity).min(self.max_effect);
                    1.0 + amplification
                } else {
                    1.0 - (error.abs() * self.sensitivity * 0.5).min(0.3)
                }
            }
        }
    }
}

/// Circadian rhythm controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianController {
    /// Phase offsets for each hormone (radians)
    pub phase_offsets: HashMap<Hormone, f64>,
    /// Amplitude of circadian variation [0.0, 1.0]
    pub amplitude: f64,
    /// Current time of day (seconds since midnight)
    pub time_of_day: f64,
}

impl Default for CircadianController {
    fn default() -> Self {
        Self::new()
    }
}

impl CircadianController {
    pub fn new() -> Self {
        let mut phase_offsets = HashMap::new();

        // Set biologically-inspired phase offsets
        // Cortisol peaks in early morning (around 8 AM)
        phase_offsets.insert(Hormone::Cortisol, -std::f64::consts::PI / 2.0);
        // Melatonin-like (serotonin precursor) peaks at night
        phase_offsets.insert(Hormone::Serotonin, std::f64::consts::PI);
        // Growth hormone peaks during sleep
        phase_offsets.insert(Hormone::GrowthHormone, std::f64::consts::PI * 0.75);
        // Dopamine and adrenaline peak during day
        phase_offsets.insert(Hormone::Dopamine, 0.0);
        phase_offsets.insert(Hormone::Adrenaline, 0.0);
        // Others have minimal circadian variation
        phase_offsets.insert(Hormone::Oxytocin, 0.0);
        phase_offsets.insert(Hormone::Endorphins, 0.0);
        phase_offsets.insert(Hormone::Norepinephrine, -std::f64::consts::PI / 4.0);

        Self {
            phase_offsets,
            amplitude: 0.15, // 15% variation
            time_of_day: 43200.0, // Start at noon
        }
    }

    /// Advance time
    pub fn tick(&mut self, delta_seconds: f64) {
        self.time_of_day = (self.time_of_day + delta_seconds) % 86400.0;
    }

    /// Get circadian modulation factor for a hormone
    pub fn modulation(&self, hormone: Hormone) -> f64 {
        let phase = self.phase_offsets.get(&hormone).copied().unwrap_or(0.0);
        let daily_phase = (self.time_of_day / 86400.0) * 2.0 * std::f64::consts::PI;
        1.0 + self.amplitude * (daily_phase + phase).sin()
    }

    /// Apply circadian modulation to all hormone levels
    pub fn apply(&self, state: &mut EndocrineState) {
        for hormone in Hormone::all() {
            let factor = self.modulation(hormone);
            if let Some(level) = state.levels.get_mut(&hormone) {
                // Modulate towards circadian-adjusted baseline
                let adjusted_baseline = 0.5 * factor;
                let current = level.level;
                // Gently pull towards adjusted baseline
                level.level = current * 0.99 + adjusted_baseline * 0.01;
            }
        }
    }
}

/// Allostasis manager - adapts set-points over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllostasisManager {
    /// Set-points for each hormone
    pub set_points: HashMap<Hormone, SetPoint>,
    /// Allostatic load (chronic deviation accumulation)
    pub allostatic_load: f64,
    /// History window for averaging (samples)
    history_size: usize,
    /// Recent level history for each hormone
    history: HashMap<Hormone, Vec<f64>>,
}

impl Default for AllostasisManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AllostasisManager {
    pub fn new() -> Self {
        let mut set_points = HashMap::new();
        let mut history = HashMap::new();

        for hormone in Hormone::all() {
            set_points.insert(hormone, SetPoint::default());
            history.insert(hormone, Vec::with_capacity(100));
        }

        Self {
            set_points,
            allostatic_load: 0.0,
            history_size: 100,
            history,
        }
    }

    /// Record current state for history
    pub fn record(&mut self, state: &EndocrineState) {
        for hormone in Hormone::all() {
            let level = state.levels.get(&hormone).map(|l| l.level).unwrap_or(0.5);

            if let Some(hist) = self.history.get_mut(&hormone) {
                if hist.len() >= self.history_size {
                    hist.remove(0);
                }
                hist.push(level);
            }
        }
    }

    /// Calculate average for a hormone
    fn average(&self, hormone: Hormone) -> Option<f64> {
        self.history.get(&hormone).and_then(|h| {
            if h.is_empty() {
                None
            } else {
                Some(h.iter().sum::<f64>() / h.len() as f64)
            }
        })
    }

    /// Update set-points based on chronic levels (allostasis)
    pub fn adapt_setpoints(&mut self) {
        for hormone in Hormone::all() {
            if let Some(avg) = self.average(hormone) {
                if let Some(sp) = self.set_points.get_mut(&hormone) {
                    sp.adapt(avg);
                }
            }
        }
    }

    /// Calculate allostatic load (chronic stress indicator)
    pub fn calculate_load(&mut self, state: &EndocrineState) {
        let mut total_deviation = 0.0;

        for hormone in Hormone::all() {
            let level = state.levels.get(&hormone).map(|l| l.level).unwrap_or(0.5);
            let set_point = self.set_points.get(&hormone).map(|s| s.target).unwrap_or(0.5);
            total_deviation += (level - set_point).abs();
        }

        // Average deviation across hormones
        let avg_deviation = total_deviation / 8.0;

        // Accumulate load slowly
        self.allostatic_load = self.allostatic_load * 0.99 + avg_deviation * 0.01;
    }

    /// Check if system is in homeostatic balance
    pub fn in_balance(&self, state: &EndocrineState) -> bool {
        for hormone in Hormone::all() {
            let level = state.levels.get(&hormone).map(|l| l.level).unwrap_or(0.5);
            if let Some(sp) = self.set_points.get(&hormone) {
                if !sp.in_tolerance(level) {
                    return false;
                }
            }
        }
        true
    }
}

/// Complete homeostasis controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeostasisController {
    /// Feedback loops for each hormone
    pub feedback_loops: HashMap<Hormone, FeedbackLoop>,
    /// Circadian rhythm controller
    pub circadian: CircadianController,
    /// Allostasis manager
    pub allostasis: AllostasisManager,
}

impl Default for HomeostasisController {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeostasisController {
    pub fn new() -> Self {
        let mut feedback_loops = HashMap::new();

        // Configure hormone-specific feedback
        for hormone in Hormone::all() {
            let mut fb = FeedbackLoop::default();

            // Custom sensitivity per hormone
            match hormone {
                Hormone::Cortisol => {
                    fb.sensitivity = 1.5; // Strong cortisol feedback
                    fb.delay = 120.0; // 2 min delay
                }
                Hormone::Adrenaline => {
                    fb.sensitivity = 2.0; // Very strong (emergency hormone)
                    fb.delay = 30.0; // Fast feedback
                }
                Hormone::GrowthHormone => {
                    fb.sensitivity = 0.5; // Weak (slow-acting)
                    fb.delay = 300.0; // Long delay
                }
                Hormone::Oxytocin => {
                    fb.sensitivity = 0.8;
                    fb.delay = 60.0;
                }
                _ => {} // Use defaults
            }

            feedback_loops.insert(hormone, fb);
        }

        Self {
            feedback_loops,
            circadian: CircadianController::new(),
            allostasis: AllostasisManager::new(),
        }
    }

    /// Tick the homeostasis system
    pub fn tick(&mut self, delta_time: f64, state: &mut EndocrineState) {
        // 1. Advance circadian clock
        self.circadian.tick(delta_time);

        // 2. Apply circadian modulation
        self.circadian.apply(state);

        // 3. Record history for allostasis
        self.allostasis.record(state);

        // 4. Calculate allostatic load
        self.allostasis.calculate_load(state);
    }

    /// Get feedback factor for a hormone
    pub fn feedback_factor(&self, hormone: Hormone, state: &EndocrineState) -> f64 {
        let level = state.levels.get(&hormone).map(|l| l.level).unwrap_or(0.5);

        let fb = self.feedback_loops.get(&hormone);
        let sp = self.allostasis.set_points.get(&hormone);

        match (fb, sp) {
            (Some(feedback), Some(set_point)) => feedback.calculate(level, set_point),
            _ => 1.0, // No feedback
        }
    }

    /// Check system health
    pub fn health_status(&self, state: &EndocrineState) -> HealthStatus {
        let load = self.allostasis.allostatic_load;
        let in_balance = self.allostasis.in_balance(state);

        if load < 0.1 && in_balance {
            HealthStatus::Optimal
        } else if load < 0.3 {
            HealthStatus::Normal
        } else if load < 0.5 {
            HealthStatus::Stressed
        } else {
            HealthStatus::Critical
        }
    }
}

/// System health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Optimal,
    Normal,
    Stressed,
    Critical,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Optimal => write!(f, "Optimal"),
            HealthStatus::Normal => write!(f, "Normal"),
            HealthStatus::Stressed => write!(f, "Stressed"),
            HealthStatus::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setpoint_tolerance() {
        let sp = SetPoint::new(0.5);

        assert!(sp.in_tolerance(0.55));
        assert!(sp.in_tolerance(0.45));
        assert!(!sp.in_tolerance(0.7));
    }

    #[test]
    fn test_negative_feedback() {
        let fb = FeedbackLoop::default();
        let sp = SetPoint::new(0.5);

        // High level → inhibition
        let factor_high = fb.calculate(0.9, &sp);
        assert!(factor_high < 1.0, "High level should inhibit");

        // Low level → stimulation
        let factor_low = fb.calculate(0.2, &sp);
        assert!(factor_low > 1.0, "Low level should stimulate");
    }

    #[test]
    fn test_circadian_variation() {
        let circadian = CircadianController::new();

        // Cortisol should peak in morning
        let mut controller_morning = circadian.clone();
        controller_morning.time_of_day = 28800.0; // 8 AM

        let mut controller_evening = circadian.clone();
        controller_evening.time_of_day = 72000.0; // 8 PM

        let morning_mod = controller_morning.modulation(Hormone::Cortisol);
        let evening_mod = controller_evening.modulation(Hormone::Cortisol);

        assert!(
            morning_mod > evening_mod,
            "Cortisol should be higher in morning"
        );
    }

    #[test]
    fn test_homeostasis_health() {
        let controller = HomeostasisController::new();
        let state = EndocrineState::new([0u8; 32]);

        let status = controller.health_status(&state);
        assert!(status == HealthStatus::Optimal || status == HealthStatus::Normal);
    }

    #[test]
    fn test_allostatic_adaptation() {
        let mut manager = AllostasisManager::new();
        let mut state = EndocrineState::new([0u8; 32]);

        // Chronically elevate cortisol
        if let Some(level) = state.levels.get_mut(&Hormone::Cortisol) {
            level.level = 0.8;
        }

        // Record multiple times
        for _ in 0..50 {
            manager.record(&state);
        }

        manager.adapt_setpoints();

        // Set-point should have moved up
        let cortisol_sp = manager.set_points.get(&Hormone::Cortisol).unwrap();
        assert!(cortisol_sp.target > 0.5, "Set-point should adapt upward");
    }
}
