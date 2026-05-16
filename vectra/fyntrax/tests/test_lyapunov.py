"""Tests for FYNTRAX Lyapunov controller."""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

import numpy as np
from fyntrax.control.lyapunov import (
    LyapunovController,
    create_identity_controller,
    create_weighted_controller,
)


def test_lyapunov_function_positive():
    """V(x) should be positive for x != 0."""
    controller = create_identity_controller(dim=3)
    
    x = np.array([1.0, 2.0, 3.0])
    
    v = controller.V(x)
    
    assert v > 0
    assert v == 14.0  # 1^2 + 2^2 + 3^2


def test_lyapunov_function_zero():
    """V(0) should be zero."""
    controller = create_identity_controller(dim=3)
    
    x = np.array([0.0, 0.0, 0.0])
    
    v = controller.V(x)
    
    assert v == 0.0


def test_safe_transition():
    """Decreasing state norm should be safe."""
    controller = create_identity_controller(dim=3, alpha=0.1)
    
    x = np.array([10.0, 5.0, 3.0])
    x_next = np.array([6.0, 3.0, 2.0])
    
    is_safe = controller.is_safe(x, x_next)
    
    assert is_safe


def test_unsafe_transition():
    """Increasing state norm should be unsafe."""
    controller = create_identity_controller(dim=3, alpha=0.1)
    
    x = np.array([10.0, 5.0, 3.0])
    x_next = np.array([15.0, 8.0, 5.0])
    
    is_safe = controller.is_safe(x, x_next)
    
    assert not is_safe


def test_action_filtering_safe():
    """Safe action should pass through."""
    controller = create_identity_controller(dim=3, alpha=0.1)
    
    x = np.array([10.0, 5.0, 3.0])
    x_next = np.array([6.0, 3.0, 2.0])
    
    filtered = controller.filter_action(x, x_next)
    
    np.testing.assert_array_equal(filtered, x_next)


def test_action_filtering_unsafe():
    """Unsafe action should be filtered to current state."""
    controller = create_identity_controller(dim=3, alpha=0.1)
    
    x = np.array([10.0, 5.0, 3.0])
    x_next = np.array([15.0, 8.0, 5.0])  # Unsafe
    
    filtered = controller.filter_action(x, x_next)
    
    np.testing.assert_array_equal(filtered, x)  # Reverts to current


def test_weighted_controller():
    """Weighted controller should weight state dimensions."""
    weights = [1.0, 2.0, 3.0]
    controller = create_weighted_controller(weights)
    
    x = np.array([1.0, 1.0, 1.0])
    
    v = controller.V(x)
    
    assert v == 6.0  # 1*1 + 2*1 + 3*1


def test_statistics():
    """Statistics should track approvals/rejections."""
    controller = create_identity_controller(dim=3, alpha=0.1)
    
    x = np.array([10.0, 5.0, 3.0])
    x_safe = np.array([6.0, 3.0, 2.0])
    x_unsafe = np.array([15.0, 8.0, 5.0])
    
    controller.evaluate(x, x_safe)
    controller.evaluate(x, x_unsafe)
    
    stats = controller.statistics()
    
    assert stats["approvals"] == 1
    assert stats["rejections"] == 1
    assert stats["rejection_rate"] == 0.5


if __name__ == "__main__":
    test_lyapunov_function_positive()
    test_lyapunov_function_zero()
    test_safe_transition()
    test_unsafe_transition()
    test_action_filtering_safe()
    test_action_filtering_unsafe()
    test_weighted_controller()
    test_statistics()
    print("All Lyapunov tests passed!")
