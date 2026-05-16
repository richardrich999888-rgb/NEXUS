//! Execution guard implementations (biological / accountability).

pub mod nervous;
pub mod immune;
pub mod composite;
pub use nervous::NervousSystemGuard;
pub use immune::ImmuneGuard;
pub use composite::CompositeGuard;
