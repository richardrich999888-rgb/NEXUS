//! ETK event chain. Append-only, hash-chained. Invariants enforced.

use etk_types::{ExecutionEvent, ExecutionProof, Hash256, OutcomeCode, ResourceClass};
use crate::codec::compute_event_id;
use crate::genesis::{create_genesis, is_genesis};
use ed25519_dalek::{Signature, Signer, SigningKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("Genesis must be first event")]
    GenesisNotFirst,
    #[error("Hash chain broken")]
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
    events: Vec<ExecutionEvent>,
}

impl EventChain {
    /// Start a new execution with Genesis.
    pub fn new(
        actor: Hash256,
        workload: Hash256,
        ctx: Hash256,
        resource: ResourceClass,
        jurisdiction: u16,
        policy: Hash256,
    ) -> Self {
        let genesis = create_genesis(actor, workload, ctx, resource, jurisdiction, policy);
        Self {
            events: vec![genesis],
        }
    }

    /// Append next event. Enforces sequence, hash chain, immutability of execution_id/policy/jurisdiction.
    pub fn append(
        &mut self,
        timestamp_utc: u64,
        outcome_code: OutcomeCode,
    ) -> Result<&ExecutionEvent, ChainError> {
        if let Some(last) = self.events.last() {
            if last.outcome_code.is_terminal() {
                return Err(ChainError::AlreadyTerminated);
            }
        }

        let prev = self.events.last().expect("chain has genesis");
        let mut ev = ExecutionEvent {
            event_id: Hash256::zero(),
            execution_id: prev.execution_id,
            sequence_number: prev.sequence_number + 1,
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
        ev.event_id = compute_event_id(&ev);

        if ev.previous_event_hash.0 != prev.event_id.0 {
            return Err(ChainError::HashChainBroken);
        }
        self.events.push(ev);
        Ok(self.events.last().unwrap())
    }

    /// Finalize and produce signed ExecutionProof.
    pub fn finalize(&self, signing_key: &SigningKey) -> Result<ExecutionProof, ChainError> {
        let genesis = self.events.first().ok_or(ChainError::GenesisNotFirst)?;
        if !is_genesis(genesis) {
            return Err(ChainError::GenesisNotFirst);
        }
        let last = self.events.last().unwrap();
        let mut proof = ExecutionProof {
            execution_id: genesis.execution_id,
            event_chain_root: last.event_id,
            start_timestamp: genesis.timestamp_utc,
            end_timestamp: last.timestamp_utc,
            policy_ref: genesis.policy_ref,
            jurisdiction_code: genesis.jurisdiction_code,
            verifier_signature: [0u8; 64],
        };
        let signing_bytes = crate::codec::encode_proof_signing_bytes(&proof);
        let sig: Signature = signing_key.sign(&signing_bytes);
        proof.verifier_signature = sig.to_bytes();
        Ok(proof)
    }

    pub fn events(&self) -> &[ExecutionEvent] {
        &self.events
    }

    pub fn execution_id(&self) -> Option<Hash256> {
        self.events.first().map(|e| e.execution_id)
    }
}
