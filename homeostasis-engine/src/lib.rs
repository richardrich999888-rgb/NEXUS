//! # Homeostasis Engine
//!
//! Constraint enforcement substrate for bio-inspired ASI safety.
//!
//! ## Core Concepts
//!
//! - **Metric**: An optimizable value with setpoint and bounds
//! - **Bounds**: Hard limits that cannot be violated
//! - **Setpoint**: Target value the system maintains
//! - **Controller**: Applies negative feedback to maintain homeostasis
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           External Perturbations            │
//! └─────────────────────┬───────────────────────┘
//!                       ▼
//! ┌─────────────────────────────────────────────┐
//! │              Metric State                   │
//! │   value, setpoint, bounds, gain, weight     │
//! └─────────────────────┬───────────────────────┘
//!                       ▼
//! ┌─────────────────────────────────────────────┐
//! │      Negative Feedback Controller           │
//! │   Δx = -k(x - setpoint) - λ·penalty(x)     │
//! └─────────────────────┬───────────────────────┘
//!                       ▼
//! ┌─────────────────────────────────────────────┐
//! │          Bounds Enforcement                 │
//! │       clamp(x, lower, upper)                │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use homeostasis_engine::prelude::*;
//!
//! // Create bounds
//! let bounds = HardBounds::new(0.0, 1.0).unwrap();
//!
//! // Create metric
//! let mut metric = Metric::new(
//!     MetricId(1),
//!     0.2,    // initial value
//!     0.5,    // setpoint
//!     bounds,
//!     0.5,    // gain
//!     1.0,    // weight
//! ).unwrap();
//!
//! // Create controller
//! let controller = SingleMetricController::new(0.1);
//!
//! // Apply corrections
//! for _ in 0..100 {
//!     controller.step(&mut metric);
//! }
//!
//! assert!((metric.value() - 0.5).abs() < 0.01);
//! ```

pub mod core;
pub mod controller;
pub mod constraints;
pub mod solver;
pub mod diagnostics;
pub mod integration;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::core::bounds::{HardBounds, BoundsError};
    pub use crate::core::metric::{Metric, MetricId, MetricError};
    pub use crate::core::setpoint::{AdaptiveSetpoint, SetpointConfig};
    pub use crate::core::feedback::FeedbackConfig;
    pub use crate::controller::single_metric::{SingleMetricController, CorrectionResult};
    pub use crate::controller::multi_objective::{
        MultiObjectiveController, 
        BoundsViolation,
        MultiObjectiveResult,
        ConvergenceResult,
        SystemHealth,
    };
    pub use crate::diagnostics::health::{HealthStatus, HealthCheck};
}

/// Re-export core types at crate root
pub use crate::core::bounds::HardBounds;
pub use crate::core::metric::{Metric, MetricId};
pub use crate::controller::single_metric::SingleMetricController;
pub use crate::controller::multi_objective::MultiObjectiveController;
