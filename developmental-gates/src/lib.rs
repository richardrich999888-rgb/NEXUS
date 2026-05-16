//! Developmental Gates for ASI
//!
//! Implements staged capability unlock based on demonstrated stability:
//! - Capabilities are gated behind developmental stages
//! - Stages require homeostatic stability over time
//! - Regression is possible if stability is lost
//!
//! # Biological Analogy
//!
//! Like human cognitive development, capabilities unlock progressively:
//! - Stage 0 (Infant): Basic perception/response only
//! - Stage 1 (Child): Limited planning, supervised actions
//! - Stage 2 (Adolescent): Extended planning, broader actions
//! - Stage 3 (Adult): Full capability with self-regulation
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 Developmental Gates                      │
//! │                                                          │
//! │  ┌────────────────────────────────────────────────────┐ │
//! │  │              Stage Manager                          │ │
//! │  │  - Tracks current developmental stage               │ │
//! │  │  - Monitors stability metrics                       │ │
//! │  │  - Manages stage transitions                        │ │
//! │  └────────────────────────────────────────────────────┘ │
//! │                         │                                │
//! │                         ▼                                │
//! │  ┌────────────────────────────────────────────────────┐ │
//! │  │              Capability Registry                    │ │
//! │  │  - Lists all capabilities                           │ │
//! │  │  - Maps capabilities to required stages             │ │
//! │  │  - Provides unlock status queries                   │ │
//! │  └────────────────────────────────────────────────────┘ │
//! │                         │                                │
//! │                         ▼                                │
//! │  ┌────────────────────────────────────────────────────┐ │
//! │  │              Gate Enforcer                          │ │
//! │  │  - Checks capability access                         │ │
//! │  │  - Blocks unauthorized actions                      │ │
//! │  │  - Logs access attempts                             │ │
//! │  └────────────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod stage;
pub mod gate;
pub mod capability;

pub use stage::manager::{StageManager, StageConfig};
pub use stage::definition::{DevelopmentalStage, StageRequirements};
pub use gate::enforcer::{GateEnforcer, AccessResult};
pub use capability::registry::{Capability, CapabilityRegistry};
