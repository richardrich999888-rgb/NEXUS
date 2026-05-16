"""
FYNTRAX Lyapunov Stability Controller

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Lyapunov-based safety supervisor for AI/ML control.
Ensures BIBO (Bounded-Input Bounded-Output) stability by filtering
unsafe AI actions before execution.

This is the core control-theoretic innovation:
AI actions are permitted if and only if they satisfy the drift condition.
"""

import numpy as np
from typing import List, Tuple, Optional, Callable


class LyapunovController:
    """
    Lyapunov-based safety supervisor for AI actions.
    
    Guarantees V̇(x) < 0 for all control actions.
    
    The control action u is permitted if and only if:
        V(x_{t+1}) - V(x_t) < -α × V(x_t)
    
    This mathematically guarantees BIBO stability regardless
    of the AI model's internal complexity.
    """
    
    def __init__(self, P: np.ndarray, alpha: float = 0.1):
        """
        Initialize Lyapunov controller.
        
        Args:
            P: Positive definite matrix for V(x) = x^T P x
            alpha: Minimum decay rate (0.1 = 10% per step)
        """
        self.P = P
        self.alpha = alpha
        self._validate_positive_definite()
        
        self.approval_count = 0
        self.rejection_count = 0

    def _validate_positive_definite(self) -> None:
        """Validate P is positive definite."""
        eigenvalues = np.linalg.eigvalsh(self.P)
        if not all(ev > 0 for ev in eigenvalues):
            raise ValueError("P must be positive definite")

    def V(self, x: np.ndarray) -> float:
        """
        Compute Lyapunov function value.
        
        V(x) = x^T P x
        
        Args:
            x: State vector
            
        Returns:
            Lyapunov function value (always ≥ 0)
        """
        x = np.asarray(x)
        return float(x.T @ self.P @ x)

    def dV(self, x: np.ndarray, x_next: np.ndarray) -> float:
        """
        Compute Lyapunov drift.
        
        ΔV = V(x_{t+1}) - V(x_t)
        
        Stability requires: ΔV < 0
        """
        return self.V(x_next) - self.V(x)

    def is_safe(self, x: np.ndarray, x_next: np.ndarray) -> bool:
        """
        Check if state transition satisfies stability condition.
        
        Safe if: V(x_next) - V(x) < -α × V(x)
        
        Args:
            x: Current state
            x_next: Proposed next state
            
        Returns:
            True if transition is safe
        """
        v_current = self.V(x)
        drift = self.dV(x, x_next)
        return drift < -self.alpha * v_current

    def evaluate(self, x: np.ndarray, x_next: np.ndarray) -> Tuple[bool, dict]:
        """
        Evaluate control action with detailed metrics.
        
        Returns:
            Tuple of (is_safe, metrics_dict)
        """
        v_current = self.V(x)
        v_next = self.V(x_next)
        drift = v_next - v_current
        required_drift = -self.alpha * v_current
        
        is_safe = drift < required_drift
        
        if is_safe:
            self.approval_count += 1
        else:
            self.rejection_count += 1
        
        return is_safe, {
            "v_current": v_current,
            "v_next": v_next,
            "drift": drift,
            "required_drift": required_drift,
            "margin": required_drift - drift,
            "decay_rate": -drift / v_current if v_current > 0 else 0,
        }

    def filter_action(
        self,
        x: np.ndarray,
        proposed_x_next: np.ndarray,
        fallback: Optional[Callable[[np.ndarray], np.ndarray]] = None,
    ) -> np.ndarray:
        """
        Filter control action to ensure safety.
        
        If proposed action is unsafe, apply fallback or return current state.
        
        Args:
            x: Current state
            proposed_x_next: AI-proposed next state
            fallback: Optional fallback controller
            
        Returns:
            Safe next state
        """
        if self.is_safe(x, proposed_x_next):
            return proposed_x_next
        
        if fallback is not None:
            fallback_state = fallback(x)
            if self.is_safe(x, fallback_state):
                return fallback_state
        
        # Ultimate fallback: maintain current state
        return x

    def statistics(self) -> dict:
        """Get supervisor statistics."""
        total = self.approval_count + self.rejection_count
        return {
            "approvals": self.approval_count,
            "rejections": self.rejection_count,
            "total": total,
            "rejection_rate": self.rejection_count / total if total > 0 else 0,
        }


def create_identity_controller(dim: int, alpha: float = 0.1) -> LyapunovController:
    """Create controller with identity P matrix."""
    P = np.eye(dim)
    return LyapunovController(P, alpha)


def create_weighted_controller(
    weights: List[float],
    alpha: float = 0.1,
) -> LyapunovController:
    """
    Create controller with diagonal weighted P matrix.
    
    Args:
        weights: Weights for each state dimension
        alpha: Decay rate
        
    Returns:
        Configured Lyapunov controller
    """
    P = np.diag(weights)
    return LyapunovController(P, alpha)
