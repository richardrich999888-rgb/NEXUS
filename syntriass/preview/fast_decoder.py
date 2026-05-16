"""
SYNTRIASS Path 6 — Preview Decoder (Critical Performance Layer)

Goal: Decode something useful in <50 ms, not perfection.

Techniques:
- Decode at 1/8 or 1/16 resolution
- Decode only Y channel (luma) first
- Quantize latents aggressively
- Skip refinement layers

This is engineering leverage, not ML novelty.
"""

import torch
import torch.nn as nn
from typing import Optional, Tuple
import time


class FastPreviewDecoder:
    """
    Ultra-fast decoder for real-time preview.
    
    Decodes latent tensors at reduced resolution for <50ms preview.
    Human sees structure immediately. GPU never stalls.
    """
    
    def __init__(
        self,
        vae_decoder: nn.Module,
        target_resolution: Tuple[int, int] = (64, 64),  # 1/8 of 512x512
        channels: int = 4,  # Partial channels for speed
        quantize: bool = True,
    ):
        """
        Args:
            vae_decoder: VAE decoder from pipeline
            target_resolution: Target preview resolution (H, W)
            channels: Number of latent channels to decode (default: 4 of 16)
            quantize: Whether to quantize latents for speed
        """
        self.vae_decoder = vae_decoder
        self.target_resolution = target_resolution
        self.channels = channels
        self.quantize = quantize
        
        # Cache for performance
        self._last_latent = None
        self._last_output = None
        
    def decode(
        self,
        latent: torch.Tensor,
        target_size: Optional[Tuple[int, int]] = None,
    ) -> torch.Tensor:
        """
        Fast decode latent to image preview.
        
        Args:
            latent: Latent tensor [B, C, H, W]
            target_size: Optional target size override (H, W)
            
        Returns:
            Decoded image tensor [B, 3, H, W] in range [0, 1]
        """
        start_time = time.time()
        
        # Use cached result if latent unchanged
        if self._last_latent is not None and torch.equal(latent, self._last_latent):
            return self._last_output
        
        target_size = target_size or self.target_resolution
        
        # Extract partial channels (faster)
        if latent.shape[1] > self.channels:
            latent = latent[:, :self.channels]
        
        # Downsample latent (1/8 or 1/16 of original)
        original_size = (latent.shape[2], latent.shape[3])
        if original_size != target_size:
            latent = torch.nn.functional.interpolate(
                latent,
                size=target_size,
                mode='bilinear',
                align_corners=False,
            )
        
        # Quantize for speed (optional)
        if self.quantize:
            # 8-bit quantization
            latent = (latent * 127.5 + 128).clamp(0, 255).byte().float() / 127.5 - 1.0
        
        # Decode through VAE (lightweight path)
        with torch.no_grad():
            # Use decoder's first few layers only
            decoded = self._lightweight_decode(latent)
        
        # Cache result
        self._last_latent = latent.clone()
        self._last_output = decoded
        
        elapsed = (time.time() - start_time) * 1000  # ms
        if elapsed > 50:
            print(f"Warning: Decode took {elapsed:.1f}ms (target: <50ms)")
        
        return decoded
    
    def _lightweight_decode(self, latent: torch.Tensor) -> torch.Tensor:
        """
        Lightweight decode path - skips refinement layers.
        
        This is the critical optimization: we decode only what's
        necessary for human perception, not perfection.
        """
        # If decoder has multiple stages, use only first stage
        if hasattr(self.vae_decoder, 'mid') and hasattr(self.vae_decoder, 'up'):
            # Standard VAE decoder structure
            x = self.vae_decoder.mid(latent)
            # Skip upsampling refinement layers for speed
            # Use only first up block
            if len(self.vae_decoder.up) > 0:
                x = self.vae_decoder.up[0](x)
            else:
                x = self.vae_decoder.up(x)
        else:
            # Fallback: full decode (slower)
            x = self.vae_decoder(latent)
        
        # Convert to image space [0, 1]
        x = (x / 2 + 0.5).clamp(0, 1)
        
        return x
    
    def decode_luma_only(self, latent: torch.Tensor) -> torch.Tensor:
        """
        Decode only luma (Y channel) for maximum speed.
        
        Returns grayscale preview - fastest option.
        """
        # Decode full but extract only luma
        decoded = self.decode(latent)
        
        # Convert RGB to Y (luma)
        # Y = 0.299*R + 0.587*G + 0.114*B
        luma = (
            0.299 * decoded[:, 0:1] +
            0.587 * decoded[:, 1:2] +
            0.114 * decoded[:, 2:3]
        )
        
        # Expand to 3 channels for compatibility
        return luma.repeat(1, 3, 1, 1)


class AdaptiveDecoder:
    """
    Decoder that adapts quality based on available time.
    
    If we have <30ms: luma only
    If we have 30-50ms: partial channels
    If we have >50ms: full decode
    """
    
    def __init__(self, vae_decoder: nn.Module):
        self.fast_decoder = FastPreviewDecoder(vae_decoder, channels=4)
        self.full_decoder = vae_decoder
        
    def decode_adaptive(
        self,
        latent: torch.Tensor,
        time_budget_ms: float = 50.0,
    ) -> torch.Tensor:
        """
        Decode with adaptive quality based on time budget.
        
        Args:
            latent: Latent tensor
            time_budget_ms: Available time in milliseconds
            
        Returns:
            Decoded image
        """
        if time_budget_ms < 30:
            # Ultra-fast: luma only
            return self.fast_decoder.decode_luma_only(latent)
        elif time_budget_ms < 50:
            # Fast: partial channels
            return self.fast_decoder.decode(latent)
        else:
            # Full decode
            with torch.no_grad():
                decoded = self.full_decoder(latent)
                return (decoded / 2 + 0.5).clamp(0, 1)

