"""
SYNTRIASS Path 6 — Gradio Frontend

Interactive UI with:
- Prompt input
- Style sliders
- Real-time preview
- Final output display
"""

import gradio as gr
import asyncio
import json
import websockets
from typing import Optional
import numpy as np
from PIL import Image
import threading
from queue import Queue


class PreviewClient:
    """WebSocket client for receiving preview frames"""
    
    def __init__(self, ws_url: str = "ws://localhost:8765"):
        self.ws_url = ws_url
        self.frame_queue: Queue = Queue(maxsize=10)
        self.running = False
        self.thread: Optional[threading.Thread] = None
        
    def start(self):
        """Start receiving preview frames"""
        if self.running:
            return
        
        self.running = True
        self.thread = threading.Thread(target=self._receive_loop, daemon=True)
        self.thread.start()
    
    def stop(self):
        """Stop receiving preview frames"""
        self.running = False
        if self.thread:
            self.thread.join(timeout=1.0)
    
    def _receive_loop(self):
        """Receive loop (runs in thread)"""
        asyncio.run(self._async_receive_loop())
    
    async def _async_receive_loop(self):
        """Async receive loop"""
        try:
            async with websockets.connect(self.ws_url) as websocket:
                while self.running:
                    try:
                        message = await asyncio.wait_for(websocket.recv(), timeout=1.0)
                        data = json.loads(message)
                        
                        if data.get("type") == "preview_frame":
                            # Decode image
                            from syntriass.preview.stream import FrameEncoder
                            encoder = FrameEncoder()
                            frame = encoder.decode_frame(message)
                            
                            # Add to queue (non-blocking)
                            try:
                                self.frame_queue.put_nowait(frame.image)
                            except:
                                pass  # Drop if queue full
                                
                    except asyncio.TimeoutError:
                        continue
                    except Exception as e:
                        print(f"Receive error: {e}")
                        break
        except Exception as e:
            print(f"WebSocket error: {e}")
    
    def get_latest_frame(self) -> Optional[np.ndarray]:
        """Get latest preview frame (non-blocking)"""
        try:
            # Get most recent frame (clear queue)
            latest = None
            while True:
                try:
                    latest = self.frame_queue.get_nowait()
                except:
                    break
            return latest
        except:
            return None


# Global preview client
_preview_client: Optional[PreviewClient] = None


def create_gradio_interface():
    """Create Gradio interface"""
    
    def generate_image(
        prompt: str,
        num_steps: int,
        guidance_scale: float,
        emotion: float,
        motion: float,
        detail: float,
    ):
        """
        Generate image with preview.
        
        Args:
            prompt: Text prompt
            num_steps: Number of inference steps
            guidance_scale: Guidance scale
            emotion: Emotion weight (-1 to 1)
            motion: Motion weight (0 to 1)
            detail: Detail weight (0 to 1)
        """
        # In real implementation, would call generation API
        # For now, return placeholder
        return None, "Generation started. Preview will appear below."
    
    def get_preview():
        """Get latest preview frame"""
        global _preview_client
        
        if _preview_client is None:
            _preview_client = PreviewClient()
            _preview_client.start()
        
        frame = _preview_client.get_latest_frame()
        
        if frame is not None:
            return Image.fromarray(frame)
        return None
    
    def update_prompt(new_prompt: str):
        """Update prompt during generation"""
        # In real implementation, would send WebSocket message
        return f"Prompt updated: {new_prompt}"
    
    def update_style(emotion: float, motion: float, detail: float):
        """Update style parameters"""
        # In real implementation, would send WebSocket message
        return f"Style updated: emotion={emotion}, motion={motion}, detail={detail}"
    
    # Create interface
    with gr.Blocks(title="SYNTRIASS Path 6 — Real-Time Preview") as interface:
        gr.Markdown("# SYNTRIASS Path 6 — Real-Time Generative AI Preview")
        gr.Markdown("**The moment generative AI became interactive.**")
        
        with gr.Row():
            with gr.Column(scale=1):
                prompt_input = gr.Textbox(
                    label="Prompt",
                    placeholder="a beautiful landscape",
                    lines=3,
                )
                
                with gr.Row():
                    num_steps = gr.Slider(
                        minimum=20,
                        maximum=100,
                        value=50,
                        step=5,
                        label="Steps",
                    )
                    guidance_scale = gr.Slider(
                        minimum=1.0,
                        maximum=20.0,
                        value=7.5,
                        step=0.5,
                        label="Guidance Scale",
                    )
                
                gr.Markdown("### Style Controls")
                emotion_slider = gr.Slider(
                    minimum=-1.0,
                    maximum=1.0,
                    value=0.0,
                    step=0.1,
                    label="Emotion",
                )
                motion_slider = gr.Slider(
                    minimum=0.0,
                    maximum=1.0,
                    value=0.0,
                    step=0.1,
                    label="Motion",
                )
                detail_slider = gr.Slider(
                    minimum=0.0,
                    maximum=1.0,
                    value=0.5,
                    step=0.1,
                    label="Detail",
                )
                
                generate_btn = gr.Button("Generate", variant="primary")
                update_prompt_btn = gr.Button("Update Prompt (Live)")
                update_style_btn = gr.Button("Update Style (Live)")
            
            with gr.Column(scale=1):
                preview_output = gr.Image(
                    label="Real-Time Preview",
                    type="pil",
                    interactive=False,
                )
                final_output = gr.Image(
                    label="Final Output",
                    type="pil",
                    interactive=False,
                )
                status_text = gr.Textbox(
                    label="Status",
                    interactive=False,
                )
        
        # Event handlers
        generate_btn.click(
            fn=generate_image,
            inputs=[
                prompt_input,
                num_steps,
                guidance_scale,
                emotion_slider,
                motion_slider,
                detail_slider,
            ],
            outputs=[final_output, status_text],
        )
        
        update_prompt_btn.click(
            fn=update_prompt,
            inputs=[prompt_input],
            outputs=[status_text],
        )
        
        update_style_btn.click(
            fn=update_style,
            inputs=[emotion_slider, motion_slider, detail_slider],
            outputs=[status_text],
        )
        
        # Auto-refresh preview
        interface.load(
            fn=get_preview,
            inputs=[],
            outputs=[preview_output],
            every=0.1,  # Update every 100ms
        )
    
    return interface


if __name__ == "__main__":
    interface = create_gradio_interface()
    interface.launch(server_name="0.0.0.0", server_port=7860)

