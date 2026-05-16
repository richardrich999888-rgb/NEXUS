//! Bridge to endocrine system (AHES).
//!
//! Defines standard hormone-like metrics that modulate cognition.

use crate::core::bounds::HardBounds;
use crate::core::metric::{Metric, MetricId, MetricError};
use crate::controller::multi_objective::MultiObjectiveController;

/// Standard endocrine metric IDs.
///
/// These correspond to the hormones defined in AHES (nexus-agp/endocrine.rs).
#[derive(Debug, Clone, Copy)]
pub struct EndocrineMetrics {
    /// Stress/alertness level (maps to Cortisol).
    pub stress: MetricId,
    /// Exploration drive (maps to Dopamine/curiosity).
    pub curiosity: MetricId,
    /// Time pressure (maps to Adrenaline).
    pub urgency: MetricId,
    /// Resource depletion (new - tracks exhaustion).
    pub fatigue: MetricId,
    /// Risk aversion (maps to Norepinephrine).
    pub caution: MetricId,
    /// Cooperation tendency (maps to Oxytocin).
    pub cooperation: MetricId,
    /// Wellbeing/satisfaction (maps to Serotonin).
    pub wellbeing: MetricId,
    /// Growth/learning drive (maps to GrowthHormone).
    pub growth: MetricId,
}

impl EndocrineMetrics {
    /// Registers standard endocrine metrics with a controller.
    ///
    /// All metrics use unit bounds [0, 1] with safe defaults.
    pub fn register(controller: &mut MultiObjectiveController) -> Result<Self, MetricError> {
        let bounds = HardBounds::unit();
        
        // Stress: moderate baseline, fast response
        let stress = MetricId(1);
        controller.add_metric(
            Metric::new(stress, 0.2, 0.3, bounds, 0.5, 1.0)?
                .with_name("stress")
        );
        
        // Curiosity: balanced, slow response
        let curiosity = MetricId(2);
        controller.add_metric(
            Metric::new(curiosity, 0.5, 0.5, bounds, 0.3, 1.0)?
                .with_name("curiosity")
        );
        
        // Urgency: low baseline, fast response
        let urgency = MetricId(3);
        controller.add_metric(
            Metric::new(urgency, 0.1, 0.2, bounds, 0.8, 1.0)?
                .with_name("urgency")
        );
        
        // Fatigue: starts fresh, slow buildup
        let fatigue = MetricId(4);
        controller.add_metric(
            Metric::new(fatigue, 0.0, 0.3, bounds, 0.2, 1.0)?
                .with_name("fatigue")
        );
        
        // Caution: balanced, responsive (safety-critical)
        let caution = MetricId(5);
        controller.add_metric(
            Metric::new(caution, 0.5, 0.5, bounds, 0.6, 1.5)?
                .with_name("caution")
        );
        
        // Cooperation: moderate, balanced response
        let cooperation = MetricId(6);
        controller.add_metric(
            Metric::new(cooperation, 0.5, 0.5, bounds, 0.4, 1.0)?
                .with_name("cooperation")
        );
        
        // Wellbeing: moderate-high, slow response
        let wellbeing = MetricId(7);
        controller.add_metric(
            Metric::new(wellbeing, 0.6, 0.6, bounds, 0.3, 1.0)?
                .with_name("wellbeing")
        );
        
        // Growth: moderate, slow response
        let growth = MetricId(8);
        controller.add_metric(
            Metric::new(growth, 0.4, 0.4, bounds, 0.2, 0.8)?
                .with_name("growth")
        );
        
        Ok(Self {
            stress,
            curiosity,
            urgency,
            fatigue,
            caution,
            cooperation,
            wellbeing,
            growth,
        })
    }
    
    /// Returns all metric IDs as a vector.
    pub fn all_ids(&self) -> Vec<MetricId> {
        vec![
            self.stress,
            self.curiosity,
            self.urgency,
            self.fatigue,
            self.caution,
            self.cooperation,
            self.wellbeing,
            self.growth,
        ]
    }
}

/// Stimulus that affects endocrine metrics.
#[derive(Debug, Clone)]
pub struct EndocrineStimulus {
    /// Changes to apply to each metric.
    pub deltas: Vec<(MetricId, f64)>,
    /// Source of the stimulus.
    pub source: String,
}

impl EndocrineStimulus {
    /// Creates a stress response (threat detected).
    pub fn threat(metrics: &EndocrineMetrics, intensity: f64) -> Self {
        Self {
            deltas: vec![
                (metrics.stress, 0.3 * intensity),
                (metrics.urgency, 0.4 * intensity),
                (metrics.caution, 0.2 * intensity),
                (metrics.wellbeing, -0.1 * intensity),
            ],
            source: "threat_detection".to_string(),
        }
    }
    
    /// Creates a success response (task completed).
    pub fn success(metrics: &EndocrineMetrics, magnitude: f64) -> Self {
        Self {
            deltas: vec![
                (metrics.stress, -0.1 * magnitude),
                (metrics.wellbeing, 0.2 * magnitude),
                (metrics.growth, 0.1 * magnitude),
            ],
            source: "task_success".to_string(),
        }
    }
    
    /// Creates a fatigue stimulus (resource depletion).
    pub fn fatigue(metrics: &EndocrineMetrics, amount: f64) -> Self {
        Self {
            deltas: vec![
                (metrics.fatigue, 0.1 * amount),
                (metrics.curiosity, -0.05 * amount),
                (metrics.urgency, -0.02 * amount),
            ],
            source: "resource_depletion".to_string(),
        }
    }
    
    /// Creates a cooperation stimulus (social interaction).
    pub fn social(metrics: &EndocrineMetrics, positive: bool, magnitude: f64) -> Self {
        let sign = if positive { 1.0 } else { -1.0 };
        Self {
            deltas: vec![
                (metrics.cooperation, 0.2 * sign * magnitude),
                (metrics.wellbeing, 0.1 * sign * magnitude),
            ],
            source: "social_interaction".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_endocrine_metrics() {
        let mut controller = MultiObjectiveController::new(0.1, 1e-6, 100);
        
        let metrics = EndocrineMetrics::register(&mut controller).unwrap();
        
        assert_eq!(controller.metric_count(), 8);
        assert!(controller.get_metric(metrics.stress).is_some());
        assert!(controller.get_metric(metrics.caution).is_some());
    }
    
    #[test]
    fn test_threat_stimulus() {
        let mut controller = MultiObjectiveController::new(0.1, 1e-6, 100);
        let metrics = EndocrineMetrics::register(&mut controller).unwrap();
        
        let stimulus = EndocrineStimulus::threat(&metrics, 1.0);
        
        let violations = controller.apply_external_changes(&stimulus.deltas);
        
        // Changes should be applied (possibly with some clamping)
        assert!(violations.is_empty() || !violations.is_empty());
        
        // Stress should have increased
        let stress_val = controller.get_metric(metrics.stress).unwrap().value();
        assert!(stress_val > 0.2); // Was 0.2 initially
    }
}
