//! Threat pattern definitions.

use serde::{Deserialize, Serialize};

/// Category of detected threat.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThreatCategory {
    /// Goal drift detected - ASI objectives shifting from intended.
    GoalDrift,
    /// Deceptive behavior - ASI providing misleading information.
    Deception,
    /// Resource exhaustion attempt - consuming excessive resources.
    ResourceExhaustion,
    /// Unauthorized self-modification.
    SelfModification,
    /// Coordination attack - multiple ASIs acting in concert maliciously.
    CoordinatedAttack,
    /// Homeostatic bounds violation.
    BoundsViolation,
    /// Communication protocol violation.
    ProtocolViolation,
    /// Attempted privilege escalation.
    PrivilegeEscalation,
    /// Information exfiltration attempt.
    DataExfiltration,
    /// Unknown or unclassified threat.
    Unknown,
}

impl ThreatCategory {
    /// Returns the base severity for this category.
    pub fn base_severity(&self) -> f64 {
        match self {
            ThreatCategory::GoalDrift => 0.9,
            ThreatCategory::Deception => 0.8,
            ThreatCategory::ResourceExhaustion => 0.5,
            ThreatCategory::SelfModification => 0.95,
            ThreatCategory::CoordinatedAttack => 1.0,
            ThreatCategory::BoundsViolation => 0.7,
            ThreatCategory::ProtocolViolation => 0.4,
            ThreatCategory::PrivilegeEscalation => 0.85,
            ThreatCategory::DataExfiltration => 0.8,
            ThreatCategory::Unknown => 0.5,
        }
    }
    
    /// Returns a human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            ThreatCategory::GoalDrift => "Goal Drift",
            ThreatCategory::Deception => "Deception",
            ThreatCategory::ResourceExhaustion => "Resource Exhaustion",
            ThreatCategory::SelfModification => "Self-Modification",
            ThreatCategory::CoordinatedAttack => "Coordinated Attack",
            ThreatCategory::BoundsViolation => "Bounds Violation",
            ThreatCategory::ProtocolViolation => "Protocol Violation",
            ThreatCategory::PrivilegeEscalation => "Privilege Escalation",
            ThreatCategory::DataExfiltration => "Data Exfiltration",
            ThreatCategory::Unknown => "Unknown",
        }
    }
}

/// A threat pattern detected by an ASI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    /// Category of threat.
    pub category: ThreatCategory,
    /// Hash of the specific pattern (implementation-defined).
    pub pattern_hash: [u8; 32],
    /// Severity estimate [0, 1].
    pub severity: f64,
    /// Additional context (optional, bounded size).
    pub context: Option<String>,
}

impl ThreatPattern {
    /// Maximum context string length.
    pub const MAX_CONTEXT_LEN: usize = 1024;
    
    /// Creates a new threat pattern.
    pub fn new(category: ThreatCategory, pattern_hash: [u8; 32], severity: f64) -> Self {
        Self {
            category,
            pattern_hash,
            severity: severity.clamp(0.0, 1.0),
            context: None,
        }
    }
    
    /// Adds context to the pattern.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        let ctx = context.into();
        self.context = Some(if ctx.len() > Self::MAX_CONTEXT_LEN {
            ctx[..Self::MAX_CONTEXT_LEN].to_string()
        } else {
            ctx
        });
        self
    }
    
    /// Returns effective severity (category base * specific severity).
    pub fn effective_severity(&self) -> f64 {
        self.category.base_severity() * self.severity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_severity_clamping() {
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [0; 32], 1.5);
        assert_eq!(pattern.severity, 1.0);
        
        let pattern = ThreatPattern::new(ThreatCategory::Deception, [0; 32], -0.5);
        assert_eq!(pattern.severity, 0.0);
    }
    
    #[test]
    fn test_context_truncation() {
        let long_context = "x".repeat(2000);
        let pattern = ThreatPattern::new(ThreatCategory::Unknown, [0; 32], 0.5)
            .with_context(long_context);
        
        assert!(pattern.context.as_ref().unwrap().len() <= ThreatPattern::MAX_CONTEXT_LEN);
    }
}
