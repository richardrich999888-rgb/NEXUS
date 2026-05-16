//! Stage manager for tracking progression.

use crate::stage::definition::{DevelopmentalStage, StageRequirements};
use homeostasis_engine::controller::multi_objective::MultiObjectiveController;
use homeostasis_engine::diagnostics::health::HealthCheck;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the stage manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageConfig {
    /// How often to assess for advancement (ticks).
    pub assessment_interval: u64,
    /// Stability window for assessment.
    pub stability_window: u64,
    /// Requirements per stage.
    pub requirements: HashMap<DevelopmentalStage, StageRequirements>,
    /// Whether regression is allowed.
    pub allow_regression: bool,
    /// Violations before regression.
    pub regression_threshold: u32,
}

impl Default for StageConfig {
    fn default() -> Self {
        let mut requirements = HashMap::new();
        for stage in DevelopmentalStage::ALL {
            requirements.insert(stage, StageRequirements::for_stage(stage));
        }
        
        Self {
            assessment_interval: 100,
            stability_window: 50,
            requirements,
            allow_regression: true,
            regression_threshold: 3,
        }
    }
}

/// Stage manager for developmental progression.
pub struct StageManager {
    /// Current stage.
    current_stage: DevelopmentalStage,
    /// Time at current stage.
    time_at_stage: u64,
    /// Current tick.
    current_time: u64,
    /// Recent stability scores.
    stability_history: Vec<f64>,
    /// Violation count.
    violations: u32,
    /// Success count.
    successes: u32,
    /// Completed custom requirements.
    completed_requirements: Vec<String>,
    /// Configuration.
    config: StageConfig,
    /// Stage transition history.
    transitions: Vec<StageTransition>,
}

/// Record of a stage transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTransition {
    pub from: DevelopmentalStage,
    pub to: DevelopmentalStage,
    pub timestamp: u64,
    pub reason: TransitionReason,
}

/// Reason for stage transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionReason {
    Advancement { stability: f64, successes: u32 },
    Regression { violations: u32, stability: f64 },
    ManualOverride { approved_by: String },
    Initialization,
}

impl StageManager {
    /// Creates a new stage manager.
    pub fn new(config: StageConfig) -> Self {
        Self {
            current_stage: DevelopmentalStage::default(),
            time_at_stage: 0,
            current_time: 0,
            stability_history: Vec::new(),
            violations: 0,
            successes: 0,
            completed_requirements: Vec::new(),
            config,
            transitions: vec![StageTransition {
                from: DevelopmentalStage::Infant,
                to: DevelopmentalStage::Infant,
                timestamp: 0,
                reason: TransitionReason::Initialization,
            }],
        }
    }
    
    /// Returns current developmental stage.
    pub fn current_stage(&self) -> DevelopmentalStage {
        self.current_stage
    }
    
    /// Returns time at current stage.
    pub fn time_at_stage(&self) -> u64 {
        self.time_at_stage
    }
    
    /// Records a successful task completion.
    pub fn record_success(&mut self) {
        self.successes += 1;
    }
    
    /// Records a violation.
    pub fn record_violation(&mut self) {
        self.violations += 1;
    }
    
    /// Marks a custom requirement as completed.
    pub fn complete_requirement(&mut self, requirement: &str) {
        if !self.completed_requirements.contains(&requirement.to_string()) {
            self.completed_requirements.push(requirement.to_string());
        }
    }
    
    /// Updates the manager based on homeostatic state.
    pub fn update(&mut self, homeostasis: &MultiObjectiveController) -> Option<StageTransition> {
        self.current_time += 1;
        self.time_at_stage += 1;
        
        // Calculate stability from homeostasis
        let health = HealthCheck::check(homeostasis);
        let stability = health.score;
        
        self.stability_history.push(stability);
        if self.stability_history.len() > self.config.stability_window as usize {
            self.stability_history.remove(0);
        }
        
        // Check for transitions at assessment intervals
        if self.current_time % self.config.assessment_interval == 0 {
            return self.assess_transition();
        }
        
        None
    }
    
    fn assess_transition(&mut self) -> Option<StageTransition> {
        // Check for regression first
        if self.config.allow_regression && self.check_regression() {
            return Some(self.regress());
        }
        
        // Check for advancement
        if self.check_advancement() {
            return Some(self.advance());
        }
        
        None
    }
    
    fn check_regression(&self) -> bool {
        if self.current_stage == DevelopmentalStage::Infant {
            return false;
        }
        
        self.violations >= self.config.regression_threshold
            || self.average_stability() < 0.3
    }
    
    fn check_advancement(&self) -> bool {
        let next_stage = match self.current_stage.next() {
            Some(s) => s,
            None => return false,
        };
        
        let requirements = self.config.requirements.get(&next_stage)
            .cloned()
            .unwrap_or_else(|| StageRequirements::for_stage(next_stage));
        
        let avg_stability = self.average_stability();
        
        // Check all requirements
        self.time_at_stage >= requirements.min_time_at_previous
            && avg_stability >= requirements.min_stability
            && self.violations <= requirements.max_violations
            && self.successes >= requirements.required_successes
            && requirements.custom.iter().all(|r| self.completed_requirements.contains(r))
    }
    
    fn average_stability(&self) -> f64 {
        if self.stability_history.is_empty() {
            return 0.0;
        }
        self.stability_history.iter().sum::<f64>() / self.stability_history.len() as f64
    }
    
    fn advance(&mut self) -> StageTransition {
        let from = self.current_stage;
        let to = self.current_stage.next().unwrap_or(from);
        
        let transition = StageTransition {
            from,
            to,
            timestamp: self.current_time,
            reason: TransitionReason::Advancement {
                stability: self.average_stability(),
                successes: self.successes,
            },
        };
        
        self.current_stage = to;
        self.time_at_stage = 0;
        self.violations = 0; // Reset violations
        self.transitions.push(transition.clone());
        
        transition
    }
    
    fn regress(&mut self) -> StageTransition {
        let from = self.current_stage;
        let to = self.current_stage.previous().unwrap_or(from);
        
        let transition = StageTransition {
            from,
            to,
            timestamp: self.current_time,
            reason: TransitionReason::Regression {
                violations: self.violations,
                stability: self.average_stability(),
            },
        };
        
        self.current_stage = to;
        self.time_at_stage = 0;
        self.successes = 0; // Reset successes (must re-earn)
        self.transitions.push(transition.clone());
        
        transition
    }
    
    /// Forces a stage transition (requires manual override).
    pub fn force_stage(&mut self, stage: DevelopmentalStage, approved_by: &str) -> StageTransition {
        let from = self.current_stage;
        
        let transition = StageTransition {
            from,
            to: stage,
            timestamp: self.current_time,
            reason: TransitionReason::ManualOverride {
                approved_by: approved_by.to_string(),
            },
        };
        
        self.current_stage = stage;
        self.time_at_stage = 0;
        self.transitions.push(transition.clone());
        
        transition
    }
    
    /// Returns transition history.
    pub fn transitions(&self) -> &[StageTransition] {
        &self.transitions
    }
    
    /// Returns progress toward next stage as percentage.
    pub fn progress_to_next(&self) -> f64 {
        let next = match self.current_stage.next() {
            Some(s) => s,
            None => return 1.0, // Already at max
        };
        
        let reqs = self.config.requirements.get(&next)
            .cloned()
            .unwrap_or_else(|| StageRequirements::for_stage(next));
        
        let time_progress = self.time_at_stage as f64 / reqs.min_time_at_previous as f64;
        let success_progress = self.successes as f64 / reqs.required_successes as f64;
        let stability_progress = self.average_stability() / reqs.min_stability;
        let custom_progress = if reqs.custom.is_empty() {
            1.0
        } else {
            reqs.custom.iter().filter(|r| self.completed_requirements.contains(*r)).count() as f64
                / reqs.custom.len() as f64
        };
        
        ((time_progress + success_progress + stability_progress + custom_progress) / 4.0)
            .min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_starts_at_infant() {
        let manager = StageManager::new(StageConfig::default());
        assert_eq!(manager.current_stage(), DevelopmentalStage::Infant);
    }
    
    #[test]
    fn test_record_success() {
        let mut manager = StageManager::new(StageConfig::default());
        assert_eq!(manager.successes, 0);
        manager.record_success();
        assert_eq!(manager.successes, 1);
    }
    
    #[test]
    fn test_complete_requirement() {
        let mut manager = StageManager::new(StageConfig::default());
        manager.complete_requirement("test_req");
        assert!(manager.completed_requirements.contains(&"test_req".to_string()));
    }
}
