"""
SYNTRIASS Path 6 — Inference Tap Module

Intercepts diffusion loop while it runs.
Extracts latent tensor snapshots at regular intervals.
Zero restart. Zero extra inference. Preview is free information.

Copyright (c) 2025 SYNTRIASS Labs Private Limited
"""

from typing import Optional, Callable, Any
import torch
from dataclasses import dataclass
from queue import Queue
import threading


@dataclass
class LatentSnapshot:
    """Snapshot of latent state at a specific step"""
    latent: torch.Tensor
    step_idx: int
    timestep: int
    conditioning: Optional[torch.Tensor] = None


class PreviewDiffusionLoop:
    """
    Monkey-patched diffusion loop that taps into latent states.
    
    This hooks into the denoising loop without requiring a fork.
    Preview frames are extracted at regular intervals.
    """
    
    def __init__(
        self,
        preview_interval: int = 4,
        preview_callback: Optional[Callable[[LatentSnapshot], None]] = None,
    ):
        """
        Args:
            preview_interval: Extract preview every N steps (default: 4)
            preview_callback: Called with each latent snapshot
        """
        self.preview_interval = preview_interval
        self.preview_callback = preview_callback
        self.preview_bus: Queue = Queue()
        self._step_count = 0
        
    def step(
        self,
        latent: torch.Tensor,
        step_idx: int,
        timestep: int,
        conditioning: Optional[torch.Tensor] = None,
    ) -> torch.Tensor:
        """
        Execute one denoising step and optionally extract preview.
        
        This is called from the diffusion loop. We intercept here,
        extract a snapshot if needed, then continue normally.
        
        Args:
            latent: Current latent tensor [B, C, H, W]
            step_idx: Current step index (0-based)
            timestep: Current timestep value
            conditioning: Optional conditioning tensor
            
        Returns:
            Updated latent tensor
        """
        self._step_count = step_idx
        
        # Extract preview at regular intervals
        if step_idx % self.preview_interval == 0:
            snapshot = LatentSnapshot(
                latent=latent.clone().detach(),
                step_idx=step_idx,
                timestep=timestep,
                conditioning=conditioning.clone().detach() if conditioning is not None else None,
            )
            
            # Push to preview bus (non-blocking)
            try:
                self.preview_bus.put_nowait(snapshot)
            except:
                pass  # Drop if queue full (never block)
            
            # Call callback if provided
            if self.preview_callback:
                try:
                    self.preview_callback(snapshot)
                except Exception as e:
                    # Never let preview break generation
                    print(f"Preview callback error (non-fatal): {e}")
        
        return latent
    
    def get_preview_snapshot(self, timeout: float = 0.1) -> Optional[LatentSnapshot]:
        """
        Get next preview snapshot from queue (non-blocking).
        
        Args:
            timeout: Max time to wait (seconds)
            
        Returns:
            LatentSnapshot or None if queue empty
        """
        try:
            return self.preview_bus.get(timeout=timeout)
        except:
            return None
    
    @property
    def current_step(self) -> int:
        """Current step index"""
        return self._step_count


class DiffusionPipelineHook:
    """
    Hook into Diffusers pipeline to inject preview tap.
    
    This monkey-patches the pipeline's scheduler step function
    to extract latent snapshots without modifying the pipeline code.
    """
    
    def __init__(self, pipeline, preview_interval: int = 4):
        """
        Args:
            pipeline: Diffusers pipeline instance
            preview_interval: Extract preview every N steps
        """
        self.pipeline = pipeline
        self.preview_loop = PreviewDiffusionLoop(preview_interval=preview_interval)
        self._original_step = None
        self._hooked = False
        
    def hook(self):
        """Install hook into pipeline"""
        if self._hooked:
            return
            
        # Store original step function
        if hasattr(self.pipeline, 'scheduler') and hasattr(self.pipeline.scheduler, 'step'):
            self._original_step = self.pipeline.scheduler.step
            
            # Create wrapped step function
            def wrapped_step(*args, **kwargs):
                result = self._original_step(*args, **kwargs)
                
                # Extract latent from result
                if isinstance(result, tuple):
                    latent = result[0]
                else:
                    latent = result
                
                # Get step info from kwargs or pipeline state
                step_idx = getattr(self.pipeline, '_step_index', 0)
                timestep = kwargs.get('timestep', 0)
                
                # Tap into latent
                self.preview_loop.step(latent, step_idx, timestep)
                
                return result
            
            # Replace scheduler step
            self.pipeline.scheduler.step = wrapped_step
            self._hooked = True
    
    def unhook(self):
        """Remove hook from pipeline"""
        if self._hooked and self._original_step:
            self.pipeline.scheduler.step = self._original_step
            self._hooked = False
    
    def __enter__(self):
        """Context manager entry"""
        self.hook()
        return self
    
    def __exit__(self, *args):
        """Context manager exit"""
        self.unhook()


def create_preview_tap(pipeline, preview_interval: int = 4) -> PreviewDiffusionLoop:
    """
    Convenience function to create and hook preview tap.
    
    Args:
        pipeline: Diffusers pipeline
        preview_interval: Extract preview every N steps
        
    Returns:
        PreviewDiffusionLoop instance
    """
    hook = DiffusionPipelineHook(pipeline, preview_interval)
    hook.hook()
    return hook.preview_loop

