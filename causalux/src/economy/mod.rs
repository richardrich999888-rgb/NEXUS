//! Morgan Economy Layer
//! 
//! Token-based economic system for CAUSALUX operations.
//! Enables metered usage, spam prevention, and incentive alignment.

pub mod token;
pub mod ledger;
pub mod pricing;

pub use token::{CausalToken, TokenBalance};
pub use ledger::{EconomyLedger, Transaction, TransactionType};
pub use pricing::{OperationPricing, PricingPolicy};
