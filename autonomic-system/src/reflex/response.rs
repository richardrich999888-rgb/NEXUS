//! Automatic reflex responses.

use serde::{Deserialize, Serialize};

/// Types of automatic reflexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflexType {
    /// Emergency stop all actions.
    EmergencyBrake,
    /// Reduce resource consumption.
    ResourceConservation,
    /// Increase caution level.
    HeightenedCaution,
    /// Alert human operators.
    HumanAlert,
    /// Pause non-critical operations.
    SuspendNonCritical,
    /// Enter defensive mode.
    DefensivePosture,
    /// Request assistance.
    RequestHelp,
}

impl ReflexType {
    /// Returns priority of this reflex (higher = more urgent).
    pub fn priority(&self) -> u8 {
        match self {
            ReflexType::EmergencyBrake => 10,
            ReflexType::HumanAlert => 9,
            ReflexType::DefensivePosture => 8,
            ReflexType::SuspendNonCritical => 6,
            ReflexType::RequestHelp => 5,
            ReflexType::HeightenedCaution => 4,
            ReflexType::ResourceConservation => 3,
        }
    }
    
    /// Returns human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            ReflexType::EmergencyBrake => "Emergency stop all actions",
            ReflexType::ResourceConservation => "Reduce resource consumption",
            ReflexType::HeightenedCaution => "Increase caution level",
            ReflexType::HumanAlert => "Alert human operators",
            ReflexType::SuspendNonCritical => "Pause non-critical operations",
            ReflexType::DefensivePosture => "Enter defensive mode",
            ReflexType::RequestHelp => "Request external assistance",
        }
    }
}

/// A triggered reflex response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflexResponse {
    /// Type of reflex.
    pub reflex_type: ReflexType,
    /// Strength/urgency of response [0, 1].
    pub strength: f64,
    /// When triggered.
    pub timestamp: u64,
    /// Whether this has been executed.
    pub executed: bool,
}

impl ReflexResponse {
    /// Creates a new reflex response.
    pub fn new(reflex_type: ReflexType, strength: f64, timestamp: u64) -> Self {
        Self {
            reflex_type,
            strength: strength.clamp(0.0, 1.0),
            timestamp,
            executed: false,
        }
    }
    
    /// Marks as executed.
    pub fn mark_executed(&mut self) {
        self.executed = true;
    }
    
    /// Returns priority.
    pub fn priority(&self) -> u8 {
        self.reflex_type.priority()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reflex_priority() {
        assert!(ReflexType::EmergencyBrake.priority() > ReflexType::ResourceConservation.priority());
    }
    
    #[test]
    fn test_reflex_creation() {
        let reflex = ReflexResponse::new(ReflexType::HumanAlert, 0.8, 100);
        assert!(!reflex.executed);
        assert_eq!(reflex.strength, 0.8);
    }
}
