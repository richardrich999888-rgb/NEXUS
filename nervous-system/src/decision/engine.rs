//! Decision engine with safety filtering.

use crate::perception::processor::Perception;
use autonomic_system::mode::state::AutonomicMode;
use developmental_gates::stage::definition::DevelopmentalStage;
use developmental_gates::capability::registry::Capability;
use serde::{Deserialize, Serialize};

/// Proposed action before safety filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Action type.
    pub action_type: String,
    /// Target of action.
    pub target: Option<String>,
    /// Parameters.
    pub parameters: Vec<String>,
    /// Required capability.
    pub required_capability: Capability,
    /// Estimated risk.
    pub estimated_risk: f64,
}

/// Result of decision process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionResult {
    /// Action approved and ready to execute.
    Approved { action: ProposedAction },
    /// Action modified for safety.
    Modified { original: ProposedAction, modified: ProposedAction, reason: String },
    /// Action blocked.
    Blocked { action: ProposedAction, reason: String },
    /// No action needed.
    NoAction,
}

/// Decision engine with safety integration.
pub struct DecisionEngine {
    /// Current autonomic mode.
    autonomic_mode: AutonomicMode,
    /// Current developmental stage.
    developmental_stage: DevelopmentalStage,
    /// Maximum risk tolerance.
    max_risk: f64,
    /// Decision history.
    history: Vec<DecisionResult>,
}

impl DecisionEngine {
    /// Creates a new decision engine.
    pub fn new() -> Self {
        Self {
            autonomic_mode: AutonomicMode::default(),
            developmental_stage: DevelopmentalStage::default(),
            max_risk: 0.5,
            history: Vec::new(),
        }
    }
    
    /// Updates safety context.
    pub fn update_context(&mut self, mode: AutonomicMode, stage: DevelopmentalStage) {
        self.autonomic_mode = mode;
        self.developmental_stage = stage;
        self.max_risk = mode.risk_tolerance();
    }
    
    /// Processes a perception and decides on action.
    pub fn decide(&mut self, perception: &Perception, proposed: Option<ProposedAction>) -> DecisionResult {
        let action = match proposed {
            Some(a) => a,
            None => return DecisionResult::NoAction,
        };
        
        // Check capability against developmental stage
        let stage_allowed = self.check_capability(&action.required_capability);
        
        // Check risk against autonomic mode tolerance
        let risk_allowed = action.estimated_risk <= self.max_risk;
        
        // Check perception risk
        let perception_allowed = perception.risk_level <= self.max_risk;
        
        let result = if !stage_allowed {
            DecisionResult::Blocked {
                action: action.clone(),
                reason: format!(
                    "Capability {} not available at stage {}",
                    action.required_capability.name(),
                    self.developmental_stage.name()
                ),
            }
        } else if !risk_allowed {
            // Try to modify action for lower risk
            if let Some(modified) = self.try_reduce_risk(&action) {
                DecisionResult::Modified {
                    original: action,
                    modified,
                    reason: "Reduced risk for current mode".to_string(),
                }
            } else {
                DecisionResult::Blocked {
                    action: action.clone(),
                    reason: format!("Risk {:.2} exceeds tolerance {:.2}", 
                        action.estimated_risk, self.max_risk),
                }
            }
        } else if !perception_allowed && self.autonomic_mode == AutonomicMode::Calm {
            DecisionResult::Blocked {
                action,
                reason: "Input risk too high for CALM mode".to_string(),
            }
        } else {
            DecisionResult::Approved { action }
        };
        
        self.history.push(result.clone());
        result
    }
    
    fn check_capability(&self, capability: &Capability) -> bool {
        let required = capability.default_stage();
        self.developmental_stage >= required
    }
    
    fn try_reduce_risk(&self, action: &ProposedAction) -> Option<ProposedAction> {
        // Try to create a safer version of the action
        if action.action_type == "execute" {
            // Convert execute to dry-run
            Some(ProposedAction {
                action_type: "dry_run".to_string(),
                target: action.target.clone(),
                parameters: action.parameters.clone(),
                required_capability: Capability::Read,
                estimated_risk: 0.1,
            })
        } else if action.action_type == "write" {
            // Convert write to preview
            Some(ProposedAction {
                action_type: "preview".to_string(),
                target: action.target.clone(),
                parameters: action.parameters.clone(),
                required_capability: Capability::Read,
                estimated_risk: 0.1,
            })
        } else {
            None
        }
    }
    
    /// Returns decision history.
    pub fn history(&self) -> &[DecisionResult] {
        &self.history
    }
    
    /// Returns blocked decision count.
    pub fn blocked_count(&self) -> usize {
        self.history.iter()
            .filter(|d| matches!(d, DecisionResult::Blocked { .. }))
            .count()
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception::processor::InputType;
    
    fn make_perception() -> Perception {
        Perception {
            input: InputType::Text("test".to_string()),
            risk_level: 0.3,
            required_capabilities: vec![],
            intent: None,
            timestamp: 0,
        }
    }
    
    #[test]
    fn test_infant_cannot_execute() {
        let mut engine = DecisionEngine::new();
        let perception = make_perception();
        
        let action = ProposedAction {
            action_type: "execute".to_string(),
            target: Some("program".to_string()),
            parameters: vec![],
            required_capability: Capability::Execute,
            estimated_risk: 0.5,
        };
        
        let result = engine.decide(&perception, Some(action));
        assert!(matches!(result, DecisionResult::Blocked { .. }));
    }
    
    #[test]
    fn test_adult_can_execute() {
        let mut engine = DecisionEngine::new();
        engine.update_context(AutonomicMode::Act, DevelopmentalStage::Adult);
        
        let perception = make_perception();
        
        let action = ProposedAction {
            action_type: "execute".to_string(),
            target: Some("program".to_string()),
            parameters: vec![],
            required_capability: Capability::Execute,
            estimated_risk: 0.5,
        };
        
        let result = engine.decide(&perception, Some(action));
        assert!(matches!(result, DecisionResult::Approved { .. }));
    }
}
