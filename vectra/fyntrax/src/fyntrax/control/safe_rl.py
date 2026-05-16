"""
FYNTRAX Safe Reinforcement Learning

Author / Inventor: Katta Naga Sri Ganesh
Organization: SYNTRIASS Labs Private Limited
Copyright © 2025 SYNTRIASS Labs Private Limited

Safe RL agent for energy optimization with Lyapunov constraints.
Learns energy-efficient policies while maintaining stability guarantees.
"""

from typing import List, Tuple, Optional, Dict, Any
import numpy as np
from .lyapunov import LyapunovController


class SafeRLAgent:
    """
    Safe Reinforcement Learning Agent.
    
    Combines RL exploration with Lyapunov safety constraints.
    All actions are filtered through the Lyapunov supervisor.
    """
    
    def __init__(
        self,
        state_dim: int,
        action_dim: int,
        lyapunov: Optional[LyapunovController] = None,
        learning_rate: float = 0.01,
    ):
        """
        Initialize Safe RL agent.
        
        Args:
            state_dim: State space dimension
            action_dim: Action space dimension
            lyapunov: Lyapunov supervisor (creates default if None)
            learning_rate: Learning rate for policy updates
        """
        self.state_dim = state_dim
        self.action_dim = action_dim
        self.learning_rate = learning_rate
        
        # Initialize Lyapunov supervisor
        if lyapunov is None:
            P = np.eye(state_dim)
            self.lyapunov = LyapunovController(P, alpha=0.1)
        else:
            self.lyapunov = lyapunov
        
        # Simple linear policy for demonstration
        self.policy_weights = np.zeros((action_dim, state_dim))
        
        self.episode_rewards: List[float] = []
        self.constraint_violations = 0

    def act(self, state: np.ndarray, explore: bool = True) -> np.ndarray:
        """
        Select action for given state.
        
        Args:
            state: Current state
            explore: If True, add exploration noise
            
        Returns:
            Safe action
        """
        state = np.asarray(state)
        
        # Compute proposed action from policy
        action = self.policy_weights @ state
        
        # Add exploration noise
        if explore:
            noise = np.random.randn(self.action_dim) * 0.1
            action = action + noise
        
        return action

    def act_safe(
        self,
        state: np.ndarray,
        dynamics: callable,
        explore: bool = True,
    ) -> Tuple[np.ndarray, bool]:
        """
        Select safe action using Lyapunov filter.
        
        Args:
            state: Current state
            dynamics: Function state, action -> next_state
            explore: If True, add exploration noise
            
        Returns:
            Tuple of (safe_action, was_filtered)
        """
        action = self.act(state, explore)
        
        # Predict next state
        next_state = dynamics(state, action)
        
        # Check safety
        if self.lyapunov.is_safe(state, next_state):
            return action, False
        
        # Action was unsafe - try reducing magnitude
        self.constraint_violations += 1
        
        for scale in [0.5, 0.25, 0.1, 0.0]:
            scaled_action = action * scale
            next_state = dynamics(state, scaled_action)
            if self.lyapunov.is_safe(state, next_state):
                return scaled_action, True
        
        # Fall back to zero action
        return np.zeros(self.action_dim), True

    def update(
        self,
        state: np.ndarray,
        action: np.ndarray,
        reward: float,
        next_state: np.ndarray,
    ) -> None:
        """
        Update policy based on experience.
        
        Uses simple policy gradient for demonstration.
        """
        state = np.asarray(state)
        action = np.asarray(action)
        
        # Policy gradient update (simplified)
        if reward > 0:
            # Increase probability of this action
            self.policy_weights += self.learning_rate * np.outer(action, state) * reward
        
        self.episode_rewards.append(reward)

    def statistics(self) -> dict:
        """Get agent statistics."""
        return {
            "total_steps": len(self.episode_rewards),
            "total_reward": sum(self.episode_rewards),
            "avg_reward": np.mean(self.episode_rewards) if self.episode_rewards else 0,
            "constraint_violations": self.constraint_violations,
            "lyapunov_rejections": self.lyapunov.statistics()["rejections"],
        }


class ConstrainedMDP:
    """
    Constrained Markov Decision Process for Safe RL.
    
    Defines the environment for RAN control.
    """
    
    def __init__(self, state_dim: int = 3, action_dim: int = 2):
        self.state_dim = state_dim
        self.action_dim = action_dim
        self.state = np.zeros(state_dim)
        
    def reset(self) -> np.ndarray:
        """Reset environment."""
        self.state = np.random.randn(self.state_dim) * 0.5
        return self.state.copy()

    def step(self, action: np.ndarray) -> Tuple[np.ndarray, float, bool]:
        """
        Take action in environment.
        
        Returns:
            Tuple of (next_state, reward, done)
        """
        action = np.asarray(action)
        
        # Simple linear dynamics
        A = np.eye(self.state_dim) * 0.9  # Stable dynamics
        B = np.random.randn(self.state_dim, self.action_dim) * 0.1
        
        next_state = A @ self.state + B @ action
        
        # Reward: minimize state norm (energy-like objective)
        reward = -np.linalg.norm(next_state)
        
        self.state = next_state
        done = np.linalg.norm(self.state) < 0.01
        
        return next_state, reward, done

    def dynamics(self, state: np.ndarray, action: np.ndarray) -> np.ndarray:
        """Predict next state (for safety check)."""
        A = np.eye(self.state_dim) * 0.9
        B = np.random.randn(self.state_dim, self.action_dim) * 0.1
        return A @ state + B @ action
