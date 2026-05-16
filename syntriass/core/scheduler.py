"""
SYNTRIASS Path 6 — Preview Scheduler

Orchestrates the preview pipeline:
- Inference tap → Preview decoder → Temporal interpolator → Stream

This is the execution layer that makes everything work together.
"""

import asyncio
import torch
from typing import Optional, Callable
from syntriass.core.inference_tap import PreviewDiffusionLoop, LatentSnapshot
from syntriass.preview.fast_decoder import FastPreviewDecoder
from syntriass.preview.temporal import TemporalInterpolator
from syntriass.preview.stream import PreviewBus
from syntriass.core.conditioning import ConditioningInjector
import numpy as np
from PIL import Image


class PreviewScheduler:
    """
    Main scheduler that orchestrates preview generation.
    
    Coordinates:
    - Inference tap (extract latents)
    - Preview decoder (fast decode)
    - Temporal interpolator (smooth frames)
    - Preview bus (stream to frontend)
    - Conditioning injection (user control)
    """
    
    def __init__(
        self,
        preview_loop: PreviewDiffusionLoop,
        decoder: FastPreviewDecoder,
        interpolator: TemporalInterpolator,
        preview_bus: PreviewBus,
        conditioning_injector: Optional[ConditioningInjector] = None,
    ):
        """
        Args:
            preview_loop: Inference tap loop
            decoder: Fast preview decoder
            interpolator: Temporal interpolator
            preview_bus: Preview streaming bus
            conditioning_injector: Optional conditioning injector
        """
        self.preview_loop = preview_loop
        self.decoder = decoder
        self.interpolator = interpolator
        self.preview_bus = preview_bus
        self.conditioning_injector = conditioning_injector
        
        self._processing_task: Optional[asyncio.Task] = None
        self._running = False
        self._last_snapshot: Optional[LatentSnapshot] = None
        
    async def start(self):
        """Start preview processing loop"""
        if self._running:
            return
        
        self._running = True
        self._processing_task = asyncio.create_task(self._process_loop())
    
    async def stop(self):
        """Stop preview processing"""
        self._running = False
        if self._processing_task:
            self._processing_task.cancel()
            try:
                await self._processing_task
            except asyncio.CancelledError:
                pass
    
    async def _process_loop(self):
        """
        Main processing loop.
        
        Continuously:
        1. Get latent snapshot from inference tap
        2. Decode to preview image
        3. Interpolate for smoothness
        4. Stream to frontend
        """
        while self._running:
            # Get next snapshot (non-blocking)
            snapshot = self.preview_loop.get_preview_snapshot(timeout=0.1)
            
            if snapshot is None:
                # No new snapshot, generate interpolated frames
                if self._last_snapshot is not None:
                    await self._generate_interpolated_frames()
                await asyncio.sleep(0.01)  # Small delay
                continue
            
            # Decode snapshot
            preview_image = await self._decode_snapshot(snapshot)
            
            if preview_image is not None:
                # Convert to numpy array
                image_array = (preview_image[0].permute(1, 2, 0).cpu().numpy() * 255).astype(np.uint8)
                
                # Push to preview bus
                await self.preview_bus.push(
                    image=image_array,
                    step_idx=snapshot.step_idx,
                    metadata={
                        "timestep": snapshot.timestep,
                        "interpolated": False,
                    }
                )
            
            # Update interpolator
            self.interpolator.add_snapshot(snapshot.latent, snapshot.step_idx)
            self._last_snapshot = snapshot
    
    async def _decode_snapshot(self, snapshot: LatentSnapshot) -> Optional[torch.Tensor]:
        """
        Decode latent snapshot to preview image.
        
        Args:
            snapshot: Latent snapshot
            
        Returns:
            Decoded image tensor or None
        """
        try:
            # Fast decode
            decoded = self.decoder.decode(snapshot.latent)
            return decoded
        except Exception as e:
            print(f"Decode error (non-fatal): {e}")
            return None
    
    async def _generate_interpolated_frames(self):
        """
        Generate interpolated frames between snapshots.
        
        This keeps preview smooth even when inference is sparse.
        """
        if self._last_snapshot is None:
            return
        
        # Get smooth sequence from interpolator
        frames = self.interpolator.get_smooth_sequence(
            self._last_snapshot.step_idx
        )
        
        # Decode and stream interpolated frames
        for frame in frames[-3:]:  # Last 3 frames only (avoid backlog)
            try:
                decoded = self.decoder.decode(frame.latent)
                image_array = (decoded[0].permute(1, 2, 0).cpu().numpy() * 255).astype(np.uint8)
                
                await self.preview_bus.push(
                    image=image_array,
                    step_idx=int(frame.step_idx),
                    metadata={
                        "interpolated": True,
                        "alpha": frame.alpha,
                    }
                )
            except Exception as e:
                # Never let interpolation break preview
                pass
    
    def update_conditioning(
        self,
        prompt: Optional[str] = None,
        style_weights: Optional[dict] = None,
        current_step: int = 0,
    ):
        """
        Update conditioning during generation.
        
        Args:
            prompt: New prompt (None to keep current)
            style_weights: New style weights (None to keep current)
            current_step: Current diffusion step
        """
        if self.conditioning_injector:
            self.conditioning_injector.set_target_conditioning(
                prompt=prompt,
                style_weights=style_weights,
                current_step=current_step,
            )


def create_preview_scheduler(
    pipeline,
    preview_interval: int = 4,
    target_resolution: tuple = (64, 64),
    preview_bus: Optional[PreviewBus] = None,
    conditioning_injector: Optional[ConditioningInjector] = None,
) -> PreviewScheduler:
    """
    Create and configure preview scheduler.
    
    Args:
        pipeline: Diffusers pipeline
        preview_interval: Extract preview every N steps
        target_resolution: Target preview resolution
        preview_bus: Optional PreviewBus (creates new if None)
        conditioning_injector: Optional ConditioningInjector
        
    Returns:
        Configured PreviewScheduler
    """
    from syntriass.core.inference_tap import create_preview_tap
    from syntriass.preview.fast_decoder import FastPreviewDecoder
    from syntriass.preview.temporal import TemporalInterpolator
    from syntriass.preview.stream import PreviewBus
    
    # Create inference tap
    preview_loop = create_preview_tap(pipeline, preview_interval)
    
    # Create decoder
    decoder = FastPreviewDecoder(
        vae_decoder=pipeline.vae.decoder,
        target_resolution=target_resolution,
    )
    
    # Create interpolator
    interpolator = TemporalInterpolator(interpolation_steps=8)
    
    # Create preview bus if not provided
    if preview_bus is None:
        preview_bus = PreviewBus(max_queue_size=10, target_fps=15.0)
    
    # Create scheduler
    scheduler = PreviewScheduler(
        preview_loop=preview_loop,
        decoder=decoder,
        interpolator=interpolator,
        preview_bus=preview_bus,
        conditioning_injector=conditioning_injector,
    )
    
    return scheduler

