//! Verifiable Delay Function (VDF) for entropy generation.
//!
//! Implements a simplified VDF based on iterated hashing.
//! In production, this would use a proper Wesolowski or Pietrzak construction.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::{Instant, Duration};

/// VDF configuration parameters.
#[derive(Debug, Clone)]
pub struct VdfConfig {
    /// Minimum steps required per unit of consequence.
    pub steps_per_tier: u64,
    /// Maximum allowed computation time.
    pub max_duration: Duration,
    /// Hash function iterations per step.
    pub iterations_per_step: u32,
}

impl Default for VdfConfig {
    fn default() -> Self {
        Self {
            steps_per_tier: 10_000,
            max_duration: Duration::from_secs(60),
            iterations_per_step: 100,
        }
    }
}

/// A VDF proof that demonstrates sequential computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdfProof {
    /// Input seed (challenge).
    pub input: Vec<u8>,
    /// Final output after sequential hashing.
    pub output: Vec<u8>,
    /// Number of iterations performed.
    pub iterations: u64,
    /// Intermediate checkpoints for faster verification.
    pub checkpoints: Vec<VdfCheckpoint>,
    /// Time taken to compute (for informational purposes).
    pub compute_time_ms: u64,
    /// Timestamp when computation started.
    pub started_at: DateTime<Utc>,
    /// Timestamp when computation completed.
    pub completed_at: DateTime<Utc>,
}

/// A checkpoint in the VDF computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdfCheckpoint {
    /// Iteration number at this checkpoint.
    pub iteration: u64,
    /// Hash value at this checkpoint.
    pub hash: [u8; 32],
}

/// VDF generator.
#[derive(Debug, Clone)]
pub struct VdfGenerator {
    config: VdfConfig,
    /// Checkpoint interval (every N iterations).
    checkpoint_interval: u64,
}

impl VdfGenerator {
    /// Create a new VDF generator with default config.
    pub fn new() -> Self {
        Self::with_config(VdfConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(config: VdfConfig) -> Self {
        Self {
            config,
            checkpoint_interval: 1000,
        }
    }

    /// Set checkpoint interval.
    pub fn with_checkpoint_interval(mut self, interval: u64) -> Self {
        self.checkpoint_interval = interval;
        self
    }

    /// Compute VDF proof for given consequence tier.
    pub fn compute(&self, seed: &[u8], consequence_tier: u8) -> VdfProof {
        let iterations = self.config.steps_per_tier * (consequence_tier as u64).max(1);
        self.compute_iterations(seed, iterations)
    }

    /// Compute VDF with specific iteration count.
    pub fn compute_iterations(&self, seed: &[u8], iterations: u64) -> VdfProof {
        let started_at = Utc::now();
        let start_time = Instant::now();
        
        let mut current = seed.to_vec();
        let mut checkpoints = Vec::new();

        for i in 0..iterations {
            // Perform hash iteration
            let mut hasher = Sha256::new();
            hasher.update(&current);
            hasher.update(i.to_le_bytes());
            current = hasher.finalize().to_vec();

            // Record checkpoint
            if i > 0 && i % self.checkpoint_interval == 0 {
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&current);
                checkpoints.push(VdfCheckpoint {
                    iteration: i,
                    hash,
                });
            }

            // Check timeout
            if start_time.elapsed() > self.config.max_duration {
                break;
            }
        }

        let completed_at = Utc::now();
        let compute_time_ms = start_time.elapsed().as_millis() as u64;

        VdfProof {
            input: seed.to_vec(),
            output: current,
            iterations,
            checkpoints,
            compute_time_ms,
            started_at,
            completed_at,
        }
    }

    /// Get required iterations for a consequence tier.
    pub fn required_iterations(&self, consequence_tier: u8) -> u64 {
        self.config.steps_per_tier * (consequence_tier as u64).max(1)
    }
}

impl Default for VdfGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// VDF verifier.
#[derive(Debug, Clone)]
pub struct VdfVerifier {
    /// Checkpoint verification enabled.
    verify_checkpoints: bool,
    /// Maximum age of proof in seconds.
    max_proof_age_secs: u64,
}

impl VdfVerifier {
    /// Create a new verifier.
    pub fn new() -> Self {
        Self {
            verify_checkpoints: true,
            max_proof_age_secs: 300, // 5 minutes
        }
    }

    /// Verify a VDF proof completely (recompute everything).
    pub fn verify_full(&self, proof: &VdfProof) -> VdfVerificationResult {
        let start = Instant::now();
        
        // Check proof age
        let age = (Utc::now() - proof.completed_at).num_seconds();
        if age > self.max_proof_age_secs as i64 {
            return VdfVerificationResult {
                valid: false,
                reason: Some("Proof too old".into()),
                verify_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Recompute the VDF
        let mut current = proof.input.clone();
        
        for i in 0..proof.iterations {
            let mut hasher = Sha256::new();
            hasher.update(&current);
            hasher.update(i.to_le_bytes());
            current = hasher.finalize().to_vec();

            // Verify checkpoints if enabled
            if self.verify_checkpoints {
                if let Some(checkpoint) = proof.checkpoints.iter().find(|c| c.iteration == i) {
                    if current != checkpoint.hash {
                        return VdfVerificationResult {
                            valid: false,
                            reason: Some(format!("Checkpoint mismatch at iteration {}", i)),
                            verify_time_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
            }
        }

        // Check final output
        if current != proof.output {
            return VdfVerificationResult {
                valid: false,
                reason: Some("Final output mismatch".into()),
                verify_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        VdfVerificationResult {
            valid: true,
            reason: None,
            verify_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Quick verification using checkpoints (O(n/checkpoint_interval) instead of O(n)).
    pub fn verify_quick(&self, proof: &VdfProof) -> VdfVerificationResult {
        let start = Instant::now();

        // Check proof age
        let age = (Utc::now() - proof.completed_at).num_seconds();
        if age > self.max_proof_age_secs as i64 {
            return VdfVerificationResult {
                valid: false,
                reason: Some("Proof too old".into()),
                verify_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Must have at least one checkpoint for quick verification
        if proof.checkpoints.is_empty() {
            return self.verify_full(proof);
        }

        // Verify first checkpoint from input
        let first_checkpoint = &proof.checkpoints[0];
        let mut current = proof.input.clone();
        
        // Compute up to AND including the checkpoint iteration
        for i in 0..=first_checkpoint.iteration {
            let mut hasher = Sha256::new();
            hasher.update(&current);
            hasher.update(i.to_le_bytes());
            current = hasher.finalize().to_vec();
        }

        if current != first_checkpoint.hash {
            return VdfVerificationResult {
                valid: false,
                reason: Some("First checkpoint verification failed".into()),
                verify_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Verify last checkpoint to output
        let last_checkpoint = proof.checkpoints.last().unwrap();
        current = last_checkpoint.hash.to_vec();
        
        // Continue from iteration AFTER the checkpoint (checkpoint was recorded after iteration completed)
        for i in (last_checkpoint.iteration + 1)..proof.iterations {
            let mut hasher = Sha256::new();
            hasher.update(&current);
            hasher.update(i.to_le_bytes());
            current = hasher.finalize().to_vec();
        }

        if current != proof.output {
            return VdfVerificationResult {
                valid: false,
                reason: Some("Last segment verification failed".into()),
                verify_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        VdfVerificationResult {
            valid: true,
            reason: None,
            verify_time_ms: start.elapsed().as_millis() as u64,
        }
    }
}

impl Default for VdfVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of VDF verification.
#[derive(Debug, Clone)]
pub struct VdfVerificationResult {
    /// Whether the proof is valid.
    pub valid: bool,
    /// Reason for invalidity (if any).
    pub reason: Option<String>,
    /// Time taken to verify.
    pub verify_time_ms: u64,
}

/// Convert VDF proof to entropy proof for the TELOS protocol.
pub fn vdf_to_entropy_proof(vdf_proof: VdfProof, consequence_tier: u8) -> crate::entropy::EntropyProof {
    use crate::entropy::{EntropyProof, EntropySourceType, EntropyProofData};
    
    // Entropy amount is proportional to iterations
    let amount = vdf_proof.iterations * 10;
    
    let proof_data = EntropyProofData::VDF {
        input: vdf_proof.input,
        output: vdf_proof.output,
        steps: vdf_proof.iterations,
    };

    EntropyProof::new(EntropySourceType::VDF, amount, proof_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdf_compute_basic() {
        let generator = VdfGenerator::with_config(VdfConfig {
            steps_per_tier: 100,
            max_duration: Duration::from_secs(10),
            iterations_per_step: 1,
        }).with_checkpoint_interval(10);

        let seed = b"test_seed_123";
        let proof = generator.compute(seed, 1);

        assert_eq!(proof.iterations, 100);
        assert!(!proof.output.is_empty());
        assert!(proof.compute_time_ms < 5000);
    }

    #[test]
    fn test_vdf_verify_full() {
        let generator = VdfGenerator::with_config(VdfConfig {
            steps_per_tier: 50,
            max_duration: Duration::from_secs(10),
            iterations_per_step: 1,
        }).with_checkpoint_interval(10);

        let seed = b"verify_test";
        let proof = generator.compute(seed, 1);

        let verifier = VdfVerifier::new();
        let result = verifier.verify_full(&proof);

        assert!(result.valid, "Proof should be valid: {:?}", result.reason);
    }

    #[test]
    fn test_vdf_verify_quick() {
        let generator = VdfGenerator::with_config(VdfConfig {
            steps_per_tier: 100,
            max_duration: Duration::from_secs(10),
            iterations_per_step: 1,
        }).with_checkpoint_interval(20);

        let seed = b"quick_verify";
        let proof = generator.compute(seed, 1);

        let verifier = VdfVerifier::new();
        let result = verifier.verify_quick(&proof);

        assert!(result.valid, "Quick verify should pass: {:?}", result.reason);
    }

    #[test]
    fn test_vdf_deterministic() {
        let generator = VdfGenerator::with_config(VdfConfig {
            steps_per_tier: 50,
            ..Default::default()
        });

        let seed = b"determinism";
        let proof1 = generator.compute(seed, 1);
        let proof2 = generator.compute(seed, 1);

        assert_eq!(proof1.output, proof2.output);
    }

    #[test]
    fn test_vdf_different_tiers() {
        let generator = VdfGenerator::with_config(VdfConfig {
            steps_per_tier: 100,
            ..Default::default()
        });

        let seed = b"tier_test";
        let proof1 = generator.compute(seed, 1);
        let proof5 = generator.compute(seed, 5);

        assert_eq!(proof1.iterations, 100);
        assert_eq!(proof5.iterations, 500);
    }

    #[test]
    fn test_vdf_to_entropy_proof() {
        let generator = VdfGenerator::with_config(VdfConfig {
            steps_per_tier: 100,
            ..Default::default()
        });

        let seed = b"entropy_test";
        let vdf_proof = generator.compute(seed, 2);
        let entropy_proof = vdf_to_entropy_proof(vdf_proof.clone(), 2);

        assert_eq!(entropy_proof.amount, vdf_proof.iterations * 10);
    }
}
