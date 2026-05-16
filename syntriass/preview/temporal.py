"""
SYNTRIASS Path 6 — Temporal Interpolator (SYNTRIASS MOAT)

Without this, preview feels jittery and fake.

What it does:
- Interpolates between sparse latent snapshots
- Predicts near-future frames
- Makes time continuous

User experience: "It's moving even when the model isn't."
"""

import torch
from typing import Optional, List, Tuple
from dataclasses import dataclass
import numpy as np


@dataclass
class InterpolationFrame:
    """Interpolated frame between two snapshots"""
    latent: torch.Tensor
    alpha: float  # Interpolation factor [0, 1]
    step_idx: float  # Fractional step index


class TemporalInterpolator:
    """
    Interpolates between sparse latent snapshots for smooth preview.
    
    This is the SYNTRIASS moat: temporal continuity makes preview
    feel real-time even when inference is sparse.
    """
    
    def __init__(
        self,
        interpolation_steps: int = 8,  # Frames between snapshots
        method: str = "linear",  # "linear", "polynomial", "harmonic"
    ):
        """
        Args:
            interpolation_steps: Number of interpolated frames between snapshots
            method: Interpolation method
        """
        self.interpolation_steps = interpolation_steps
        self.method = method
        self._snapshot_buffer: List[Tuple[torch.Tensor, int]] = []
        self._max_buffer_size = 3
        
    def add_snapshot(self, latent: torch.Tensor, step_idx: int):
        """
        Add a new latent snapshot.
        
        Args:
            latent: Latent tensor
            step_idx: Step index
        """
        self._snapshot_buffer.append((latent.clone().detach(), step_idx))
        
        # Keep only recent snapshots
        if len(self._snapshot_buffer) > self._max_buffer_size:
            self._snapshot_buffer.pop(0)
    
    def interpolate(
        self,
        latent_prev: torch.Tensor,
        latent_now: torch.Tensor,
        alpha: float,
    ) -> torch.Tensor:
        """
        Interpolate between two latents.
        
        Mathematically: L̂(t+Δ) = L(t) + (L(t) − L(t−1)) · Δ
        
        Args:
            latent_prev: Previous latent [B, C, H, W]
            latent_now: Current latent [B, C, H, W]
            alpha: Interpolation factor [0, 1]
            
        Returns:
            Interpolated latent
        """
        if self.method == "linear":
            return self._linear_interpolate(latent_prev, latent_now, alpha)
        elif self.method == "polynomial":
            return self._polynomial_interpolate(latent_prev, latent_now, alpha)
        elif self.method == "harmonic":
            return self._harmonic_interpolate(latent_prev, latent_now, alpha)
        else:
            return self._linear_interpolate(latent_prev, latent_now, alpha)
    
    def _linear_interpolate(
        self,
        latent_prev: torch.Tensor,
        latent_now: torch.Tensor,
        alpha: float,
    ) -> torch.Tensor:
        """Simple linear interpolation"""
        return latent_prev + alpha * (latent_now - latent_prev)
    
    def _polynomial_interpolate(
        self,
        latent_prev: torch.Tensor,
        latent_now: torch.Tensor,
        alpha: float,
    ) -> torch.Tensor:
        """
        Polynomial interpolation for smoother motion.
        
        Uses smoothstep: 3α² - 2α³
        """
        smooth_alpha = alpha * alpha * (3.0 - 2.0 * alpha)
        return latent_prev + smooth_alpha * (latent_now - latent_prev)
    
    def _harmonic_interpolate(
        self,
        latent_prev: torch.Tensor,
        latent_now: torch.Tensor,
        alpha: float,
    ) -> torch.Tensor:
        """
        Harmonic interpolation (sine-based).
        
        Smoothest option, but slightly more expensive.
        """
        harmonic_alpha = 0.5 * (1.0 - np.cos(np.pi * alpha))
        return latent_prev + harmonic_alpha * (latent_now - latent_prev)
    
    def generate_interpolation_sequence(
        self,
        latent_prev: torch.Tensor,
        latent_now: torch.Tensor,
        step_prev: int,
        step_now: int,
    ) -> List[InterpolationFrame]:
        """
        Generate sequence of interpolated frames between two snapshots.
        
        Args:
            latent_prev: Previous snapshot latent
            latent_now: Current snapshot latent
            step_prev: Previous step index
            step_now: Current step index
            
        Returns:
            List of interpolated frames
        """
        frames = []
        num_steps = step_now - step_prev
        
        if num_steps <= 1:
            # No interpolation needed
            return [InterpolationFrame(latent_now, 1.0, float(step_now))]
        
        for i in range(1, num_steps):
            alpha = i / num_steps
            step_idx = step_prev + i
            
            interpolated = self.interpolate(latent_prev, latent_now, alpha)
            
            frames.append(InterpolationFrame(
                latent=interpolated,
                alpha=alpha,
                step_idx=float(step_idx),
            ))
        
        # Add final frame
        frames.append(InterpolationFrame(latent_now, 1.0, float(step_now)))
        
        return frames
    
    def predict_next(
        self,
        latent_prev: torch.Tensor,
        latent_now: torch.Tensor,
        steps_ahead: int = 1,
    ) -> torch.Tensor:
        """
        Predict near-future latent state.
        
        Uses velocity extrapolation: v = (now - prev), next = now + v
        
        Args:
            latent_prev: Previous latent
            latent_now: Current latent
            steps_ahead: How many steps to predict ahead
            
        Returns:
            Predicted latent
        """
        velocity = latent_now - latent_prev
        predicted = latent_now + velocity * steps_ahead
        
        return predicted
    
    def get_smooth_sequence(
        self,
        current_step: int,
    ) -> List[InterpolationFrame]:
        """
        Get smooth sequence of frames up to current step.
        
        Uses buffered snapshots to generate interpolated sequence.
        
        Args:
            current_step: Current step index
            
        Returns:
            List of interpolated frames
        """
        if len(self._snapshot_buffer) < 2:
            # Not enough data for interpolation
            if len(self._snapshot_buffer) == 1:
                latent, step = self._snapshot_buffer[0]
                return [InterpolationFrame(latent, 1.0, float(step))]
            return []
        
        frames = []
        
        # Generate interpolated sequence between all snapshot pairs
        for i in range(len(self._snapshot_buffer) - 1):
            latent_prev, step_prev = self._snapshot_buffer[i]
            latent_now, step_now = self._snapshot_buffer[i + 1]
            
            interp_frames = self.generate_interpolation_sequence(
                latent_prev, latent_now, step_prev, step_now
            )
            frames.extend(interp_frames)
        
        # Filter to current step
        frames = [f for f in frames if f.step_idx <= current_step]
        
        return frames

