//! # nexus-agp
//!
//! Bridge between AGP (Agent Governance Protocol) and NEXUS infrastructure.
//!
//! ## Patent Claims Enabled
//!
//! - Claim 5: CRDT-Based Reputation Convergence
//! - Claim 6: PQC-Bound Agent Identity
//! - Claim 7: Cross-Protocol Reputation Portability
//! - Claim 8: Bio-inspired Computational Governance (Artificial Endocrine System)
//! - Claim 9: Virtual Gland Hormone Secretion
//! - Claim 10: Biological Feedback Loops for Self-Regulation
//! - Claim 11: Circadian Rhythm Modulation
//! - Claim 12: Allostatic Adaptation
//!
//! ## Architecture
//!
//! ```text
//! AGP (Python) ←→ nexus-agp (Rust) ←→ NEXUS Infrastructure
//!                      ↓
//!              Artificial Endocrine System
//!                      ↓
//!         [Hypothalamus] → [Pituitary] → [Glands]
//!                      ↓
//!              Homeostasis Controller
//! ```

// Core modules
pub mod identity;
pub mod reputation;
pub mod verification;

// Artificial Human Endocrine System (AHES) modules
pub mod endocrine;
pub mod glands;
pub mod homeostasis;

// Core exports
pub use identity::{NexusAgentIdentity, AgentRegistration};
pub use reputation::{ReputationCRDT, ReputationProof};
pub use verification::{NexusVerifier, VerificationResult};

// AHES exports
pub use endocrine::{Hormone, HormoneLevel, HormoneReceptor, EndocrineState};
pub use glands::{Stimulus, Gland, GlandularSystem};
pub use homeostasis::{HomeostasisController, HealthStatus, SetPoint};
