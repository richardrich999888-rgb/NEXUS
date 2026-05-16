//! Single metric controller tests.

use homeostasis_engine::core::bounds::HardBounds;
use homeostasis_engine::core::metric::{Metric, MetricId};
use homeostasis_engine::controller::single_metric::SingleMetricController;

#[test]
fn test_convergence_to_setpoint() {
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    let mut metric = Metric::new(
        MetricId(1),
        0.0,    // start at 0
        0.5,    // setpoint at 0.5
        bounds,
        0.5,    // gain
        1.0,
    ).unwrap();

    let controller = SingleMetricController::new(0.1);

    // Run 100 steps
    for _ in 0..100 {
        controller.step(&mut metric);
    }

    // Should converge near setpoint
    assert!((metric.value() - 0.5).abs() < 0.01);
}

#[test]
fn test_bounds_enforcement_setpoint() {
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    let result = Metric::new(
        MetricId(1),
        0.5,
        2.0,    // setpoint OUTSIDE bounds
        bounds,
        0.5,
        1.0,
    );

    // Construction should fail
    assert!(result.is_err());
}

#[test]
fn test_hard_bound_prevents_overshoot() {
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    let mut metric = Metric::new(
        MetricId(1),
        0.95,   // near upper bound
        0.9,    // setpoint slightly lower
        bounds,
        2.0,    // high gain
        1.0,
    ).unwrap();

    // Force value above bound
    metric.update(0.5);
    
    // Should be clamped to 1.0
    assert_eq!(metric.value(), 1.0);
}

#[test]
fn test_negative_feedback_direction() {
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    
    // Above setpoint
    let metric_high = Metric::new(
        MetricId(1), 0.8, 0.5, bounds, 1.0, 1.0
    ).unwrap();
    assert!(metric_high.correction_signal() < 0.0); // Should decrease
    
    // Below setpoint
    let metric_low = Metric::new(
        MetricId(2), 0.2, 0.5, bounds, 1.0, 1.0
    ).unwrap();
    assert!(metric_low.correction_signal() > 0.0); // Should increase
}

#[test]
fn test_convergence_with_damping() {
    let bounds = HardBounds::new(0.0, 1.0).unwrap();
    let mut metric = Metric::new(
        MetricId(1), 0.0, 0.5, bounds, 1.0, 1.0
    ).unwrap();
    
    let controller = SingleMetricController::with_damping(0.2, 0.3);
    
    let (steps, converged) = controller.converge(&mut metric, 0.01, 200);
    
    assert!(converged);
    assert!(steps < 200);
}
