//! Integration tests for the complete homeostasis engine.

use homeostasis_engine::prelude::*;
use homeostasis_engine::integration::endocrine_bridge::{EndocrineMetrics, EndocrineStimulus};
use homeostasis_engine::diagnostics::health::HealthCheck;

#[test]
fn test_full_endocrine_system() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    
    let metrics = EndocrineMetrics::register(&mut controller).unwrap();
    
    // Should have 8 metrics
    assert_eq!(controller.metric_count(), 8);
    
    // System should start healthy
    let health = HealthCheck::check(&controller);
    assert!(health.is_healthy() || health.score > 0.7);
}

#[test]
fn test_threat_response() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    let metrics = EndocrineMetrics::register(&mut controller).unwrap();
    
    // Get initial stress
    let initial_stress = controller.get_metric(metrics.stress).unwrap().value();
    
    // Apply threat stimulus
    let stimulus = EndocrineStimulus::threat(&metrics, 1.0);
    controller.apply_external_changes(&stimulus.deltas);
    
    // Stress should have increased
    let new_stress = controller.get_metric(metrics.stress).unwrap().value();
    assert!(new_stress > initial_stress);
    
    // Let system recover
    let result = controller.converge();
    
    // Should converge
    assert!(result.converged);
}

#[test]
fn test_homeostatic_recovery() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    let metrics = EndocrineMetrics::register(&mut controller).unwrap();
    
    // Perturb the system
    controller.apply_external_changes(&[
        (metrics.stress, 0.5),
        (metrics.urgency, 0.5),
        (metrics.wellbeing, -0.3),
    ]);
    
    // System should be stressed
    let health_before = HealthCheck::check(&controller);
    
    // Let it recover
    controller.converge();
    
    // System should be healthier
    let health_after = HealthCheck::check(&controller);
    
    assert!(health_after.score >= health_before.score);
}

#[test]
fn test_bounds_never_violated() {
    let mut controller = MultiObjectiveController::new(0.2, 1e-6, 100);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    controller.add_metric(Metric::new(
        MetricId(1), 0.5, 0.5, bounds, 1.0, 1.0
    ).unwrap());
    
    // Try to violate bounds multiple times
    for _ in 0..100 {
        controller.apply_external_changes(&[(MetricId(1), 10.0)]);
        controller.step();
        
        let value = controller.get_metric(MetricId(1)).unwrap().value();
        assert!(bounds.contains(value));
    }
}

#[test]
fn test_snapshot_and_state() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 100);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    controller.add_metric(Metric::new(MetricId(1), 0.3, 0.5, bounds, 0.5, 1.0).unwrap());
    controller.add_metric(Metric::new(MetricId(2), 0.7, 0.5, bounds, 0.5, 1.0).unwrap());
    
    let snapshot = controller.snapshot();
    
    assert_eq!(snapshot.len(), 2);
    assert!((snapshot[&MetricId(1)] - 0.3).abs() < 0.01);
    assert!((snapshot[&MetricId(2)] - 0.7).abs() < 0.01);
}
