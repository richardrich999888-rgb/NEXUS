use crate::hash::Hash;
use crate::op::Operation;
use crate::errors::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub operation: Operation,
    pub output: Vec<u8>,
    pub proof: Hash,
}

impl LogEntry {
    pub fn new(operation: Operation, output: Vec<u8>) -> Self {
        let proof = Self::compute_proof(&operation, &output);
        LogEntry { operation, output, proof }
    }

    pub fn id(&self) -> Hash {
        let mut data = self.operation.serialize();
        data.extend_from_slice(&self.output);
        data.extend_from_slice(self.proof.as_bytes());
        Hash::of(&data)
    }

    pub fn verify(&self) -> bool {
        let expected = Self::compute_proof(&self.operation, &self.output);
        self.proof == expected
    }

    fn compute_proof(operation: &Operation, output: &[u8]) -> Hash {
        let mut data = operation.serialize();
        data.extend_from_slice(output);
        Hash::of(&data)
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| crate::errors::NexusError::StorageError(e.to_string()))
    }
}
