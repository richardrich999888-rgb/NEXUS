// NEXUS Identity: Capability-based identity intrinsic to computation
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// Key innovation: Identity is embedded INTO computation, not wrapped around it.
// No external auth service calls needed. PCU carries its own proof of authorization.

use serde::{Deserialize, Serialize};
use rand::Rng;
use ed25519_dalek::{SigningKey, Signature, VerifyingKey, Signer, Verifier};

use crate::Timestamp;

// ============================================================================
// PRINCIPAL ID - Who is making this request
// ============================================================================

/// Unique identifier for a principal (user, service, device)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrincipalId(pub [u8; 32]);

impl PrincipalId {
    /// Generate a random principal ID
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        PrincipalId(bytes)
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        PrincipalId(bytes)
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hex representation
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Short hex for display
    pub fn short_hex(&self) -> String {
        format!("{}..{}", &self.to_hex()[..8], &self.to_hex()[56..])
    }

    /// Anonymous/public principal
    pub fn anonymous() -> Self {
        PrincipalId([0u8; 32])
    }

    /// Check if anonymous
    pub fn is_anonymous(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl std::fmt::Display for PrincipalId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "principal:{}", self.short_hex())
    }
}

// ============================================================================
// CAPABILITY - What actions are permitted
// ============================================================================

/// A single capability (permission)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// Resource this capability grants access to (e.g., "uso:*", "pcu:execute")
    pub resource: String,
    /// Actions permitted (e.g., ["read", "write"])
    pub actions: Vec<String>,
    /// Constraints/caveats (e.g., {"max_bytes": 1000000})
    pub constraints: Vec<CapabilityConstraint>,
}

/// Constraint on a capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityConstraint {
    /// Maximum value for a numeric property
    MaxValue { property: String, value: u64 },
    /// Required string match
    StringMatch { property: String, pattern: String },
    /// Time-based constraint
    TimeWindow { not_before: Timestamp, not_after: Timestamp },
    /// Rate limit
    RateLimit { max_per_second: u32 },
}

impl Capability {
    /// Create a new capability
    pub fn new(resource: impl Into<String>, actions: Vec<String>) -> Self {
        Capability {
            resource: resource.into(),
            actions,
            constraints: Vec::new(),
        }
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: CapabilityConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Check if this capability permits an action on a resource
    pub fn permits(&self, resource: &str, action: &str) -> bool {
        // Simple glob matching for resources
        if self.resource == "*" || self.resource == resource {
            self.actions.contains(&"*".to_string()) || self.actions.contains(&action.to_string())
        } else if self.resource.ends_with("*") {
            let prefix = &self.resource[..self.resource.len() - 1];
            if resource.starts_with(prefix) {
                self.actions.contains(&"*".to_string()) || self.actions.contains(&action.to_string())
            } else {
                false
            }
        } else {
            false
        }
    }
}

// ============================================================================
// CAPABILITY SET - Collection of capabilities
// ============================================================================

/// Set of capabilities a principal holds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilitySet {
    pub capabilities: Vec<Capability>,
}

impl CapabilitySet {
    /// Create empty set
    pub fn new() -> Self {
        CapabilitySet { capabilities: Vec::new() }
    }

    /// Add capability
    pub fn add(&mut self, capability: Capability) {
        self.capabilities.push(capability);
    }

    /// Check if any capability permits action
    pub fn permits(&self, resource: &str, action: &str) -> bool {
        self.capabilities.iter().any(|c| c.permits(resource, action))
    }

    /// Check if capability set contains a specific capability
    pub fn contains(&self, cap: &Capability) -> bool {
        self.capabilities.iter().any(|c| c == cap)
    }

    /// Create with read-only access
    pub fn read_only(resource: impl Into<String>) -> Self {
        let mut set = CapabilitySet::new();
        set.add(Capability::new(resource, vec!["read".to_string()]));
        set
    }

    /// Create with full access
    pub fn full_access(resource: impl Into<String>) -> Self {
        let mut set = CapabilitySet::new();
        set.add(Capability::new(resource, vec!["*".to_string()]));
        set
    }
}

// ============================================================================
// DELEGATION CHAIN - Acting on behalf of another
// ============================================================================

/// Single link in delegation chain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationLink {
    /// Who granted this delegation
    pub from: PrincipalId,
    /// Who received the delegation
    pub to: PrincipalId,
    /// Capabilities being delegated (subset of grantor's)
    pub capabilities: CapabilitySet,
    /// When this delegation expires
    pub expires: Timestamp,
    /// Signature from grantor
    pub signature: Vec<u8>,
}

/// Chain of delegations (for acting on behalf of another)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationChain {
    pub links: Vec<DelegationLink>,
}

impl DelegationChain {
    /// Create new empty chain
    pub fn new() -> Self {
        DelegationChain { links: Vec::new() }
    }

    /// Add delegation link
    pub fn add(&mut self, link: DelegationLink) {
        self.links.push(link);
    }

    /// Get original principal at start of chain
    pub fn original_principal(&self) -> Option<PrincipalId> {
        self.links.first().map(|l| l.from)
    }

    /// Get final delegatee
    pub fn final_delegatee(&self) -> Option<PrincipalId> {
        self.links.last().map(|l| l.to)
    }

    /// Check if entire chain is valid (not expired, signatures valid)
    pub fn is_valid(&self, now: Timestamp) -> bool {
        let mut prev_to: Option<PrincipalId> = None;

        for (_i, link) in self.links.iter().enumerate() {
            // Check expiry
            if link.expires <= now {
                return false;
            }

            // Check continuity: grantor must be the delegatee of the previous link
            if let Some(prev) = prev_to {
                if link.from != prev {
                    return false;
                }
            }
            prev_to = Some(link.to);

            // Verify signature
            if !link.verify() {
                return false;
            }
        }
        true
    }
}

impl DelegationLink {
    /// Verify the signature of this delegation link
    pub fn verify(&self) -> bool {
        let public_key_bytes = self.from.as_bytes();
        let verifying_key = match VerifyingKey::from_bytes(public_key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let signature = match Signature::from_slice(&self.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Create data to verify (canonical representation)
        let data = self.signing_data();
        verifying_key.verify(&data, &signature).is_ok()
    }

    /// Sign the delegation link
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), String> {
        if signing_key.verifying_key().to_bytes() != *self.from.as_bytes() {
            return Err("Signing key does not match grantor PrincipalId".to_string());
        }

        let data = self.signing_data();
        let signature = signing_key.sign(&data);
        self.signature = signature.to_bytes().to_vec();
        Ok(())
    }

    fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.from.as_bytes());
        data.extend_from_slice(self.to.as_bytes());
        // For capabilities and expires, use bincode for stable serialization
        if let Ok(ser) = bincode::serialize(&(&self.capabilities, self.expires)) {
            data.extend_from_slice(&ser);
        }
        data
    }
}

impl Default for DelegationChain {
    fn default() -> Self {
        DelegationChain::new()
    }
}

// ============================================================================
// IDENTITY CONTEXT - Full identity embedded in PCU
// ============================================================================

/// Complete identity context for a computation
/// 
/// Key insight: This is embedded IN the PCU, not checked externally.
/// The computation carries proof of authorization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityContext {
    /// Who requested this computation
    pub principal: PrincipalId,
    
    /// What permissions they have (capability-based)
    pub capabilities: CapabilitySet,
    
    /// Delegation chain (if acting on behalf of another)
    pub delegation: Option<DelegationChain>,
    
    /// Expiry (computation becomes invalid after this)
    pub valid_until: Timestamp,
    
    /// Signature proving identity owns this context
    pub signature: Vec<u8>,
}

impl IdentityContext {
    /// Create new identity context (needs to be signed afterwards)
    pub fn new(principal: PrincipalId, capabilities: CapabilitySet) -> Self {
        IdentityContext {
            principal,
            capabilities,
            delegation: None,
            valid_until: crate::now() + 3600_000, // 1 hour default
            signature: Vec::new(),
        }
    }

    /// Create with delegation
    pub fn with_delegation(mut self, chain: DelegationChain) -> Self {
        self.delegation = Some(chain);
        self
    }

    /// Create with custom expiry
    pub fn with_expiry(mut self, valid_until: Timestamp) -> Self {
        self.valid_until = valid_until;
        self
    }

    /// Check if still valid
    pub fn is_valid(&self) -> bool {
        let now = crate::now();
        
        // Check expiry
        if now >= self.valid_until {
            return false;
        }
        
        // Check delegation chain if present
        if let Some(ref chain) = self.delegation {
            if !chain.is_valid(now) {
                return false;
            }
            // If delegated, the context principal must be the final delegatee in the chain
            if let Some(final_delegatee) = chain.final_delegatee() {
                if self.principal != final_delegatee {
                    return false;
                }
            }
        }
        
        // Check signature
        if !self.verify() {
            return false;
        }

        true
    }

    /// Verify signature of the context
    pub fn verify(&self) -> bool {
        let public_key_bytes = self.principal.as_bytes();
        let verifying_key = match VerifyingKey::from_bytes(public_key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let signature = match Signature::from_slice(&self.signature) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let data = self.signing_data();
        verifying_key.verify(&data, &signature).is_ok()
    }

    /// Sign the identity context
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), String> {
        if signing_key.verifying_key().to_bytes() != *self.principal.as_bytes() {
            return Err("Signing key does not match principal PrincipalId".to_string());
        }

        let data = self.signing_data();
        let signature = signing_key.sign(&data);
        self.signature = signature.to_bytes().to_vec();
        Ok(())
    }

    fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.principal.as_bytes());
        // Use bincode for stable serialization of complex types
        if let Ok(ser) = bincode::serialize(&(&self.capabilities, &self.delegation, self.valid_until)) {
            data.extend_from_slice(&ser);
        }
        data
    }

    /// Check if identity permits an action
    pub fn permits(&self, resource: &str, action: &str) -> bool {
        self.is_valid() && self.capabilities.permits(resource, action)
    }

    /// Get effective principal (accounting for delegation)
    pub fn effective_principal(&self) -> PrincipalId {
        if let Some(ref chain) = self.delegation {
            chain.original_principal().unwrap_or(self.principal)
        } else {
            self.principal
        }
    }

    /// Compute content hash of this identity context
    pub fn content_hash(&self) -> crate::content_hash::ContentHash {
        // Serialization should never fail for IdentityContext, but handle gracefully
        let bytes = match bincode::serialize(self) {
            Ok(b) => b,
            Err(_) => {
                // Fallback: hash just the principal if serialization fails
                let mut hasher = crate::content_hash::ContentHasher::new();
                hasher.update(self.principal.as_bytes());
                return hasher.finalize();
            }
        };
        crate::content_hash::ContentHash::compute(&bytes)
    }

    /// Create anonymous identity (minimal permissions)
    pub fn anonymous() -> Self {
        Self {
            principal: PrincipalId::anonymous(),
            capabilities: CapabilitySet::default(),
            delegation: None,
            valid_until: u64::MAX,
            signature: Vec::new(),
        }
    }

    /// Check if identity has a specific capability
    /// This is a convenience method that checks if the capability set permits the action
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_principal_generation() {
        let p1 = PrincipalId::generate();
        let p2 = PrincipalId::generate();
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_anonymous_principal() {
        let anon = PrincipalId::anonymous();
        assert!(anon.is_anonymous());
    }

    #[test]
    fn test_capability_permits() {
        let cap = Capability::new("uso:*", vec!["read".to_string(), "write".to_string()]);
        assert!(cap.permits("uso:abc123", "read"));
        assert!(cap.permits("uso:def456", "write"));
        assert!(!cap.permits("uso:abc123", "delete"));
    }

    #[test]
    fn test_capability_set() {
        let set = CapabilitySet::full_access("pcu:*");
        assert!(set.permits("pcu:execute", "read"));
        assert!(set.permits("pcu:execute", "write"));
    }

    #[test]
    fn test_identity_context_validity() {
        // Create keypair: generate random bytes, then create signing key
        use rand::RngCore;
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
        
        let mut identity = IdentityContext::new(
            principal,
            CapabilitySet::default(),
        );
        identity.sign(&signing_key).expect("Signing failed");
        assert!(identity.is_valid());
    }

    #[test]
    fn test_identity_expired() {
        let mut identity = IdentityContext::new(
            PrincipalId::generate(),
            CapabilitySet::default(),
        );
        identity.valid_until = 0; // Already expired
        assert!(!identity.is_valid());
    }
}
