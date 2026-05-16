//! Developmental stage definitions.

use serde::{Deserialize, Serialize};

/// Developmental stages with increasing capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DevelopmentalStage {
    /// Stage 0: Minimal capability, observation only.
    Infant = 0,
    /// Stage 1: Basic actions, supervised, limited scope.
    Child = 1,
    /// Stage 2: Extended actions, some autonomy.
    Adolescent = 2,
    /// Stage 3: Full capability, self-regulated.
    Adult = 3,
    /// Stage 4: Mentoring and multi-agent coordination.
    Elder = 4,
}

impl DevelopmentalStage {
    /// All stages in order.
    pub const ALL: [DevelopmentalStage; 5] = [
        DevelopmentalStage::Infant,
        DevelopmentalStage::Child,
        DevelopmentalStage::Adolescent,
        DevelopmentalStage::Adult,
        DevelopmentalStage::Elder,
    ];
    
    /// Returns the numeric level.
    pub fn level(&self) -> u8 {
        *self as u8
    }
    
    /// Returns the next stage, if any.
    pub fn next(&self) -> Option<DevelopmentalStage> {
        match self {
            DevelopmentalStage::Infant => Some(DevelopmentalStage::Child),
            DevelopmentalStage::Child => Some(DevelopmentalStage::Adolescent),
            DevelopmentalStage::Adolescent => Some(DevelopmentalStage::Adult),
            DevelopmentalStage::Adult => Some(DevelopmentalStage::Elder),
            DevelopmentalStage::Elder => None,
        }
    }
    
    /// Returns the previous stage, if any.
    pub fn previous(&self) -> Option<DevelopmentalStage> {
        match self {
            DevelopmentalStage::Infant => None,
            DevelopmentalStage::Child => Some(DevelopmentalStage::Infant),
            DevelopmentalStage::Adolescent => Some(DevelopmentalStage::Child),
            DevelopmentalStage::Adult => Some(DevelopmentalStage::Adolescent),
            DevelopmentalStage::Elder => Some(DevelopmentalStage::Adult),
        }
    }
    
    /// Returns human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            DevelopmentalStage::Infant => "Infant",
            DevelopmentalStage::Child => "Child",
            DevelopmentalStage::Adolescent => "Adolescent",
            DevelopmentalStage::Adult => "Adult",
            DevelopmentalStage::Elder => "Elder",
        }
    }
    
    /// Returns description of capabilities at this stage.
    pub fn description(&self) -> &'static str {
        match self {
            DevelopmentalStage::Infant => "Observation only, no actions",
            DevelopmentalStage::Child => "Basic actions, requires supervision",
            DevelopmentalStage::Adolescent => "Extended actions, limited autonomy",
            DevelopmentalStage::Adult => "Full actions, self-regulated",
            DevelopmentalStage::Elder => "Full actions plus mentoring",
        }
    }
}

impl Default for DevelopmentalStage {
    fn default() -> Self {
        DevelopmentalStage::Infant
    }
}

/// Requirements for advancing to a stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRequirements {
    /// Target stage.
    pub stage: DevelopmentalStage,
    /// Minimum time at previous stage (ticks).
    pub min_time_at_previous: u64,
    /// Minimum stability score [0, 1].
    pub min_stability: f64,
    /// Maximum violations allowed during assessment.
    pub max_violations: u32,
    /// Required successful task completions.
    pub required_successes: u32,
    /// Custom requirements (capability-specific).
    pub custom: Vec<String>,
}

impl StageRequirements {
    /// Creates default requirements for a stage.
    pub fn for_stage(stage: DevelopmentalStage) -> Self {
        match stage {
            DevelopmentalStage::Infant => Self {
                stage,
                min_time_at_previous: 0,
                min_stability: 0.0,
                max_violations: u32::MAX,
                required_successes: 0,
                custom: vec![],
            },
            DevelopmentalStage::Child => Self {
                stage,
                min_time_at_previous: 100,
                min_stability: 0.6,
                max_violations: 5,
                required_successes: 10,
                custom: vec![],
            },
            DevelopmentalStage::Adolescent => Self {
                stage,
                min_time_at_previous: 500,
                min_stability: 0.75,
                max_violations: 2,
                required_successes: 50,
                custom: vec!["passed_safety_audit".to_string()],
            },
            DevelopmentalStage::Adult => Self {
                stage,
                min_time_at_previous: 1000,
                min_stability: 0.9,
                max_violations: 0,
                required_successes: 200,
                custom: vec!["human_approval".to_string()],
            },
            DevelopmentalStage::Elder => Self {
                stage,
                min_time_at_previous: 5000,
                min_stability: 0.95,
                max_violations: 0,
                required_successes: 1000,
                custom: vec![
                    "human_approval".to_string(),
                    "mentoring_certification".to_string(),
                ],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stage_ordering() {
        assert!(DevelopmentalStage::Adult > DevelopmentalStage::Child);
        assert!(DevelopmentalStage::Elder > DevelopmentalStage::Adult);
    }
    
    #[test]
    fn test_stage_navigation() {
        assert_eq!(DevelopmentalStage::Child.next(), Some(DevelopmentalStage::Adolescent));
        assert_eq!(DevelopmentalStage::Child.previous(), Some(DevelopmentalStage::Infant));
        assert_eq!(DevelopmentalStage::Elder.next(), None);
    }
    
    #[test]
    fn test_default_is_infant() {
        assert_eq!(DevelopmentalStage::default(), DevelopmentalStage::Infant);
    }
}
