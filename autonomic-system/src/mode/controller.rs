//! Autonomic mode controller.

use crate::mode::state::{AutonomicMode, Arousal};
use crate::reflex::response::{ReflexResponse, ReflexType};
use crate::regulation::transition::{ModeTransition, TransitionTrigger};
use homeostasis_engine::controller::multi_objective::MultiObjectiveController;
use homeostasis_engine::core::metric::MetricId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Configuration for the autonomic controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    /// Arousal threshold for ACT mode.
    pub act_threshold: f64,
    /// Arousal threshold for CALM mode.
    pub calm_threshold: f64,
    /// Arousal threshold for EMERGENCY mode.
    pub emergency_threshold: f64,
    /// Minimum time in mode before transition (ticks).
    pub min_mode_duration: u64,
    /// Time constant for arousal dynamics.
    pub arousal_tau: f64,
    /// Stress metric ID in homeostasis.
    pub stress_metric_id: u32,
    /// Maximum reflexes in queue.
    pub max_reflex_queue: usize,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            act_threshold: 0.6,
            calm_threshold: 0.4,
            emergency_threshold: 0.9,
            min_mode_duration: 10,
            arousal_tau: 10.0,
            stress_metric_id: 1, // Standard stress metric
            max_reflex_queue: 16,
        }
    }
}

/// Main autonomic controller.
pub struct AutonomicController {
    /// Current operating mode.
    mode: AutonomicMode,
    /// Current arousal state.
    arousal: Arousal,
    /// Time (ticks) in current mode.
    mode_duration: u64,
    /// Configuration.
    config: ControllerConfig,
    /// Pending reflex responses.
    reflex_queue: VecDeque<ReflexResponse>,
    /// Transition history.
    transitions: VecDeque<ModeTransition>,
    /// Current monotonic time.
    current_time: u64,
}

impl AutonomicController {
    /// Creates a new controller with default config.
    pub fn new(config: ControllerConfig) -> Self {
        let mode = AutonomicMode::default();
        let arousal = Arousal::new(mode.base_arousal());
        
        Self {
            mode,
            arousal,
            mode_duration: 0,
            config,
            reflex_queue: VecDeque::new(),
            transitions: VecDeque::with_capacity(100),
            current_time: 0,
        }
    }
    
    /// Returns current mode.
    pub fn mode(&self) -> AutonomicMode {
        self.mode
    }
    
    /// Returns current arousal.
    pub fn arousal(&self) -> &Arousal {
        &self.arousal
    }
    
    /// Returns mutable arousal for external stimulation.
    pub fn arousal_mut(&mut self) -> &mut Arousal {
        &mut self.arousal
    }
    
    /// Returns time in current mode.
    pub fn mode_duration(&self) -> u64 {
        self.mode_duration
    }
    
    /// Returns pending reflexes.
    pub fn pending_reflexes(&self) -> &VecDeque<ReflexResponse> {
        &self.reflex_queue
    }
    
    /// Takes the next reflex response.
    pub fn take_reflex(&mut self) -> Option<ReflexResponse> {
        self.reflex_queue.pop_front()
    }
    
    /// Applies external stimulus to arousal.
    pub fn stimulate(&mut self, amount: f64) {
        self.arousal.stimulate(amount);
    }
    
    /// Updates controller state based on homeostatic metrics.
    pub fn update_from_homeostasis(
        &mut self,
        homeostasis: &MultiObjectiveController,
        dt: f64,
    ) -> Option<ModeTransition> {
        // Get stress metric
        let stress = homeostasis
            .get_metric(MetricId(self.config.stress_metric_id))
            .map(|m| m.value())
            .unwrap_or(0.5);
        
        // Stress affects arousal
        self.arousal.set_target(stress * 0.8 + self.mode.base_arousal() * 0.2);
        self.arousal.update(dt);
        
        self.tick_internal(dt)
    }
    
    /// Updates controller state (without homeostasis input).
    pub fn tick(&mut self, dt: f64) -> Option<ModeTransition> {
        self.arousal.set_target(self.mode.base_arousal());
        self.arousal.update(dt);
        self.tick_internal(dt)
    }
    
    fn tick_internal(&mut self, dt: f64) -> Option<ModeTransition> {
        self.current_time += 1;
        self.mode_duration += 1;
        
        // Check for mode transitions
        let new_mode = self.determine_mode();
        
        if new_mode != self.mode && self.mode_duration >= self.config.min_mode_duration {
            return Some(self.transition_to(new_mode));
        }
        
        // Generate reflexes if in action mode
        if self.mode.reflexes_enabled() && self.arousal.is_critical() {
            self.queue_reflex(ReflexResponse::new(
                ReflexType::EmergencyBrake,
                1.0,
                self.current_time,
            ));
        }
        
        None
    }
    
    fn determine_mode(&self) -> AutonomicMode {
        let level = self.arousal.level();
        
        if level >= self.config.emergency_threshold {
            AutonomicMode::Emergency
        } else if level >= self.config.act_threshold {
            if self.mode == AutonomicMode::Emergency {
                AutonomicMode::Recovery
            } else {
                AutonomicMode::Act
            }
        } else if level <= self.config.calm_threshold {
            AutonomicMode::Calm
        } else {
            // Hysteresis: stay in current mode
            self.mode
        }
    }
    
    fn transition_to(&mut self, new_mode: AutonomicMode) -> ModeTransition {
        let transition = ModeTransition {
            from: self.mode,
            to: new_mode,
            trigger: TransitionTrigger::ArousalLevel(self.arousal.level()),
            timestamp: self.current_time,
        };
        
        self.mode = new_mode;
        self.mode_duration = 0;
        
        // Record transition
        self.transitions.push_back(transition.clone());
        if self.transitions.len() > 100 {
            self.transitions.pop_front();
        }
        
        transition
    }
    
    fn queue_reflex(&mut self, reflex: ReflexResponse) {
        if self.reflex_queue.len() < self.config.max_reflex_queue {
            self.reflex_queue.push_back(reflex);
        }
    }
    
    /// Forces a mode transition (for testing or emergency override).
    pub fn force_mode(&mut self, mode: AutonomicMode) -> ModeTransition {
        self.transition_to(mode)
    }
    
    /// Returns recent transitions.
    pub fn recent_transitions(&self) -> &VecDeque<ModeTransition> {
        &self.transitions
    }
    
    /// Returns current behavior modifiers.
    pub fn behavior_modifiers(&self) -> BehaviorModifiers {
        BehaviorModifiers {
            risk_tolerance: self.mode.risk_tolerance(),
            speed_factor: self.mode.speed_factor(),
            reflexes_enabled: self.mode.reflexes_enabled(),
            arousal_level: self.arousal.level(),
        }
    }
}

/// Current behavior modifiers based on autonomic state.
#[derive(Debug, Clone, Copy)]
pub struct BehaviorModifiers {
    /// How much risk is acceptable [0, 1].
    pub risk_tolerance: f64,
    /// Processing speed multiplier.
    pub speed_factor: f64,
    /// Whether automatic reflexes are enabled.
    pub reflexes_enabled: bool,
    /// Current arousal level.
    pub arousal_level: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_mode() {
        let controller = AutonomicController::new(ControllerConfig::default());
        assert_eq!(controller.mode(), AutonomicMode::Calm);
    }
    
    #[test]
    fn test_stimulation_triggers_mode_change() {
        let mut controller = AutonomicController::new(ControllerConfig {
            min_mode_duration: 1,
            ..Default::default()
        });
        
        // Stimulate to high arousal
        controller.stimulate(0.8);
        
        // Tick to process
        let transition = controller.tick(1.0);
        
        // Should transition to ACT or higher
        assert!(controller.mode() != AutonomicMode::Calm || transition.is_some());
    }
    
    #[test]
    fn test_behavior_modifiers() {
        let controller = AutonomicController::new(ControllerConfig::default());
        let modifiers = controller.behavior_modifiers();
        
        // CALM mode should have low risk tolerance
        assert!(modifiers.risk_tolerance < 0.5);
        assert!(!modifiers.reflexes_enabled);
    }
}
