//! Mode transition types and triggers.

use crate::mode::state::AutonomicMode;
use serde::{Deserialize, Serialize};

/// Trigger that caused a mode transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionTrigger {
    /// Arousal level crossed threshold.
    ArousalLevel(f64),
    /// External threat detected.
    ThreatDetected { severity: f64 },
    /// Homeostatic bounds violated.
    BoundsViolation { metric_id: u32 },
    /// Manual override.
    ManualOverride,
    /// Recovery timer expired.
    RecoveryComplete,
    /// System initialization.
    Initialization,
}

impl TransitionTrigger {
    /// Returns priority (for logging/analysis).
    pub fn priority(&self) -> u8 {
        match self {
            TransitionTrigger::ManualOverride => 10,
            TransitionTrigger::ThreatDetected { .. } => 9,
            TransitionTrigger::BoundsViolation { .. } => 8,
            TransitionTrigger::ArousalLevel(_) => 5,
            TransitionTrigger::RecoveryComplete => 3,
            TransitionTrigger::Initialization => 1,
        }
    }
}

/// Record of a mode transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    /// Previous mode.
    pub from: AutonomicMode,
    /// New mode.
    pub to: AutonomicMode,
    /// What triggered the transition.
    pub trigger: TransitionTrigger,
    /// When it occurred.
    pub timestamp: u64,
}

impl ModeTransition {
    /// Returns true if this was an escalation (CALM → ACT → EMERGENCY).
    pub fn is_escalation(&self) -> bool {
        matches!(
            (&self.from, &self.to),
            (AutonomicMode::Calm, AutonomicMode::Act)
                | (AutonomicMode::Calm, AutonomicMode::Emergency)
                | (AutonomicMode::Act, AutonomicMode::Emergency)
        )
    }
    
    /// Returns true if this was a de-escalation.
    pub fn is_deescalation(&self) -> bool {
        matches!(
            (&self.from, &self.to),
            (AutonomicMode::Emergency, AutonomicMode::Recovery)
                | (AutonomicMode::Recovery, AutonomicMode::Calm)
                | (AutonomicMode::Act, AutonomicMode::Calm)
                | (AutonomicMode::Emergency, AutonomicMode::Act)
        )
    }
    
    /// Returns a human-readable description.
    pub fn description(&self) -> String {
        format!(
            "{} → {} ({})",
            self.from.name(),
            self.to.name(),
            match &self.trigger {
                TransitionTrigger::ArousalLevel(l) => format!("arousal={:.2}", l),
                TransitionTrigger::ThreatDetected { severity } => format!("threat severity={:.2}", severity),
                TransitionTrigger::BoundsViolation { metric_id } => format!("bounds violation metric={}", metric_id),
                TransitionTrigger::ManualOverride => "manual".to_string(),
                TransitionTrigger::RecoveryComplete => "recovery".to_string(),
                TransitionTrigger::Initialization => "init".to_string(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_escalation_detection() {
        let transition = ModeTransition {
            from: AutonomicMode::Calm,
            to: AutonomicMode::Emergency,
            trigger: TransitionTrigger::ThreatDetected { severity: 0.9 },
            timestamp: 100,
        };
        
        assert!(transition.is_escalation());
        assert!(!transition.is_deescalation());
    }
    
    #[test]
    fn test_deescalation_detection() {
        let transition = ModeTransition {
            from: AutonomicMode::Emergency,
            to: AutonomicMode::Recovery,
            trigger: TransitionTrigger::RecoveryComplete,
            timestamp: 200,
        };
        
        assert!(transition.is_deescalation());
        assert!(!transition.is_escalation());
    }
}
