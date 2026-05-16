//! # TELOS Protocol
//!
//! **Cognitive Accountability Protocol for ASI**
//!
//! TELOS enforces a commitment membrane between AI reasoning and action,
//! where crossing requires entropy-metered cost, verified authority,
//! external attestation, and creates unforkable trust.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    TELOS PROTOCOL                           │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Layer 5: Trust Accumulator                                 │
//! │  ├── Commitment History                                     │
//! │  └── Unforkable Trust Score                                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Layer 4: Validator Network                                 │
//! │  ├── External Validators                                    │
//! │  ├── Slashable Stake                                        │
//! │  └── BFT Attestation                                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Layer 3: Authority Registry                                │
//! │  ├── Agent → Scope Mapping                                  │
//! │  ├── Delegation Chains                                      │
//! │  └── Additive Constraints                                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Layer 2: Entropy Meter                                     │
//! │  ├── VDF / Random Beacon                                    │
//! │  ├── Consequence Scaling                                    │
//! │  └── Budget Management                                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Layer 1: Commitment Membrane                               │
//! │  ├── Reversible Zone (Reasoning)                            │
//! │  ├── Irreversible Zone (Action)                             │
//! │  └── Crossing Protocol                                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use telos_protocol::{
//!     membrane::{CommitmentMembrane, Decision},
//!     entropy::{EntropyMeter, ConsequenceTier},
//!     authority::{AuthorityRegistry, AgentId},
//! };
//!
//! // Create the protocol layers
//! let mut membrane = CommitmentMembrane::new();
//! let mut entropy = EntropyMeter::new(1000); // 1000 entropy budget
//! let mut authority = AuthorityRegistry::new();
//!
//! // Define a decision to commit
//! let decision = Decision::new("deploy_model_v2", ConsequenceTier::High);
//!
//! // Attempt commitment (requires entropy + authority + validation)
//! let result = membrane.request_crossing(decision, &mut entropy, &authority);
//! ```

pub mod membrane;
pub mod entropy;
pub mod authority;
pub mod validator;
pub mod trust;
pub mod error;
pub mod merkle;
pub mod ledger;
pub mod vdf;
pub mod network;
#[cfg(feature = "python")]
mod python;

pub use membrane::{CommitmentMembrane, Decision, CrossingResult};
pub use entropy::{EntropyMeter, ConsequenceTier, EntropyProof};
pub use authority::{AuthorityRegistry, AgentId, Authority, Constraint};
pub use validator::{Validator, Attestation, ValidatorNetwork};
pub use trust::{TrustAccumulator, TrustScore, CommitmentHistory};
pub use error::TelosError;
pub use merkle::{MerkleTree, MerkleProof};
pub use ledger::{Ledger, Block, LedgerPosition};
pub use vdf::{VdfGenerator, VdfVerifier, VdfProof};
pub use network::{NetworkCoordinator, Message, AttestationRequest, AttestationResponse};
