//! Nervous System for ASI
//!
//! The central coordinator integrating all bio-inspired safety layers:
//! - Perception: Processing inputs from environment
//! - Decision: Selecting actions through safety filters
//! - Motor: Executing outputs with capability gating
//! - Integration: Coordinating homeostasis, autonomic, and developmental systems
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                           Nervous System                                  │
//! │                                                                           │
//! │  Inputs ─────┐      ┌─────────────────────────────────────┐      ┌───────│
//! │              ▼      │                                     │      ▼       │
//! │         ┌────────┐  │  ┌─────────────────────────────┐    │  ┌────────┐  │
//! │         │Percept-│  │  │      Decision Engine        │    │  │ Motor  │──┼──▶ Outputs
//! │         │  ion   │──┼─▶│  - Risk assessment          │────┼─▶│ Output │  │
//! │         │ Layer  │  │  │  - Capability checking      │    │  │ Layer  │  │
//! │         └────────┘  │  │  - Action selection         │    │  └────────┘  │
//! │              │      │  └─────────────────────────────┘    │      │       │
//! │              │      │              │                      │      │       │
//! │              │      └──────────────┼──────────────────────┘      │       │
//! │              │                     │                             │       │
//! │              ▼                     ▼                             ▼       │
//! │         ┌────────────────────────────────────────────────────────────┐   │
//! │         │                    Safety Layers                            │   │
//! │         │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐ │   │
//! │         │  │Homeostasis │ │ Autonomic  │ │Development │ │ Immune   │ │   │
//! │         │  │  Engine    │ │  System    │ │   Gates    │ │ Protocol │ │   │
//! │         │  └────────────┘ └────────────┘ └────────────┘ └──────────┘ │   │
//! │         └────────────────────────────────────────────────────────────┘   │
//! └──────────────────────────────────────────────────────────────────────────┘
//! ```

pub mod perception;
pub mod decision;
pub mod motor;
pub mod integration;

pub use integration::coordinator::{NervousSystemCoordinator, CoordinatorConfig};
pub use integration::safety::{SafetyState, SafetySummary};
