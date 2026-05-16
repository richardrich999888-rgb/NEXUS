//! Autonomic System for ASI
//!
//! Implements the biological analogy of the autonomic nervous system:
//! - ACT (Action) mode: High arousal, fast responses, risk-tolerant
//! - CALM (Contemplation) mode: Low arousal, deliberate, safety-focused
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Autonomic System                       │
//! │                                                          │
//! │  ┌──────────────┐         ┌──────────────┐              │
//! │  │   ACT Mode   │◄───────►│  CALM Mode   │              │
//! │  │ (Sympathetic)│         │(Parasympathetic)            │
//! │  └──────────────┘         └──────────────┘              │
//! │          │                        │                      │
//! │          ▼                        ▼                      │
//! │  ┌──────────────────────────────────────────┐           │
//! │  │           Mode Controller                 │           │
//! │  │  - Arousal level management               │           │
//! │  │  - Transition thresholds                  │           │
//! │  │  - Reflex responses                       │           │
//! │  └──────────────────────────────────────────┘           │
//! │                      │                                   │
//! │                      ▼                                   │
//! │  ┌──────────────────────────────────────────┐           │
//! │  │         Homeostasis Integration           │           │
//! │  │  - Stress → ACT transition                │           │
//! │  │  - Recovery → CALM transition             │           │
//! │  └──────────────────────────────────────────┘           │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod mode;
pub mod reflex;
pub mod regulation;

pub use mode::controller::{AutonomicController, ControllerConfig};
pub use mode::state::{AutonomicMode, Arousal};
pub use reflex::response::{ReflexResponse, ReflexType};
pub use regulation::transition::{ModeTransition, TransitionTrigger};
