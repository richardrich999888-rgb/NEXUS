"""
SYNTRIASS Path 6 — Control Plane API

REST API for control operations:
- Start/stop generation
- Update prompts
- Adjust style parameters
- Query status
"""

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Optional, Dict, Any
import asyncio
from syntriass.preview.stream import PreviewBus
from syntriass.core.conditioning import ConditioningInjector


app = FastAPI(title="SYNTRIASS Preview API")

# CORS for frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class PromptRequest(BaseModel):
    """Prompt update request"""
    prompt: str
    blend_duration: Optional[int] = 10


class StyleRequest(BaseModel):
    """Style parameter update request"""
    style_weights: Dict[str, float]
    blend_duration: Optional[int] = 10


class GenerationRequest(BaseModel):
    """Start generation request"""
    prompt: str
    num_inference_steps: int = 50
    guidance_scale: float = 7.5
    style_weights: Optional[Dict[str, float]] = None


# Global state (in production, use proper state management)
_preview_bus: Optional[PreviewBus] = None
_conditioning_injector: Optional[ConditioningInjector] = None
_generation_task: Optional[asyncio.Task] = None


@app.post("/api/v1/generate")
async def start_generation(request: GenerationRequest):
    """
    Start new generation.
    
    Args:
        request: Generation parameters
        
    Returns:
        Generation ID and status
    """
    # In real implementation, would start generation task
    return {
        "generation_id": "gen_123",
        "status": "started",
        "message": "Generation started"
    }


@app.post("/api/v1/prompt")
async def update_prompt(request: PromptRequest):
    """
    Update prompt during generation.
    
    Args:
        request: New prompt and blend duration
        
    Returns:
        Update status
    """
    if _conditioning_injector is None:
        return {"error": "No active generation"}
    
    # Update conditioning (non-blocking)
    _conditioning_injector.set_target_conditioning(
        prompt=request.prompt,
        current_step=0,  # Would get from generation state
    )
    
    return {
        "status": "updated",
        "message": "Prompt update queued"
    }


@app.post("/api/v1/style")
async def update_style(request: StyleRequest):
    """
    Update style parameters during generation.
    
    Args:
        request: Style weights and blend duration
        
    Returns:
        Update status
    """
    if _conditioning_injector is None:
        return {"error": "No active generation"}
    
    _conditioning_injector.set_target_conditioning(
        style_weights=request.style_weights,
        current_step=0,  # Would get from generation state
    )
    
    return {
        "status": "updated",
        "message": "Style update queued"
    }


@app.get("/api/v1/status")
async def get_status():
    """
    Get generation status.
    
    Returns:
        Current status and statistics
    """
    if _preview_bus is None:
        return {"status": "idle"}
    
    stats = _preview_bus.get_stats()
    
    return {
        "status": "generating" if _generation_task else "idle",
        "preview_stats": stats,
    }


@app.websocket("/ws/preview")
async def websocket_preview(websocket: WebSocket):
    """
    WebSocket endpoint for preview streaming.
    
    Args:
        websocket: WebSocket connection
    """
    await websocket.accept()
    
    if _preview_bus is None:
        await websocket.close(code=1008, reason="No preview bus available")
        return
    
    try:
        while True:
            frame = await _preview_bus.get(timeout=1.0)
            
            if frame is None:
                # Send heartbeat
                await websocket.send_json({"type": "heartbeat"})
                continue
            
            # Encode and send frame
            from syntriass.preview.stream import FrameEncoder
            encoder = FrameEncoder()
            encoded = encoder.encode_frame(frame)
            await websocket.send_text(encoded)
            
    except WebSocketDisconnect:
        pass


def initialize_api(
    preview_bus: PreviewBus,
    conditioning_injector: ConditioningInjector,
):
    """
    Initialize API with dependencies.
    
    Args:
        preview_bus: PreviewBus instance
        conditioning_injector: ConditioningInjector instance
    """
    global _preview_bus, _conditioning_injector
    _preview_bus = preview_bus
    _conditioning_injector = conditioning_injector

