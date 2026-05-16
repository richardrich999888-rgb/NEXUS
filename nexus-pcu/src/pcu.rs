// NEXUS PCU: Portable Computation Unit
// Copyright (c) 2025 SYNTRIASS Labs Private Limited
// Inventor: Katta Naga Sri Ganesh
//
// The core innovation: computation that carries its own identity,
// routes to where data lives, and generates cryptographic proofs.

use serde::{Deserialize, Serialize};


use crate::identity::IdentityContext;
use crate::proof::ExecutionProof;
use crate::content_hash::ContentHash;
use crate::Timestamp;

// ============================================================================
// WASM MODULE - Executable code in a PCU
// ============================================================================

/// WASM bytecode with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModule {
    /// Raw WASM bytecode
    pub bytecode: Vec<u8>,
    /// Content hash of bytecode (for deduplication)
    pub hash: ContentHash,
    /// Optional symbolic name
    pub name: Option<String>,
    /// Expected memory pages (for resource limits)
    pub memory_pages: u32,
}

impl WasmModule {
    /// Create from raw bytecode
    pub fn new(bytecode: Vec<u8>) -> Self {
        let hash = ContentHash::compute(&bytecode);
        WasmModule {
            bytecode,
            hash,
            name: None,
            memory_pages: 16, // 1MB default
        }
    }

    /// Create with name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Create with memory limit
    pub fn with_memory(mut self, pages: u32) -> Self {
        self.memory_pages = pages;
        self
    }

    /// Get content hash of the bytecode
    pub fn content_hash(&self) -> ContentHash {
        self.hash
    }

    /// Check if module has valid WASM header
    pub fn is_valid_header(&self) -> bool {
        // WASM magic number: 0x00 0x61 0x73 0x6D ("\0asm")
        self.bytecode.len() >= 8
            && self.bytecode[0..4] == [0x00, 0x61, 0x73, 0x6D]
    }

    /// Get bytecode size
    pub fn size(&self) -> usize {
        self.bytecode.len()
    }
}

// ============================================================================
// EXECUTION CONSTRAINTS - Where and how a PCU can execute
// ============================================================================

/// Constraints on PCU execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConstraints {
    /// Required node capabilities (e.g., "gpu", "sgx", "arm64")
    pub required_capabilities: Vec<String>,
    /// Maximum execution time in milliseconds
    pub max_duration_ms: u64,
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,
    /// Allowed regions for execution (empty = any)
    pub allowed_regions: Vec<String>,
    /// Whether execution can be parallelized
    pub allow_parallel: bool,
    /// Priority (higher = more urgent)
    pub priority: u8,
}

impl Default for ExecutionConstraints {
    fn default() -> Self {
        ExecutionConstraints {
            required_capabilities: Vec::new(),
            max_duration_ms: 30_000,      // 30 seconds
            max_memory_bytes: 256 * 1024 * 1024, // 256 MB
            allowed_regions: Vec::new(),
            allow_parallel: false,
            priority: 5,
        }
    }
}

// ============================================================================
// PCU - Portable Computation Unit
// ============================================================================

/// Portable Computation Unit
/// 
/// A self-contained computation that includes:
/// - Code: The function to execute (WASM bytecode)
/// - Data references: Content-addressed pointers to required data
/// - Identity context: Who requested this, with what permissions
/// - Execution constraints: Where/how this can execute
/// - Execution proof: After execution, cryptographic proof of correctness
///
/// Key innovation: PCU routes TO where data lives, not the other way around.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCU {
    /// Unique identifier (hash of code + inputs + identity)
    pub id: ContentHash,
    
    /// The WASM module to execute
    pub code: WasmModule,
    
    /// Content-addressed references to input data
    /// PCU routes to node where these are locally available
    pub inputs: Vec<ContentHash>,
    
    /// Additional input parameters (inline, not content-addressed)
    pub parameters: Vec<u8>,
    
    /// Identity context (intrinsic, not external)
    pub identity: IdentityContext,
    
    /// Execution constraints
    pub constraints: ExecutionConstraints,
    
    /// When this PCU was created
    pub created_at: Timestamp,
    
    /// Execution result (populated after execution)
    pub result: Option<PCUResult>,
}

/// Result of PCU execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PCUResult {
    /// Output data
    pub output: Vec<u8>,
    /// Content hash of output
    pub output_hash: ContentHash,
    /// Execution duration in microseconds
    pub duration_us: u64,
    /// Memory used in bytes
    pub memory_used: u64,
    /// Cryptographic proof of correct execution
    pub proof: ExecutionProof,
}

impl PCU {
    /// Create a new PCU
    pub fn new(
        code: WasmModule,
        inputs: Vec<ContentHash>,
        parameters: Vec<u8>,
        identity: IdentityContext,
    ) -> Self {
        let constraints = ExecutionConstraints::default();
        let created_at = crate::now();
        
        // Compute PCU ID from deterministic components
        let id = Self::compute_id(&code, &inputs, &parameters, &identity);
        
        PCU {
            id,
            code,
            inputs,
            parameters,
            identity,
            constraints,
            created_at,
            result: None,
        }
    }

    /// Create with custom constraints
    pub fn with_constraints(mut self, constraints: ExecutionConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Compute deterministic PCU ID
    fn compute_id(
        code: &WasmModule,
        inputs: &[ContentHash],
        parameters: &[u8],
        identity: &IdentityContext,
    ) -> ContentHash {
        let mut hasher = crate::content_hash::ContentHasher::new();
        
        // Hash code
        hasher.update(code.hash.as_bytes());
        
        // Hash inputs
        for input in inputs {
            hasher.update(input.as_bytes());
        }
        
        // Hash parameters
        hasher.update(parameters);
        
        // Hash identity (principal ID)
        hasher.update(identity.principal.as_bytes());
        
        ContentHash(*hasher.finalize().as_bytes())
    }

    /// Check if PCU has been executed
    pub fn is_executed(&self) -> bool {
        self.result.is_some()
    }

    /// Check if PCU identity is still valid
    pub fn is_valid(&self) -> bool {
        self.identity.is_valid()
    }

    /// Serialize PCU to bytes
    ///
    /// # Errors
    ///
    /// Returns error if serialization fails
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize PCU from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Get total size of inputs (for routing decisions)
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Compute content hash of this PCU
    ///
    /// The hash covers code, inputs, and parameters (not identity,
    /// as different identities may execute the same computation).
    /// Used for integrity and data locality routing.
    pub fn content_hash(&self) -> ContentHash {
        self.id
    }

    /// Compute semantic hash for result caching
    ///
    /// The semantic hash includes identity, as different principals
    /// may have different views or permissions even for the same code/inputs.
    pub fn semantic_hash(&self) -> ContentHash {
        let mut hasher = crate::content_hash::ContentHasher::new();
        
        // 1. Code hash
        hasher.update(self.code.hash.as_bytes());
        
        // 2. Inputs (sorted for determinism)
        let mut sorted_inputs = self.inputs.clone();
        sorted_inputs.sort_by_key(|h| *h.as_bytes());
        for input in sorted_inputs {
            hasher.update(input.as_bytes());
        }
        
        // 3. Parameters
        hasher.update(&self.parameters);
        
        // 4. Identity (Principal ID is the key part for semantic separation)
        hasher.update(self.identity.principal.as_bytes());
        
        hasher.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{PrincipalId, CapabilitySet};

    #[test]
    fn test_content_hash_determinism() {
        let data = b"hello world";
        let h1 = ContentHash::compute(data);
        let h2 = ContentHash::compute(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_pcu_creation() {
        use rand::RngCore;
        use ed25519_dalek::SigningKey;
        
        let code = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d]); // WASM magic
        
        // Create a properly signed identity
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let principal = PrincipalId::from_bytes(signing_key.verifying_key().to_bytes());
        
        let mut identity = IdentityContext::new(principal, CapabilitySet::default());
        identity.sign(&signing_key).expect("Signing failed");
        
        let pcu = PCU::new(code, vec![], vec![1, 2, 3], identity);
        
        assert!(!pcu.is_executed());
        assert!(pcu.is_valid());
    }

    #[test]
    fn test_pcu_id_determinism() {
        let code = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d]);
        let principal = PrincipalId::from_bytes([1u8; 32]);
        let identity = IdentityContext::new(principal, CapabilitySet::default());
        
        let pcu1 = PCU::new(code.clone(), vec![], vec![1, 2, 3], identity.clone());
        let pcu2 = PCU::new(code, vec![], vec![1, 2, 3], identity);
        
        assert_eq!(pcu1.id, pcu2.id);
    }

    #[test]
    fn test_pcu_serialization() {
        let code = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d]);
        let identity = IdentityContext::new(
            PrincipalId::generate(),
            CapabilitySet::default(),
        );
        
        let pcu = PCU::new(code, vec![], vec![], identity);
        let bytes = pcu.to_bytes().unwrap();
        let restored = PCU::from_bytes(&bytes).unwrap();
        
        assert_eq!(pcu.id, restored.id);
    }
}
