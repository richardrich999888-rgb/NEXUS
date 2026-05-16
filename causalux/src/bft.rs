// Byzantine Fault Tolerant Validator
// 
// Optional module for high-security environments.
// Provides quorum-based operation validation.

use crate::causal_op::CausalOp;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Validator signature on an operation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator_id: String,
    pub operation_id: String,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

impl ValidatorSignature {
    pub fn new(validator_id: String, operation_id: String, keypair: &SigningKey) -> Self {
        let message = format!("VALIDATE:{}:{}", operation_id, validator_id);
        let signature = keypair.sign(message.as_bytes()).to_bytes().to_vec();

        Self {
            validator_id,
            operation_id,
            signature,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn verify(&self, public_key: &VerifyingKey) -> bool {
        let message = format!("VALIDATE:{}:{}", self.operation_id, self.validator_id);
        let sig = Signature::from_bytes(&self.signature.clone().try_into().unwrap_or([0u8; 64]));
        public_key.verify(message.as_bytes(), &sig).is_ok()
    }
}

/// Validator node information
#[derive(Clone, Debug)]
pub struct ValidatorInfo {
    pub id: String,
    pub public_key: VerifyingKey,
    pub priority: u8,
    pub reputation: f64,
}

/// Operation awaiting validation
#[derive(Clone, Debug)]
pub struct PendingValidation {
    pub operation: CausalOp,
    pub signatures: Vec<ValidatorSignature>,
    pub submitted_at: Instant,
    pub timeout: Duration,
}

/// BFT Validator errors
#[derive(Debug, Clone)]
pub enum BFTError {
    NotAValidator,
    UnknownValidator,
    OperationNotPending,
    AlreadyPending,
    InvalidSignature,
    DuplicateSignature,
    TimestampManipulation,
    InsufficientValidators,
}

/// Validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Operation validated with quorum
    Validated(CausalOp),
    /// Waiting for more signatures
    Pending { current: usize, required: usize },
    /// BFT disabled (pass-through)
    PassThrough,
}

/// Byzantine Fault Tolerant operation validator.
/// 
/// Requires 2f+1 honest validators to tolerate f Byzantine faults.
pub struct BFTValidator {
    validators: HashMap<String, ValidatorInfo>,
    quorum_size: usize,
    pending: HashMap<String, PendingValidation>,
    validation_timeout: Duration,
    keypair: Option<SigningKey>,
    enabled: bool,
}

impl BFTValidator {
    /// Create a new BFT validator
    pub fn new(
        validators: Vec<ValidatorInfo>,
        fault_tolerance: usize,
        validation_timeout: Duration,
    ) -> Result<Self, BFTError> {
        let quorum_size = 2 * fault_tolerance + 1;

        if validators.len() < quorum_size {
            return Err(BFTError::InsufficientValidators);
        }

        let validator_map: HashMap<String, ValidatorInfo> = validators
            .into_iter()
            .map(|v| (v.id.clone(), v))
            .collect();

        Ok(Self {
            validators: validator_map,
            quorum_size,
            pending: HashMap::new(),
            validation_timeout,
            keypair: None,
            enabled: true,
        })
    }

    /// Create a disabled (pass-through) BFT validator
    pub fn disabled() -> Self {
        Self {
            validators: HashMap::new(),
            quorum_size: 0,
            pending: HashMap::new(),
            validation_timeout: Duration::from_secs(0),
            keypair: None,
            enabled: false,
        }
    }

    /// Check if BFT is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set this node as a validator
    pub fn set_validator_keypair(&mut self, keypair: SigningKey) {
        self.keypair = Some(keypair);
    }

    /// Submit operation for validation
    pub fn submit_for_validation(&mut self, op: CausalOp) -> Result<(), BFTError> {
        if !self.enabled {
            return Ok(());
        }

        if self.pending.contains_key(&op.id) {
            return Err(BFTError::AlreadyPending);
        }

        self.validate_operation_structure(&op)?;

        self.pending.insert(
            op.id.clone(),
            PendingValidation {
                operation: op,
                signatures: vec![],
                submitted_at: Instant::now(),
                timeout: self.validation_timeout,
            },
        );

        Ok(())
    }

    /// Sign an operation as a validator
    pub fn sign_operation(&self, operation_id: &str) -> Result<ValidatorSignature, BFTError> {
        let keypair = self.keypair.as_ref().ok_or(BFTError::NotAValidator)?;
        let validator_id = Self::derive_identity(&keypair.verifying_key());

        if !self.validators.contains_key(&validator_id) {
            return Err(BFTError::NotAValidator);
        }

        Ok(ValidatorSignature::new(
            validator_id,
            operation_id.to_string(),
            keypair,
        ))
    }

    /// Add a validator signature
    pub fn add_validator_signature(
        &mut self,
        signature: ValidatorSignature,
    ) -> Result<ValidationResult, BFTError> {
        if !self.enabled {
            return Ok(ValidationResult::PassThrough);
        }

        let pending = self
            .pending
            .get_mut(&signature.operation_id)
            .ok_or(BFTError::OperationNotPending)?;

        let validator = self
            .validators
            .get(&signature.validator_id)
            .ok_or(BFTError::UnknownValidator)?;

        if !signature.verify(&validator.public_key) {
            self.penalize_validator(&signature.validator_id);
            return Err(BFTError::InvalidSignature);
        }

        if pending.signatures.iter().any(|s| s.validator_id == signature.validator_id) {
            return Err(BFTError::DuplicateSignature);
        }

        pending.signatures.push(signature);

        if pending.signatures.len() >= self.quorum_size {
            let op = pending.operation.clone();
            self.pending.remove(&op.id);
            return Ok(ValidationResult::Validated(op));
        }

        Ok(ValidationResult::Pending {
            current: pending.signatures.len(),
            required: self.quorum_size,
        })
    }

    /// Check for timed-out validations
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let mut timed_out = vec![];

        self.pending.retain(|op_id, pending| {
            if pending.submitted_at.elapsed() > pending.timeout {
                timed_out.push(op_id.clone());
                false
            } else {
                true
            }
        });

        timed_out
    }

    fn validate_operation_structure(&self, op: &CausalOp) -> Result<(), BFTError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Check for future timestamps (clock manipulation)
        if op.wall_clock > now + 300_000 {
            return Err(BFTError::TimestampManipulation);
        }

        Ok(())
    }

    fn penalize_validator(&mut self, validator_id: &str) {
        if let Some(validator) = self.validators.get_mut(validator_id) {
            validator.reputation *= 0.9;

            if validator.reputation < 0.5 {
                self.validators.remove(validator_id);
                eprintln!("⚠️ Validator {} removed (low reputation)", validator_id);
            }
        }
    }

    fn derive_identity(public_key: &VerifyingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&public_key.to_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get pending operation count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get quorum size
    pub fn quorum_size(&self) -> usize {
        self.quorum_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version_vector::VersionVector;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn create_test_keypair() -> SigningKey {
        let mut bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    fn create_test_validator(keypair: &SigningKey) -> ValidatorInfo {
        ValidatorInfo {
            id: BFTValidator::derive_identity(&keypair.verifying_key()),
            public_key: keypair.verifying_key().clone(),
            priority: 1,
            reputation: 1.0,
        }
    }


    #[test]
    fn test_bft_creation() {
        let keypairs: Vec<SigningKey> = (0..4).map(|_| create_test_keypair()).collect();
        let validators: Vec<ValidatorInfo> = keypairs
            .iter()
            .map(|kp| create_test_validator(kp))
            .collect();

        let bft = BFTValidator::new(validators, 1, Duration::from_secs(30)).unwrap();

        assert_eq!(bft.quorum_size(), 3); // 2f+1 = 3 for f=1
        assert!(bft.is_enabled());
    }

    #[test]
    fn test_insufficient_validators() {
        let keypairs: Vec<SigningKey> = (0..2).map(|_| create_test_keypair()).collect();
        let validators: Vec<ValidatorInfo> = keypairs
            .iter()
            .map(|kp| create_test_validator(kp))
            .collect();

        let result = BFTValidator::new(validators, 1, Duration::from_secs(30));
        assert!(matches!(result, Err(BFTError::InsufficientValidators)));
    }

    #[test]
    fn test_disabled_bft() {
        let bft = BFTValidator::disabled();
        assert!(!bft.is_enabled());
    }

    #[test]
    fn test_validator_signature() {
        let keypair = create_test_keypair();
        let sig = ValidatorSignature::new(
            "validator1".to_string(),
            "op123".to_string(),
            &keypair,
        );

        assert!(sig.verify(&keypair.verifying_key()));
    }
}
