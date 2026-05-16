//! Multi-objective controller tests.

use homeostasis_engine::core::bounds::HardBounds;
use homeostasis_engine::core::metric::{Metric, MetricId};
use homeostasis_engine::controller::multi_objective::MultiObjectiveController;

#[test]
fn test_multi_metric_convergence() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    controller.add_metric(Metric::new(
        MetricId(1), 0.0, 0.5, bounds, 0.5, 1.0
    ).unwrap());
    
    controller.add_metric(Metric::new(
        MetricId(2), 1.0, 0.3, bounds, 0.5, 1.0
    ).unwrap());

    let result = controller.converge();
    
    assert!(result.converged);
    
    let m1 = controller.get_metric(MetricId(1)).unwrap();
    let m2 = controller.get_metric(MetricId(2)).unwrap();
    
    assert!((m1.value() - 0.5).abs() < 0.05);
    assert!((m2.value() - 0.3).abs() < 0.05);
}

#[test]
fn test_weighted_optimization() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    // High weight metric - start at 0.2 (not at bound)
    controller.add_metric(Metric::new(
        MetricId(1), 0.2, 0.5, bounds, 0.5, 10.0  // weight = 10
    ).unwrap());
    
    // Low weight metric - start at 0.2 (not at bound)
    controller.add_metric(Metric::new(
        MetricId(2), 0.2, 0.5, bounds, 0.5, 1.0   // weight = 1
    ).unwrap());
    
    // Run enough steps
    for _ in 0..50 {
        controller.step();
    }
    
    // Both should have moved toward 0.5
    let m1 = controller.get_metric(MetricId(1)).unwrap();
    let m2 = controller.get_metric(MetricId(2)).unwrap();
    
    // Both should have increased from 0.2 toward 0.5
    assert!(m1.value() > 0.2);
    assert!(m2.value() > 0.2);
}

#[test]
fn test_external_change_violation_detection() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    controller.add_metric(Metric::new(
        MetricId(1), 0.5, 0.5, bounds, 0.5, 1.0
    ).unwrap());
    
    // Try to push beyond bounds
    let violations = controller.apply_external_changes(&[(MetricId(1), 2.0)]);
    
    // Should report violation
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].metric_id, MetricId(1));
    
    // But value should be clamped
    assert_eq!(controller.get_metric(MetricId(1)).unwrap().value(), 1.0);
}

#[test]
fn test_health_assessment() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    // Add a healthy metric
    controller.add_metric(Metric::new(
        MetricId(1), 0.5, 0.5, bounds, 0.5, 1.0
    ).unwrap());
    
    let health = controller.health();
    
    assert!(health.healthy);
    assert_eq!(health.at_lower_bound, 0);
    assert_eq!(health.at_upper_bound, 0);
    assert!(health.score() > 0.9);
}

#[test]
fn test_health_degradation() {
    let mut controller = MultiObjectiveController::new(0.1, 1e-6, 1000);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    // Add a metric at bound
    controller.add_metric(Metric::new(
        MetricId(1), 0.0, 0.5, bounds, 0.5, 1.0
    ).unwrap());
    
    let health = controller.health();
    
    assert!(health.at_lower_bound > 0);
    assert!(!health.healthy);
    assert!(health.score() < 1.0);
}

#[test]
fn test_many_metrics() {
    let mut controller = MultiObjectiveController::new(0.05, 1e-6, 2000);
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    // Add 10 metrics with different initial values and setpoints
    for i in 0..10 {
        let initial = (i as f64) / 10.0;
        let setpoint = 0.5;
        
        controller.add_metric(Metric::new(
            MetricId(i), initial, setpoint, bounds, 0.5, 1.0
        ).unwrap());
    }
    
    let result = controller.converge();
    
    assert!(result.converged);
    
    // All should be near 0.5
    for i in 0..10 {
        let m = controller.get_metric(MetricId(i)).unwrap();
        assert!((m.value() - 0.5).abs() < 0.1);
    }
}
