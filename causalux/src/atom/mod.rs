//! Da Vinci Atom Layer
//! 
//! Unified primitive for all CAUSALUX data types.
//! Everything is a CausalAtom - documents, counters, tensors, tokens.

pub mod causal_atom;
pub mod compose;

pub use causal_atom::{CausalAtom, AtomValue, AtomMeta};
pub use compose::{AtomComposer, CompositeAtom, AtomRef};
