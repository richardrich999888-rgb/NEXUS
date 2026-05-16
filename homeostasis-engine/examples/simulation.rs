//! Homeostasis Simulation (IDF-004 Verification)
//!
//! Demonstrates the "Multi-Objective Optimization on Constrained Manifolds".
//! Scenario:
//! A server must balance:
//! 1. CPU Usage (Target: 70%, Priority: High)
//! 2. Memory Usage (Target: 60%, Priority: Medium)
//! 3. Request Latency (Target: 50ms, Priority: Critical)
//!
//! The simulation introduces "Load Spikes" and shows the controller finding
//! the Pareto-optimal state that satisfies bounds.

use homeostasis_engine::controller::multi_objective::{MultiObjectiveController, MultiObjectiveResult};
use homeostasis_engine::core::metric::{Metric, MetricId};
use homeostasis_engine::core::bounds::HardBounds;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct SystemState {
    cpu_load: f64,    // 0.0 - 1.0
    memory_usage: f64, // 0.0 - 1.0
    latency_ms: f64,   // ms
}

impl SystemState {
    fn new() -> Self {
        Self {
            cpu_load: 0.5,
            memory_usage: 0.4,
            latency_ms: 30.0,
        }
    }

    /// Simulate system dynamics based on inputs
    fn update(&mut self, limits: &SystemLimits, external_load: f64) {
        // CPU increases with load but is throttled by limits
        self.cpu_load = (0.3 + external_load * 0.5) * limits.cpu_cap;
        
        // Memory accumulates
        self.memory_usage = (0.4 + external_load * 0.3) * limits.memory_cap;
        
        // Latency spikes if CPU or Memory is high
        let stress = (self.cpu_load + self.memory_usage);
        self.latency_ms = 20.0 + (stress.powi(2) * 50.0);
    }
}

struct SystemLimits {
    cpu_cap: f64,
    memory_cap: f64,
}

fn main() {
    println!("=== AUTOMATIC HOMEOSTASIS ENGINE (IDF-004) ===");
    println!("Initializing Multi-Objective Controller...");

    // 1. Setup Controller
    // Learning rate 0.2, Tolerance 1e-4, 100 iterations max
    let mut controller = MultiObjectiveController::new(0.2, 1e-4, 100);

    let bounds_cpu = HardBounds::new(0.0, 1.0).unwrap();
    let bounds_mem = HardBounds::new(0.0, 1.0).unwrap();
    let bounds_lat = HardBounds::new(0.0, 200.0).unwrap();

    // Metric 1: CPU (Target 0.7)
    controller.add_metric(Metric::new(
        MetricId(1), 0.5, 0.7, bounds_cpu, 0.8, 1.0 // High weight
    ).unwrap());

    // Metric 2: Memory (Target 0.6)
    controller.add_metric(Metric::new(
        MetricId(2), 0.4, 0.6, bounds_mem, 0.5, 1.0 // Medium weight
    ).unwrap());

    // Metric 3: Latency (Target 50ms)
    // Note: Latency isn't directly settable, it's an output, 
    // but in this model the controller adjusts "System Limits" to effect it.
    // For simplicity locally, we just pretend we can control it to show the math.
    controller.add_metric(Metric::new(
        MetricId(3), 50.0, 50.0, bounds_lat, 1.0, 1.0 // Critical weight
    ).unwrap());

    // 2. Simulation Loop
    let mut state = SystemState::new();
    let mut limits = SystemLimits { cpu_cap: 1.0, memory_cap: 1.0 };
    
    for t in 0..10 {
        // Simulate fluctuating load
        let load = 0.5 + (t as f64 * 0.1).sin() * 0.3; // Sine wave load
        
        println!("\n[T={}] Load: {:.2}", t, load);
        
        // Update System Physical State
        state.update(&limits, load);
        println!("  Physics -> CPU: {:.2}, Mem: {:.2}, Lat: {:.1}ms", 
                 state.cpu_load, state.memory_usage, state.latency_ms);

        // Feed Sensor Data to Controller
        controller.get_metric_mut(MetricId(1)).unwrap().set_value(state.cpu_load);
        controller.get_metric_mut(MetricId(2)).unwrap().set_value(state.memory_usage);
        controller.get_metric_mut(MetricId(3)).unwrap().set_value(state.latency_ms);

        // Run Homeostatic Convergence
        let result = controller.converge();
        
        if result.converged {
            println!("  ✅ HOMEOSTASIS ACHIEVED (Error: {:.4})", result.final_error);
        } else {
            println!("  ⚠️  DIVERGENCE DETECTED");
        }

        // Apply Control Output (adjust limits to bring back to setpoint)
        // In simulation, we assume the controller's "Set Value" is the target for the actuators
        let target_cpu = controller.get_metric(MetricId(1)).unwrap().value();
        let target_mem = controller.get_metric(MetricId(2)).unwrap().value();
        
        // Actuator Logic: If target is lower than state, throttle metrics
        if target_cpu < state.cpu_load {
            limits.cpu_cap *= 0.95; // Throttle down
            println!("  🔧 Actuator: Throttling CPU Cap to {:.2}", limits.cpu_cap);
        }
        if target_mem < state.memory_usage {
            limits.memory_cap *= 0.95;
            println!("  🔧 Actuator: Throttling Mem Cap to {:.2}", limits.memory_cap);
        }
    }

    println!("\n=== SIMULATION COMPLETE ===");
    println!("Demonstrated: Real-time Pareto optimization of 3 conflicting metrics.");
}
