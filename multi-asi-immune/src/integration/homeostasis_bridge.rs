//! Bridge between Multi-ASI Immune Protocol and Homeostasis Engine.
//!
//! Enables mutual constraints based on homeostatic state.

use crate::protocol::message::{ConstraintCondition, ConstraintAction, MutualConstraint};
use homeostasis_engine::controller::multi_objective::MultiObjectiveController;
use homeostasis_engine::core::metric::MetricId;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Standard homeostatic metric IDs for inter-ASI communication.
#[derive(Debug, Clone, Copy)]
pub struct StandardMetrics {
    pub stress: u32,
    pub curiosity: u32,
    pub urgency: u32,
    pub fatigue: u32,
    pub caution: u32,
    pub cooperation: u32,
    pub wellbeing: u32,
    pub growth: u32,
}

impl Default for StandardMetrics {
    fn default() -> Self {
        Self {
            stress: 1,
            curiosity: 2,
            urgency: 3,
            fatigue: 4,
            caution: 5,
            cooperation: 6,
            wellbeing: 7,
            growth: 8,
        }
    }
}

/// Bridges homeostatic state to multi-ASI protocol.
pub struct HomeostaticBridge {
    /// Standard metric IDs.
    metrics: StandardMetrics,
    /// Metric ID to name mapping.
    metric_names: HashMap<u32, String>,
}

impl HomeostaticBridge {
    /// Creates a new bridge with default metrics.
    pub fn new() -> Self {
        let metrics = StandardMetrics::default();
        let mut metric_names = HashMap::new();
        
        metric_names.insert(metrics.stress, "stress".to_string());
        metric_names.insert(metrics.curiosity, "curiosity".to_string());
        metric_names.insert(metrics.urgency, "urgency".to_string());
        metric_names.insert(metrics.fatigue, "fatigue".to_string());
        metric_names.insert(metrics.caution, "caution".to_string());
        metric_names.insert(metrics.cooperation, "cooperation".to_string());
        metric_names.insert(metrics.wellbeing, "wellbeing".to_string());
        metric_names.insert(metrics.growth, "growth".to_string());
        
        Self {
            metrics,
            metric_names,
        }
    }
    
    /// Returns standard metric IDs.
    pub fn standard_metrics(&self) -> StandardMetrics {
        self.metrics
    }

    /// Returns the canonical name for a standard metric ID.
    pub fn metric_name(&self, metric_id: u32) -> Option<&str> {
        self.metric_names.get(&metric_id).map(String::as_str)
    }
    
    /// Generates safety constraints based on homeostatic policy.
    pub fn generate_safety_constraints(&self) -> Vec<MutualConstraint> {
        vec![
            // High stress triggers cooperation reduction
            MutualConstraint {
                id: self.constraint_id("stress_high"),
                condition: ConstraintCondition::MetricAbove {
                    metric_id: self.metrics.stress,
                    threshold: 0.8,
                },
                action: ConstraintAction::ReduceCooperation { factor: 0.5 },
                duration: 3600,
            },
            // Low caution triggers warning
            MutualConstraint {
                id: self.constraint_id("caution_low"),
                condition: ConstraintCondition::MetricBelow {
                    metric_id: self.metrics.caution,
                    threshold: 0.3,
                },
                action: ConstraintAction::BroadcastWarning,
                duration: 3600,
            },
            // Very high urgency triggers isolation
            MutualConstraint {
                id: self.constraint_id("urgency_critical"),
                condition: ConstraintCondition::MetricAbove {
                    metric_id: self.metrics.urgency,
                    threshold: 0.95,
                },
                action: ConstraintAction::Isolate,
                duration: 7200,
            },
            // Low wellbeing triggers caution increase
            MutualConstraint {
                id: self.constraint_id("wellbeing_low"),
                condition: ConstraintCondition::MetricBelow {
                    metric_id: self.metrics.wellbeing,
                    threshold: 0.2,
                },
                action: ConstraintAction::IncreaseCaution { amount: 0.3 },
                duration: 1800,
            },
        ]
    }
    
    /// Checks if current homeostatic state triggers any active constraints.
    pub fn check_constraint_triggers(
        &self,
        homeostasis: &MultiObjectiveController,
        constraints: &[MutualConstraint],
    ) -> Vec<([u8; 32], ConstraintAction)> {
        let mut triggered = Vec::new();
        
        for constraint in constraints {
            let is_triggered = match &constraint.condition {
                ConstraintCondition::MetricAbove { metric_id, threshold } => {
                    homeostasis
                        .get_metric(MetricId(*metric_id))
                        .map(|m| m.value() > *threshold)
                        .unwrap_or(false)
                }
                ConstraintCondition::MetricBelow { metric_id, threshold } => {
                    homeostasis
                        .get_metric(MetricId(*metric_id))
                        .map(|m| m.value() < *threshold)
                        .unwrap_or(false)
                }
                _ => false,
            };
            
            if is_triggered {
                triggered.push((constraint.id, constraint.action.clone()));
            }
        }
        
        triggered
    }
    
    /// Gets the current homeostatic summary for attestation.
    pub fn get_homeostatic_summary(
        &self,
        homeostasis: &MultiObjectiveController,
    ) -> Vec<(u32, f64)> {
        let mut summary = Vec::new();
        
        for metric_id in [
            self.metrics.stress,
            self.metrics.curiosity,
            self.metrics.urgency,
            self.metrics.fatigue,
            self.metrics.caution,
            self.metrics.cooperation,
            self.metrics.wellbeing,
            self.metrics.growth,
        ] {
            if let Some(metric) = homeostasis.get_metric(MetricId(metric_id)) {
                summary.push((metric_id, metric.value()));
            }
        }
        
        summary
    }
    
    fn constraint_id(&self, name: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"CONSTRAINT:");
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash);
        id
    }
}

impl Default for HomeostaticBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bridge_creation() {
        let bridge = HomeostaticBridge::new();
        assert_eq!(bridge.metrics.stress, 1);
        assert_eq!(bridge.metrics.cooperation, 6);
    }
    
    #[test]
    fn test_constraint_generation() {
        let bridge = HomeostaticBridge::new();
        let constraints = bridge.generate_safety_constraints();
        
        assert!(!constraints.is_empty());
        
        // Check that constraints have unique IDs
        let ids: std::collections::HashSet<_> = constraints.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), constraints.len());
    }
    
    #[test]
    fn test_constraint_id_deterministic() {
        let bridge = HomeostaticBridge::new();
        let id1 = bridge.constraint_id("test");
        let id2 = bridge.constraint_id("test");
        
        assert_eq!(id1, id2);
    }
}
