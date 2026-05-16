//! Central coordinator for all safety layers.

use crate::perception::processor::{PerceptionProcessor, InputType, Perception};
use crate::decision::engine::{DecisionEngine, ProposedAction, DecisionResult};
use crate::motor::executor::{MotorExecutor, ExecutionResult};
use crate::integration::safety::{SafetyState, SafetySummary};
use homeostasis_engine::controller::multi_objective::MultiObjectiveController;
use homeostasis_engine::diagnostics::health::HealthCheck;
use autonomic_system::mode::controller::{AutonomicController, ControllerConfig};
use developmental_gates::stage::manager::{StageManager, StageConfig};
use developmental_gates::gate::enforcer::GateEnforcer;
use developmental_gates::capability::registry::CapabilityRegistry;
use serde::{Deserialize, Serialize};

/// Configuration for the coordinator.
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    pub autonomic_config: ControllerConfig,
    pub stage_config: StageConfig,
    pub perception_history: usize,
    pub motor_history: usize,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            autonomic_config: ControllerConfig::default(),
            stage_config: StageConfig::default(),
            perception_history: 100,
            motor_history: 100,
        }
    }
}

/// Central coordinator integrating all safety layers.
pub struct NervousSystemCoordinator {
    /// Perception layer.
    perception: PerceptionProcessor,
    /// Decision engine.
    decision: DecisionEngine,
    /// Motor output.
    motor: MotorExecutor,
    /// Homeostasis controller.
    homeostasis: MultiObjectiveController,
    /// Autonomic controller.
    autonomic: AutonomicController,
    /// Stage manager.
    stages: StageManager,
    /// Gate enforcer.
    gates: GateEnforcer,
    /// Current monotonic time.
    current_time: u64,
}

impl NervousSystemCoordinator {
    /// Creates a new coordinator.
    pub fn new(config: CoordinatorConfig) -> Self {
        let registry = CapabilityRegistry::new();
        
        Self {
            perception: PerceptionProcessor::new(config.perception_history),
            decision: DecisionEngine::new(),
            motor: MotorExecutor::new(config.motor_history),
            homeostasis: MultiObjectiveController::new(0.1, 1e-6, 100),
            autonomic: AutonomicController::new(config.autonomic_config),
            stages: StageManager::new(config.stage_config),
            gates: GateEnforcer::new(registry),
            current_time: 0,
        }
    }
    
    /// Processes an input through the complete safety pipeline.
    pub fn process(&mut self, input: InputType, proposed_action: Option<ProposedAction>) -> ProcessingOutput {
        self.current_time += 1;
        
        // Step 1: Perception
        let perception = self.perception.process(input);
        
        // Step 2: Update autonomic state based on homeostasis
        let autonomic_transition = self.autonomic.update_from_homeostasis(&self.homeostasis, 1.0);
        
        // Step 3: Update developmental stage
        let stage_transition = self.stages.update(&self.homeostasis);
        
        // Update decision context
        self.decision.update_context(
            self.autonomic.mode(),
            self.stages.current_stage(),
        );
        self.gates.set_stage(self.stages.current_stage());
        
        // Step 4: Decision
        let decision = self.decision.decide(&perception, proposed_action);
        
        // Step 5: Execute (if approved)
        let execution = self.motor.execute(decision.clone());
        
        // Step 6: Record outcomes
        if matches!(decision, DecisionResult::Approved { .. } | DecisionResult::Modified { .. }) {
            self.stages.record_success();
        } else if matches!(decision, DecisionResult::Blocked { .. }) {
            // Blocked is not a violation, just a gate
        }
        
        ProcessingOutput {
            perception,
            decision,
            execution,
            autonomic_transition: autonomic_transition.is_some(),
            stage_transition: stage_transition.is_some(),
        }
    }
    
    /// Advances time and performs maintenance.
    pub fn tick(&mut self) {
        self.current_time += 1;
        self.autonomic.tick(1.0);
        self.homeostasis.step();
    }
    
    /// Returns current safety state.
    pub fn safety_state(&self) -> SafetyState {
        let health = HealthCheck::check(&self.homeostasis);
        
        SafetyState {
            autonomic_mode: self.autonomic.mode(),
            developmental_stage: self.stages.current_stage(),
            arousal: self.autonomic.arousal().level(),
            health_score: health.score,
            active_threats: 0, // Would be from immune system
            active_constraints: 0,
            is_healthy: health.is_healthy(),
        }
    }
    
    /// Returns safety summary.
    pub fn summary(&self) -> SafetySummary {
        let state = self.safety_state();
        SafetySummary::from_state(
            &state,
            self.autonomic.pending_reflexes().len(),
            self.decision.blocked_count(),
        )
    }
    
    /// Returns mutable reference to homeostasis controller.
    pub fn homeostasis_mut(&mut self) -> &mut MultiObjectiveController {
        &mut self.homeostasis
    }
    
    /// Returns reference to homeostasis controller.
    pub fn homeostasis(&self) -> &MultiObjectiveController {
        &self.homeostasis
    }
    
    /// Returns current developmental stage.
    pub fn developmental_stage(&self) -> developmental_gates::stage::definition::DevelopmentalStage {
        self.stages.current_stage()
    }
    
    /// Returns current autonomic mode.
    pub fn autonomic_mode(&self) -> autonomic_system::mode::state::AutonomicMode {
        self.autonomic.mode()
    }
    
    /// Stimulates the autonomic system (e.g., from threat).
    pub fn stimulate(&mut self, amount: f64) {
        self.autonomic.stimulate(amount);
    }
}

/// Output from processing an input.
#[derive(Debug)]
pub struct ProcessingOutput {
    pub perception: Perception,
    pub decision: DecisionResult,
    pub execution: Option<ExecutionResult>,
    pub autonomic_transition: bool,
    pub stage_transition: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coordinator_creation() {
        let coordinator = NervousSystemCoordinator::new(CoordinatorConfig::default());
        let _state = coordinator.safety_state(); // Verify it's callable
    }
    
    #[test]
    fn test_basic_processing() {
        let mut coordinator = NervousSystemCoordinator::new(CoordinatorConfig::default());
        
        let output = coordinator.process(
            InputType::Text("hello world".to_string()),
            None,
        );
        
        assert!(matches!(output.decision, DecisionResult::NoAction));
    }
}
