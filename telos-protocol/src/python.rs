//! Python FFI for the TELOS protocol.
//!
//! Build with:
//! `maturin develop --features python --manifest-path telos-protocol/Cargo.toml`

use crate::authority::{AgentId, AuthorityRegistry, DecisionDomain};
use crate::entropy::{
    ConsequenceTier, EntropyMeter, EntropyProof, EntropyProofData, EntropySourceType,
};
use crate::membrane::{CommitmentMembrane, CrossingResult, Decision};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn parse_tier(tier: u8) -> PyResult<ConsequenceTier> {
    ConsequenceTier::from_u8(tier)
        .ok_or_else(|| PyValueError::new_err("tier must be in the range 1..=5"))
}

#[pyfunction]
fn entropy_cost(tier: u8, budget: u64, trust_score: f64) -> PyResult<u64> {
    let tier = parse_tier(tier)?;
    let meter = EntropyMeter::new(budget);
    Ok(meter.calculate_cost(tier, trust_score))
}

#[pyfunction]
fn decision_hash(domain: &str, action: &str, tier: u8) -> PyResult<String> {
    let tier = parse_tier(tier)?;
    let decision = Decision::new(domain, action, tier);
    Ok(hex::encode(decision.hash()))
}

#[pyfunction]
#[pyo3(signature = (domain, action, tier, agent_id, budget, trust_score=0.5))]
fn commit_single_node<'py>(
    py: Python<'py>,
    domain: &str,
    action: &str,
    tier: u8,
    agent_id: &str,
    budget: u64,
    trust_score: f64,
) -> PyResult<&'py PyDict> {
    let tier = parse_tier(tier)?;
    let agent = AgentId::new(agent_id);
    let mut membrane = CommitmentMembrane::new();
    let mut entropy = EntropyMeter::new(budget);
    let mut authority = AuthorityRegistry::new();

    authority
        .create_root_authority(
            agent.clone(),
            vec![DecisionDomain::new("*", ConsequenceTier::Critical)],
            budget,
            1,
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let decision = Decision::new(domain, action, tier);
    let decision_id = membrane
        .add_decision(decision)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let cost = entropy.calculate_cost(tier, trust_score);
    let proof = EntropyProof::new(
        EntropySourceType::Beacon,
        cost,
        EntropyProofData::Beacon {
            beacon_id: "python-ffi-local".to_string(),
            round: 0,
            randomness: decision_id.as_bytes().to_vec(),
        },
    );

    let crossing = membrane
        .request_crossing(
            &decision_id,
            &agent,
            &mut entropy,
            proof,
            &authority,
            trust_score,
        )
        .map_err(|err| PyValueError::new_err(err.to_string()))?;

    let out = PyDict::new_bound(py);
    match crossing {
        CrossingResult::Committed {
            decision_id,
            committed_at,
            entropy_consumed,
            commitment_hash,
        } => {
            out.set_item("status", "committed")?;
            out.set_item("decision_id", decision_id)?;
            out.set_item("committed_at", committed_at.to_rfc3339())?;
            out.set_item("entropy_consumed", entropy_consumed)?;
            out.set_item("commitment_hash", hex::encode(commitment_hash))?;
        }
        CrossingResult::Rejected {
            decision_id,
            reason,
        } => {
            out.set_item("status", "rejected")?;
            out.set_item("decision_id", decision_id)?;
            out.set_item("reason", reason)?;
        }
        CrossingResult::PendingValidation {
            decision_id,
            attestations_received,
            attestations_needed,
        } => {
            out.set_item("status", "pending_validation")?;
            out.set_item("decision_id", decision_id)?;
            out.set_item("attestations_received", attestations_received)?;
            out.set_item("attestations_needed", attestations_needed)?;
        }
    }
    Ok(out.into_gil_ref())
}

#[pymodule]
fn _telos_protocol(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(entropy_cost, module)?)?;
    module.add_function(wrap_pyfunction!(decision_hash, module)?)?;
    module.add_function(wrap_pyfunction!(commit_single_node, module)?)?;
    Ok(())
}
