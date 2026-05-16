"""
TELOS Module - Commitment Accountability for AGP-OS

Provides the commitment membrane that all governed actions must cross.
"""

from .membrane import (
    CommitmentMembrane,
    Decision,
    CrossingResult,
    ConsequenceTier,
    EntropyMeter,
    AuthorityRegistry,
    TrustAccumulator,
    Authority,
    telos_membrane,
    ExecutionBlocked,
)

__all__ = [
    "CommitmentMembrane",
    "Decision", 
    "CrossingResult",
    "ConsequenceTier",
    "EntropyMeter",
    "AuthorityRegistry",
    "TrustAccumulator",
    "Authority",
    "telos_membrane",
    "ExecutionBlocked",
]
