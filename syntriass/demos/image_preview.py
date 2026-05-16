"""
SYNTRIASS Path 6 — Image Preview Demo

Complete end-to-end demo of real-time image generation preview.
"""

import asyncio
import torch
from diffusers import StableDiffusionPipeline
from syntriass.patch.diffusers_hook import patch_pipeline
from syntriass.core.scheduler import create_preview_scheduler
from syntriass.core.conditioning import ConditioningInjector
from syntriass.preview.stream import PreviewBus
from syntriass.api.websocket import PreviewWebSocketServer
import numpy as np
from PIL import Image


async def main():
    """Main demo function"""
    print("SYNTRIASS Path 6 — Image Preview Demo")
    print("=" * 50)
    
    # Load pipeline
    print("Loading Stable Diffusion pipeline...")
    pipeline = StableDiffusionPipeline.from_pretrained(
        "runwayml/stable-diffusion-v1-5",
        torch_dtype=torch.float16 if torch.cuda.is_available() else torch.float32,
    )
    
    if torch.cuda.is_available():
        pipeline = pipeline.to("cuda")
    
    # Patch pipeline for preview
    print("Patching pipeline for preview extraction...")
    preview_loop = patch_pipeline(pipeline, preview_interval=4)
    
    # Create preview bus
    preview_bus = PreviewBus(max_queue_size=10, target_fps=15.0)
    
    # Create conditioning injector
    conditioning_injector = ConditioningInjector(
        tokenizer=pipeline.tokenizer,
        text_encoder=pipeline.text_encoder,
        blend_duration=10,
    )
    
    # Register style modifiers
    from syntriass.core.conditioning import (
        emotion_modifier,
        motion_modifier,
        detail_modifier,
    )
    conditioning_injector.register_style_modifier("emotion", emotion_modifier)
    conditioning_injector.register_style_modifier("motion", motion_modifier)
    conditioning_injector.register_style_modifier("detail", detail_modifier)
    
    # Create preview scheduler
    print("Creating preview scheduler...")
    scheduler = create_preview_scheduler(
        pipeline=pipeline,
        preview_interval=4,
        target_resolution=(64, 64),
        preview_bus=preview_bus,
        conditioning_injector=conditioning_injector,
    )
    
    # Start preview processing
    await scheduler.start()
    
    # Start WebSocket server
    print("Starting WebSocket server on ws://localhost:8765...")
    ws_server = PreviewWebSocketServer(preview_bus, host="localhost", port=8765)
    ws_task = asyncio.create_task(ws_server.start())
    
    # Generate image with preview
    prompt = "a beautiful landscape with mountains and a lake, sunset, highly detailed"
    print(f"\nGenerating: '{prompt}'")
    print("Preview will stream to WebSocket clients...")
    
    # Run generation in background
    async def generate():
        try:
            image = pipeline(
                prompt,
                num_inference_steps=50,
                guidance_scale=7.5,
            ).images[0]
            
            print("\nGeneration complete!")
            print(f"Final image size: {image.size}")
            
            # Save final image
            image.save("output_final.png")
            print("Saved: output_final.png")
            
        except Exception as e:
            print(f"Generation error: {e}")
    
    gen_task = asyncio.create_task(generate())
    
    # Wait a bit for preview to start
    await asyncio.sleep(2.0)
    
    # Simulate user control: update prompt mid-generation
    print("\n[User Action] Updating prompt mid-generation...")
    new_prompt = "a beautiful landscape with mountains and a lake, sunset, highly detailed, cinematic lighting"
    conditioning_injector.set_target_conditioning(
        prompt=new_prompt,
        current_step=25,  # Mid-generation
    )
    print("Prompt update queued (will blend over 10 steps)")
    
    # Wait for generation to complete
    await gen_task
    
    # Stop preview
    await scheduler.stop()
    ws_task.cancel()
    
    print("\nDemo complete!")
    print("Connect to ws://localhost:8765 to see preview stream")


if __name__ == "__main__":
    asyncio.run(main())

