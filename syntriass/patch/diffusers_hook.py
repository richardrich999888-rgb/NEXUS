"""
SYNTRIASS Path 6 — Diffusers Hook

Monkey-patches Diffusers pipeline to inject preview tap.

No fork required. Works with existing models.
"""

from typing import Optional, Callable
import torch
from diffusers import DiffusionPipeline
from syntriass.core.inference_tap import PreviewDiffusionLoop, DiffusionPipelineHook


def patch_pipeline(
    pipeline: DiffusionPipeline,
    preview_interval: int = 4,
    preview_callback: Optional[Callable] = None,
) -> PreviewDiffusionLoop:
    """
    Patch Diffusers pipeline to enable preview extraction.
    
    This is the entry point for Path 6 integration.
    
    Args:
        pipeline: Diffusers pipeline (StableDiffusionPipeline, etc.)
        preview_interval: Extract preview every N steps
        preview_callback: Optional callback for each snapshot
        
    Returns:
        PreviewDiffusionLoop instance
        
    Example:
        >>> from diffusers import StableDiffusionPipeline
        >>> pipeline = StableDiffusionPipeline.from_pretrained("runwayml/stable-diffusion-v1-5")
        >>> preview_loop = patch_pipeline(pipeline, preview_interval=4)
        >>> # Now generate - previews will be extracted automatically
        >>> image = pipeline("a beautiful landscape")
    """
    hook = DiffusionPipelineHook(pipeline, preview_interval)
    hook.hook()
    
    if preview_callback:
        hook.preview_loop.preview_callback = preview_callback
    
    return hook.preview_loop


def unpatch_pipeline(pipeline: DiffusionPipeline):
    """
    Remove preview hook from pipeline.
    
    Args:
        pipeline: Patched pipeline
    """
    # In real implementation, would track hooks and remove them
    # For now, this is a placeholder
    pass


class PatchedPipeline:
    """
    Wrapper that automatically patches pipeline on creation.
    
    Usage:
        pipeline = PatchedPipeline.from_pretrained("runwayml/stable-diffusion-v1-5")
        # Pipeline is automatically patched
    """
    
    def __init__(self, pipeline: DiffusionPipeline, preview_interval: int = 4):
        """
        Args:
            pipeline: Diffusers pipeline
            preview_interval: Preview extraction interval
        """
        self.pipeline = pipeline
        self.preview_interval = preview_interval
        self.preview_loop = patch_pipeline(pipeline, preview_interval)
    
    def __getattr__(self, name):
        """Delegate to underlying pipeline"""
        return getattr(self.pipeline, name)
    
    def __call__(self, *args, **kwargs):
        """Delegate call to pipeline"""
        return self.pipeline(*args, **kwargs)
    
    @classmethod
    def from_pretrained(cls, *args, preview_interval: int = 4, **kwargs):
        """
        Create patched pipeline from pretrained model.
        
        Args:
            *args: Arguments for pipeline.from_pretrained
            preview_interval: Preview extraction interval
            **kwargs: Keyword arguments for pipeline.from_pretrained
            
        Returns:
            PatchedPipeline instance
        """
        # Import here to avoid circular dependency
        from diffusers import StableDiffusionPipeline
        
        pipeline = StableDiffusionPipeline.from_pretrained(*args, **kwargs)
        return cls(pipeline, preview_interval=preview_interval)

