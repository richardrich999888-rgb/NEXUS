// Causal Operation v2.0 with version vectors

use crate::version_vector::VersionVector;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Causal Operation v2.0
/// 
/// An immutable, cryptographically-signed operation with explicit dependencies
/// and version vector tracking for conflict detection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausalOp {
    /// Unique operation ID (hash of signature)
    pub id: String,
    
    /// Operation type (e.g., "insert", "delete", "update")
    pub operation: String,
    
    /// Operation input/parameters
    pub input: serde_json::Value,
    
    /// IDs of operations this depends on
    pub dependencies: BTreeSet<String>,
    
    /// Version vector at time of creation
    pub version_vector: VersionVector,
    
    /// Lamport logical clock
    pub lamport_clock: u64,
    
    /// Wall clock timestamp (untrusted, for UX only)
    pub wall_clock: u64,
    
    /// Node that created this operation
    pub node_id: String,
    
    /// Identity of creator (hash of public key)
    pub identity: String,
    
    /// Ed25519 signature
    pub signature: Vec<u8>,
}

impl CausalOp {
    /// Create a new causal operation
    /// 
    /// # Arguments
    /// 
    /// * `operation` - Operation type
    /// * `input` - Operation parameters
    /// * `dependencies` - Set of operation IDs this depends on
    /// * `version_vector` - Current version vector
    /// * `node_id` - ID of node creating this operation
    /// * `keypair` - Ed25519 keypair for signing
    pub fn new(
        operation: String,
        input: serde_json::Value,
        dependencies: BTreeSet<String>,
        version_vector: VersionVector,
        node_id: String,
        keypair: &SigningKey,
    ) -> Self {
        let identity = Self::derive_identity(&keypair.verifying_key());
        let lamport_clock = version_vector.get(&node_id);
        let wall_clock = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let message = Self::create_message(
            &operation,
            &input,
            &dependencies,
            &version_vector,
            lamport_clock,
            wall_clock,
        );
        let signature = keypair.sign(message.as_bytes()).to_bytes().to_vec();
        let id = Self::compute_hash(&signature);

        Self {
            id,
            operation,
            input,
            dependencies,
            version_vector,
            lamport_clock,
            wall_clock,
            node_id,
            identity,
            signature,
        }
    }

    /// Verify operation signature
    pub fn verify(&self, public_key: &VerifyingKey) -> bool {
        let message = Self::create_message(
            &self.operation,
            &self.input,
            &self.dependencies,
            &self.version_vector,
            self.lamport_clock,
            self.wall_clock,
        );

        let sig = Signature::from_bytes(&self.signature.clone().try_into().unwrap_or([0u8; 64]));
        public_key.verify(message.as_bytes(), &sig).is_ok()
    }

    /// Check if this operation conflicts with another
    pub fn conflicts_with(&self, other: &CausalOp) -> bool {
        self.version_vector.conflicts_with(&other.version_vector)
    }

    /// Derive identity from public key
    pub fn derive_identity(public_key: &VerifyingKey) -> String {
        Self::compute_hash(&public_key.to_bytes())
    }

    /// Compute SHA-256 hash
    fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Create message for signing/verification
    fn create_message(
        operation: &str,
        input: &serde_json::Value,
        dependencies: &BTreeSet<String>,
        version_vector: &VersionVector,
        lamport: u64,
        wall: u64,
    ) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}",
            operation,
            serde_json::to_string(input).unwrap(),
            dependencies.iter().cloned().collect::<Vec<_>>().join(","),
            serde_json::to_string(&version_vector.versions).unwrap(),
            lamport,
            wall
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_create_and_verify() {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        let keypair = SigningKey::from_bytes(&bytes);
        let mut vv = VersionVector::new();
        vv.increment("node1");

        let op = CausalOp::new(
            "test_op".to_string(),
            serde_json::json!({"key": "value"}),
            BTreeSet::new(),
            vv,
            "node1".to_string(),
            &keypair,
        );

        assert!(op.verify(&keypair.verifying_key()));
        assert_eq!(op.operation, "test_op");
        assert_eq!(op.node_id, "node1");
    }

    #[test]
    fn test_conflict_detection() {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        let keypair = SigningKey::from_bytes(&bytes);

        let mut vv1 = VersionVector::new();
        vv1.increment("node1");

        let mut vv2 = VersionVector::new();
        vv2.increment("node2");

        let op1 = CausalOp::new(
            "edit".to_string(),
            serde_json::json!({"value": "A"}),
            BTreeSet::new(),
            vv1,
            "node1".to_string(),
            &keypair,
        );

        let op2 = CausalOp::new(
            "edit".to_string(),
            serde_json::json!({"value": "B"}),
            BTreeSet::new(),
            vv2,
            "node2".to_string(),
            &keypair,
        );

        assert!(op1.conflicts_with(&op2));
    }

    #[test]
    fn test_causally_ordered_operations() {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        let keypair = SigningKey::from_bytes(&bytes);

        let mut vv1 = VersionVector::new();
        vv1.increment("node1");

        let op1 = CausalOp::new(
            "op1".to_string(),
            serde_json::json!({}),
            BTreeSet::new(),
            vv1.clone(),
            "node1".to_string(),
            &keypair,
        );

        let mut vv2 = vv1.clone();
        vv2.increment("node1");

        let mut deps = BTreeSet::new();
        deps.insert(op1.id.clone());

        let op2 = CausalOp::new(
            "op2".to_string(),
            serde_json::json!({}),
            deps,
            vv2,
            "node1".to_string(),
            &keypair,
        );

        // op2 depends on op1, so they don't conflict
        assert!(!op1.conflicts_with(&op2));
    }
}
