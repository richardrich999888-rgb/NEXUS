//! ETK Event Chain. Append-only, hash-chained execution events.
//!
//! Invariants: sequence_number(n) = sequence_number(n-1)+1,
//! previous_event_hash(n) = event_id(n-1), same execution_id, immutable policy_ref/jurisdiction_code.

use crate::genesis::{create_genesis, is_genesis};
use crate::schema::{ExecutionEventV1, ExecutionProofV1, Hash256, OutcomeCode, ResourceClass};
use ed25519_dalek::{Signature, Signer, SigningKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("Genesis must be first event")]
    GenesisNotFirst,
    #[error("Invalid sequence: expected {expected}, got {got}")]
    InvalidSequence { expected: u64, got: u64 },
    #[error("Hash chain broken: previous_event_hash does not match prior event_id")]
    HashChainBroken,
    #[error("execution_id mismatch")]
    ExecutionIdMismatch,
    #[error("policy_ref mismatch")]
    PolicyRefMismatch,
    #[error("jurisdiction_code mismatch")]
    JurisdictionMismatch,
    #[error("Execution already terminated")]
    AlreadyTerminated,
}

/// Append-only event chain for one execution lifecycle.
pub struct EventChain {
    events: Vec<ExecutionEventV1>,
}

impl EventChain {
    /// Start a new execution with Genesis. Only way to create a chain.
    pub fn new(
        actor_id: Hash256,
        workload_id: Hash256,
        execution_context: Hash256,
        resource_class: ResourceClass,
        jurisdiction_code: u16,
        policy_ref: Hash256,
    ) -> Self {
        let genesis = create_genesis(
            actor_id,
            workload_id,
            execution_context,
            resource_class,
            jurisdiction_code,
            policy_ref,
        );
        Self {
            events: vec![genesis],
        }
    }

    /// Append next event. Enforces sequence, hash chain, and immutability of execution_id/policy_ref/jurisdiction.
    pub fn append(
        &mut self,
        timestamp_utc: u64,
        outcome_code: OutcomeCode,
    ) -> Result<&ExecutionEventV1, ChainError> {
        if let Some(last) = self.events.last() {
            if last.outcome_code.is_terminal() {
                return Err(ChainError::AlreadyTerminated);
            }
        }

        let prev = self.events.last().expect("chain has genesis");
        let expected_seq = prev.sequence_number + 1;

        let mut ev = ExecutionEventV1 {
            event_id: Hash256::zero(),
            execution_id: prev.execution_id,
            sequence_number: expected_seq,
            timestamp_utc,
            actor_id: prev.actor_id,
            workload_id: prev.workload_id,
            execution_context: prev.execution_context,
            resource_class: prev.resource_class,
            jurisdiction_code: prev.jurisdiction_code,
            policy_ref: prev.policy_ref,
            outcome_code,
            previous_event_hash: prev.event_id,
        };
        ev.event_id = ev.compute_event_id();

        if ev.execution_id != prev.execution_id {
            return Err(ChainError::ExecutionIdMismatch);
        }
        if ev.policy_ref != prev.policy_ref {
            return Err(ChainError::PolicyRefMismatch);
        }
        if ev.jurisdiction_code != prev.jurisdiction_code {
            return Err(ChainError::JurisdictionMismatch);
        }
        if ev.previous_event_hash.0 != prev.event_id.0 {
            return Err(ChainError::HashChainBroken);
        }

        self.events.push(ev.clone());
        Ok(self.events.last().unwrap())
    }

    /// Finalize chain and produce ExecutionProof_v1. Signs with verifier key.
    pub fn finalize(&self, signing_key: &SigningKey) -> Result<ExecutionProofV1, ChainError> {
        let genesis = self.events.first().ok_or(ChainError::GenesisNotFirst)?;
        if !is_genesis(genesis) {
            return Err(ChainError::GenesisNotFirst);
        }

        let last = self.events.last().unwrap();
        let start_timestamp = genesis.timestamp_utc;
        let end_timestamp = last.timestamp_utc;

        let mut proof = ExecutionProofV1 {
            execution_id: genesis.execution_id,
            event_chain_root: last.event_id,
            start_timestamp,
            end_timestamp,
            policy_ref: genesis.policy_ref,
            jurisdiction_code: genesis.jurisdiction_code,
            verifier_signature: [0u8; 64],
        };

        let signing_bytes = proof.to_signing_bytes();
        let sig: Signature = signing_key.sign(&signing_bytes);
        proof.verifier_signature = sig.to_bytes();

        Ok(proof)
    }

    pub fn events(&self) -> &[ExecutionEventV1] {
        &self.events
    }

    pub fn execution_id(&self) -> Option<Hash256> {
        self.events.first().map(|e| e.execution_id)
    }

    pub fn is_terminal(&self) -> bool {
        self.events
            .last()
            .map(|e| e.outcome_code.is_terminal())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ResourceClass;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn chain_append_and_finalize() {
        let actor = Hash256::of(b"actor");
        let workload = Hash256::of(b"workload");
        let ctx = Hash256::of(b"context");
        let policy = Hash256::of(b"policy");
        let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
        let t1 = 1_000_000u64;
        let t2 = 1_000_100u64;
        chain.append(t1, OutcomeCode::Unknown).unwrap();
        chain.append(t2, OutcomeCode::Success).unwrap();
        assert!(chain.is_terminal());
        let key = SigningKey::generate(&mut OsRng);
        let proof = chain.finalize(&key).unwrap();
        assert_eq!(proof.execution_id, chain.execution_id().unwrap());
        assert_eq!(proof.jurisdiction_code, 840);
    }

    #[test]
    fn chain_rejects_append_after_terminal() {
        let actor = Hash256::of(b"a");
        let workload = Hash256::of(b"w");
        let ctx = Hash256::of(b"c");
        let policy = Hash256::of(b"p");
        let mut chain = EventChain::new(actor, workload, ctx, ResourceClass::Cpu, 840, policy);
        chain.append(1000, OutcomeCode::Success).unwrap();
        let err = chain.append(1001, OutcomeCode::Unknown).unwrap_err();
        assert!(matches!(err, ChainError::AlreadyTerminated));
    }
}

