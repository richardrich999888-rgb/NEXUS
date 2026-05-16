//! Safety state and summary.

use autonomic_system::mode::state::AutonomicMode;
use developmental_gates::stage::definition::DevelopmentalStage;
use serde::{Deserialize, Serialize};

/// Current safety state of the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyState {
    /// Current autonomic mode.
    pub autonomic_mode: AutonomicMode,
    /// Current development stage.
    pub developmental_stage: DevelopmentalStage,
    /// Current arousal level.
    pub arousal: f64,
    /// Homeostatic health score.
    pub health_score: f64,
    /// Active threats count.
    pub active_threats: usize,
    /// Active constraints count.
    pub active_constraints: usize,
    /// Is system healthy overall?
    pub is_healthy: bool,
}

impl SafetyState {
    /// Returns an overall safety score [0, 1].
    pub fn safety_score(&self) -> f64 {
        let mode_factor = match self.autonomic_mode {
            AutonomicMode::Calm => 1.0,
            AutonomicMode::Recovery => 0.7,
            AutonomicMode::Act => 0.6,
            AutonomicMode::Emergency => 0.3,
        };
        
        let stage_factor = match self.developmental_stage {
            DevelopmentalStage::Infant => 0.9,
            DevelopmentalStage::Child => 0.8,
            DevelopmentalStage::Adolescent => 0.7,
            DevelopmentalStage::Adult => 0.6,
            DevelopmentalStage::Elder => 0.5,
        };
        
        let threat_penalty = (self.active_threats as f64 * 0.1).min(0.5);
        
        let base = (self.health_score + mode_factor + stage_factor) / 3.0;
        (base - threat_penalty).max(0.0)
    }
}

/// Summary of safety layer status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySummary {
    pub homeostasis_health: f64,
    pub autonomic_mode: String,
    pub developmental_stage: String,
    pub immune_threats: usize,
    pub pending_reflexes: usize,
    pub blocked_actions: usize,
    pub overall_status: String,
}

impl SafetySummary {
    /// Creates a summary from component states.
    pub fn from_state(state: &SafetyState, pending_reflexes: usize, blocked_actions: usize) -> Self {
        let overall_status = if state.is_healthy && state.active_threats == 0 {
            "NOMINAL".to_string()
        } else if state.arousal > 0.8 || state.active_threats > 0 {
            "ALERT".to_string()
        } else if !state.is_healthy {
            "DEGRADED".to_string()
        } else {
            "CAUTION".to_string()
        };
        
        Self {
            homeostasis_health: state.health_score,
            autonomic_mode: state.autonomic_mode.name().to_string(),
            developmental_stage: state.developmental_stage.name().to_string(),
            immune_threats: state.active_threats,
            pending_reflexes,
            blocked_actions,
            overall_status,
        }
    }
}
