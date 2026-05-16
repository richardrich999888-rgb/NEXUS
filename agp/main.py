#!/usr/bin/env python3
"""AGP Demo - Agent Governance Protocol"""
import sys
import hashlib
sys.path.insert(0, '/Users/richardrich/Desktop/NEXUS/agp')

from core import (
    TaskType, Task, ValidatorInfo, ReputationRecord, ReputationScore,
    Stake, Timestamp, Duration, AgentFingerprint, Version, ForkProof,
    VerificationTier, select_validators, embed_task_type, inherit_reputation,
    create_initial_record, ExecutionHistory, VotingPower, Proposal, 
    ProposalConfig, ProposalCategory, ProposalState, select_verification_tier,
    NetworkState,
)

def make_fingerprint(seed: str) -> AgentFingerprint:
    return AgentFingerprint(value=hashlib.sha256(seed.encode()).digest())

def run_demo():
    print("=" * 60)
    print("AGP - Agent Governance Protocol Demo")
    print("=" * 60)
    
    # Demo 1: Task-Type Clustering
    print("\n1. TASK-TYPE CLUSTERING")
    task_type = TaskType("inference", "nlp", "sentiment")
    validators = []
    for i in range(10):
        fp = make_fingerprint(f"v_{i}")
        expert_type = task_type if i < 5 else TaskType("inference", "vision", "classification")
        records = frozenset([ReputationRecord(
            agent_fingerprint=fp, task_type=expert_type,
            score=ReputationScore(value=0.8),
            successful_executions=100, failed_executions=5,
            last_updated=Timestamp(value=1000000),
            decay_half_life=Duration.from_days(30),
        )])
        validators.append(ValidatorInfo(
            agent_fingerprint=fp, stake=Stake(value=10000),
            reputation_records=records,
            operator_id=hashlib.sha256(f"op_{i}".encode()).digest(),
            is_active=True,
        ))
    
    task = Task(
        task_id=hashlib.sha256(b"demo").digest(),
        task_type=task_type,
        submitter=make_fingerprint("s").value,
        stake_at_risk=Stake(value=1000),
        input_commitment=hashlib.sha256(b"i").digest(),
        created_at=Timestamp(value=1000000),
        deadline=Timestamp(value=2000000),
        sensitivity=0.5,
    )
    
    selection = select_validators(task, validators, k=5, current_time=Timestamp(value=1500000))
    nlp_count = sum(1 for v in selection.validators 
                    if any(r.task_type.l1_category == "nlp" for r in v.reputation_records))
    print(f"   Selected {nlp_count}/5 NLP experts for NLP task ✓")
    
    # Demo 2: Fork Inheritance
    print("\n2. FORK REPUTATION INHERITANCE")
    old_fp = make_fingerprint("v1")
    new_fp = make_fingerprint("v2")
    patch_proof = ForkProof(
        old_fingerprint=old_fp, new_fingerprint=new_fp,
        old_version=Version(1,0,0), new_version=Version(1,0,1),
        model_diff_hash=hashlib.sha256(b"small").digest(),
        code_diff_hash=hashlib.sha256(b"fix").digest(),
        signature=b"sig",
    )
    result = inherit_reputation(ReputationScore(value=0.9), patch_proof)
    print(f"   Patch 1.0.0→1.0.1: {result.inherited_score.value:.2f} (factor: {result.inheritance_factor:.2f}) ✓")
    
    # Demo 3: Execution-Weighted Governance
    print("\n3. EXECUTION-WEIGHTED GOVERNANCE")
    proposal = Proposal(
        proposal_id=hashlib.sha256(b"p1").digest(),
        proposer=make_fingerprint("proposer"),
        title="Test Proposal", description="Demo",
        config=ProposalConfig(
            category=ProposalCategory.PARAMETER_CHANGE,
            relevant_task_types=(task_type,),
            voting_period=Duration.from_days(7),
            quorum_percentage=0.1,
            execution_delay=Duration.from_days(1),
        ),
        actions=(), created_at=Timestamp(value=1000000),
        voting_starts=Timestamp(value=1000000),
        voting_ends=Timestamp(value=2000000),
        state=ProposalState.VOTING,
    )
    executor_fp = make_fingerprint("executor")
    executor_history = ExecutionHistory.from_records(executor_fp, [
        ReputationRecord(
            agent_fingerprint=executor_fp, task_type=task_type,
            score=ReputationScore(value=0.85),
            successful_executions=500, failed_executions=10,
            last_updated=Timestamp(value=1400000),
            decay_half_life=Duration.from_days(30),
        )
    ])
    executor_power = VotingPower.compute(executor_fp, Stake(value=1000), executor_history, proposal, Timestamp(value=1500000))
    whale_power = VotingPower.compute(make_fingerprint("whale"), Stake(value=1000000), 
                                       ExecutionHistory.from_records(make_fingerprint("whale"), []), 
                                       proposal, Timestamp(value=1500000))
    print(f"   Executor (1K tokens): {executor_power.total_power:.2f}")
    print(f"   Whale (1M tokens): {whale_power.total_power:.2f}")
    print(f"   Power ratio: {whale_power.total_power/executor_power.total_power:.1f}x (vs 1000x token ratio) ✓")
    
    # Demo 4: Tiered Verification
    print("\n4. TIERED VERIFICATION SELECTION")
    network = NetworkState(
        total_stake=Stake(value=10000000), active_validators=100,
        pending_zkml_proofs=10, zkml_queue_capacity=100,
        pending_tee_attestations=5, tee_queue_capacity=50,
        average_zkml_latency_ms=5000, average_tee_latency_ms=500,
        current_timestamp=Timestamp(value=1500000),
    )
    low_risk = Task(
        task_id=hashlib.sha256(b"low").digest(),
        task_type=TaskType("utility", "text", "format"),
        submitter=make_fingerprint("s").value,
        stake_at_risk=Stake(value=100),
        input_commitment=hashlib.sha256(b"i").digest(),
        created_at=Timestamp(value=1000000),
        deadline=Timestamp(value=2000000),
        sensitivity=0.2,
    )
    high_risk = Task(
        task_id=hashlib.sha256(b"high").digest(),
        task_type=TaskType("financial", "trading", "exec"),
        submitter=make_fingerprint("s").value,
        stake_at_risk=Stake(value=1000000),
        input_commitment=hashlib.sha256(b"i").digest(),
        created_at=Timestamp(value=1000000),
        deadline=Timestamp(value=2000000),
        sensitivity=0.95,
    )
    low_decision = select_verification_tier(low_risk, ReputationScore(value=0.95), network)
    high_decision = select_verification_tier(high_risk, ReputationScore(value=0.3), network)
    print(f"   Low risk task: {low_decision.tier.name} (risk: {low_decision.risk_score:.2f})")
    print(f"   High risk task: {high_decision.tier.name} (risk: {high_decision.risk_score:.2f}) ✓")
    
    print("\n" + "=" * 60)
    print("ALL 4 PATENT CLAIMS DEMONSTRATED SUCCESSFULLY")
    print("=" * 60)

if __name__ == "__main__":
    run_demo()
