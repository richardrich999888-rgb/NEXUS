"""
SYNTRIASS Path 6 — Conditioning Injection (INTERACTIVITY CORE)

This is why Path 6 is revolutionary.

User changes:
- prompt text
- style sliders
- emotion / motion weights

What you do:
- Modify conditioning vectors
- Blend over time
- Never restart diffusion

Key rule: Conditioning is a signal, not a constant.

This turns AI from batch → instrument.
"""

import torch
from typing import Optional, Dict, Any, Callable
from dataclasses import dataclass
import numpy as np


@dataclass
class ConditioningState:
    """Current conditioning state"""
    text_embeddings: torch.Tensor
    style_weights: Dict[str, float]
    timestep: int
    blend_factor: float = 1.0


class ConditioningInjector:
    """
    Injects user control into ongoing diffusion without restart.
    
    This is the core innovation: conditioning becomes a time-varying
    signal that can be modified mid-generation.
    """
    
    def __init__(
        self,
        tokenizer,
        text_encoder,
        blend_duration: int = 10,  # Steps to blend over
    ):
        """
        Args:
            tokenizer: Text tokenizer
            text_encoder: Text encoder model
            blend_duration: Number of steps to blend conditioning over
        """
        self.tokenizer = tokenizer
        self.text_encoder = text_encoder
        self.blend_duration = blend_duration
        
        self._current_conditioning: Optional[ConditioningState] = None
        self._target_conditioning: Optional[ConditioningState] = None
        self._blend_start_step: int = 0
        self._style_modifiers: Dict[str, Callable] = {}
        
    def encode_prompt(self, prompt: str) -> torch.Tensor:
        """
        Encode text prompt to conditioning embeddings.
        
        Args:
            prompt: Text prompt
            
        Returns:
            Text embeddings [1, 77, 768] (or similar)
        """
        with torch.no_grad():
            tokens = self.tokenizer(
                prompt,
                padding="max_length",
                max_length=77,
                truncation=True,
                return_tensors="pt",
            )
            
            embeddings = self.text_encoder(tokens.input_ids)[0]
            
        return embeddings
    
    def set_target_conditioning(
        self,
        prompt: Optional[str] = None,
        style_weights: Optional[Dict[str, float]] = None,
        current_step: int = 0,
    ):
        """
        Set target conditioning to blend towards.
        
        Args:
            prompt: New text prompt (None to keep current)
            style_weights: New style weights (None to keep current)
            current_step: Current diffusion step
        """
        if prompt is not None:
            text_embeddings = self.encode_prompt(prompt)
        elif self._current_conditioning:
            text_embeddings = self._current_conditioning.text_embeddings
        else:
            raise ValueError("No prompt provided and no current conditioning")
        
        if style_weights is None:
            style_weights = self._current_conditioning.style_weights if self._current_conditioning else {}
        
        self._target_conditioning = ConditioningState(
            text_embeddings=text_embeddings,
            style_weights=style_weights,
            timestep=current_step,
            blend_factor=0.0,
        )
        
        self._blend_start_step = current_step
    
    def get_conditioning(
        self,
        step: int,
        base_conditioning: torch.Tensor,
    ) -> torch.Tensor:
        """
        Get blended conditioning for current step.
        
        Blends from current to target conditioning over blend_duration steps.
        
        Args:
            step: Current diffusion step
            base_conditioning: Base conditioning from pipeline
            
        Returns:
            Blended conditioning tensor
        """
        if self._target_conditioning is None:
            # No target, use base
            return base_conditioning
        
        # Calculate blend factor
        steps_elapsed = step - self._blend_start_step
        blend_alpha = min(1.0, steps_elapsed / self.blend_duration)
        
        # Smoothstep for smooth blending
        blend_alpha = blend_alpha * blend_alpha * (3.0 - 2.0 * blend_alpha)
        
        # Blend text embeddings
        if self._current_conditioning:
            current_emb = self._current_conditioning.text_embeddings
        else:
            current_emb = base_conditioning
        
        target_emb = self._target_conditioning.text_embeddings
        
        # Ensure same shape
        if current_emb.shape != target_emb.shape:
            # Resize if needed
            target_emb = torch.nn.functional.interpolate(
                target_emb.unsqueeze(0),
                size=current_emb.shape[1:],
                mode='nearest',
            ).squeeze(0)
        
        blended = current_emb + blend_alpha * (target_emb - current_emb)
        
        # Apply style modifiers
        blended = self._apply_style_modifiers(blended, self._target_conditioning.style_weights)
        
        # Update current conditioning
        if blend_alpha >= 1.0:
            self._current_conditioning = self._target_conditioning
            self._target_conditioning = None
        
        return blended
    
    def _apply_style_modifiers(
        self,
        embeddings: torch.Tensor,
        style_weights: Dict[str, float],
    ) -> torch.Tensor:
        """
        Apply style weight modifications to embeddings.
        
        Args:
            embeddings: Text embeddings
            style_weights: Style weight dictionary
            
        Returns:
            Modified embeddings
        """
        modified = embeddings.clone()
        
        for style_name, weight in style_weights.items():
            if style_name in self._style_modifiers:
                modifier = self._style_modifiers[style_name]
                modified = modifier(modified, weight)
        
        return modified
    
    def register_style_modifier(
        self,
        name: str,
        modifier: Callable[[torch.Tensor, float], torch.Tensor],
    ):
        """
        Register a style modifier function.
        
        Args:
            name: Style name (e.g., "emotion", "motion", "detail")
            modifier: Function that takes (embeddings, weight) -> modified_embeddings
        """
        self._style_modifiers[name] = modifier
    
    def blend(
        self,
        old_cond: torch.Tensor,
        new_cond: torch.Tensor,
        alpha: float,
    ) -> torch.Tensor:
        """
        Blend two conditioning tensors.
        
        Args:
            old_cond: Old conditioning
            new_cond: New conditioning
            alpha: Blend factor [0, 1]
            
        Returns:
            Blended conditioning
        """
        # Smoothstep for smooth transition
        smooth_alpha = alpha * alpha * (3.0 - 2.0 * alpha)
        
        return old_cond + smooth_alpha * (new_cond - old_cond)


# Predefined style modifiers

def emotion_modifier(embeddings: torch.Tensor, weight: float) -> torch.Tensor:
    """
    Modify embeddings for emotion intensity.
    
    Weight: -1.0 (calm) to +1.0 (intense)
    """
    # Scale certain embedding dimensions based on weight
    # This is a simplified example - real implementation would
    # use learned transformations
    scale = 1.0 + weight * 0.2
    return embeddings * scale


def motion_modifier(embeddings: torch.Tensor, weight: float) -> torch.Tensor:
    """
    Modify embeddings for motion intensity.
    
    Weight: 0.0 (static) to 1.0 (high motion)
    """
    # Add motion-related bias
    bias = weight * 0.1
    return embeddings + bias


def detail_modifier(embeddings: torch.Tensor, weight: float) -> torch.Tensor:
    """
    Modify embeddings for detail level.
    
    Weight: 0.0 (simple) to 1.0 (detailed)
    """
    # Enhance high-frequency components
    enhanced = embeddings * (1.0 + weight * 0.15)
    return enhanced

