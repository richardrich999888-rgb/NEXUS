"""
Optional Rust TELOS bridge.

Install locally with:
    maturin develop --manifest-path telos-protocol/Cargo.toml --features python
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


try:
    import _telos_protocol as _rust_telos
except ImportError:  # pragma: no cover - depends on optional local build step
    _rust_telos = None


def rust_telos_available() -> bool:
    """Return True when the PyO3 TELOS extension is installed."""
    return _rust_telos is not None


@dataclass(frozen=True)
class RustCommitmentResult:
    """Stable Python-side shape for Rust commitment results."""

    status: str
    decision_id: str
    entropy_consumed: int | None = None
    commitment_hash: str | None = None
    committed_at: str | None = None
    reason: str | None = None


def entropy_cost(tier: int, budget: int, trust_score: float) -> int:
    """Calculate TELOS entropy cost using the Rust protocol implementation."""
    if _rust_telos is None:
        raise RuntimeError("Rust TELOS extension is not installed")
    return int(_rust_telos.entropy_cost(tier, budget, trust_score))


def decision_hash(domain: str, action: str, tier: int) -> str:
    """Hash a TELOS decision using the Rust protocol implementation."""
    if _rust_telos is None:
        raise RuntimeError("Rust TELOS extension is not installed")
    return str(_rust_telos.decision_hash(domain, action, tier))


def commit_single_node(
    domain: str,
    action: str,
    tier: int,
    agent_id: str,
    budget: int,
    trust_score: float = 0.5,
) -> RustCommitmentResult:
    """Run a single-node Rust TELOS commitment from Python."""
    if _rust_telos is None:
        raise RuntimeError("Rust TELOS extension is not installed")
    raw: dict[str, Any] = dict(
        _rust_telos.commit_single_node(domain, action, tier, agent_id, budget, trust_score)
    )
    return RustCommitmentResult(
        status=str(raw["status"]),
        decision_id=str(raw["decision_id"]),
        entropy_consumed=raw.get("entropy_consumed"),
        commitment_hash=raw.get("commitment_hash"),
        committed_at=raw.get("committed_at"),
        reason=raw.get("reason"),
    )
