//! ETK constants. Schema version, zero hash.

use etk_types::Hash256;

/// Schema version. Immutable; any change requires new version.
pub const ETK_SCHEMA_VERSION: &str = "1.0";

/// Zero hash (genesis previous_event_hash).
pub const ZERO_HASH: Hash256 = Hash256::zero();
