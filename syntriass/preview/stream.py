"""
SYNTRIASS Path 6 — Preview Bus (Streaming Backbone)

Responsibilities:
- Collect preview frames
- Throttle intelligently
- Stream to frontend

Implementation:
- Async queue
- WebSocket
- Drop frames if needed (never block)

Low-tech. Rock solid.
"""

import asyncio
from asyncio import Queue
from typing import Optional, Dict, Any
from dataclasses import dataclass
import time
import json
import base64
from PIL import Image
import io
import numpy as np


@dataclass
class PreviewFrame:
    """Single preview frame"""
    image: np.ndarray  # [H, W, 3] uint8
    step_idx: int
    timestamp: float
    metadata: Dict[str, Any]


class PreviewBus:
    """
    Async queue for preview frames with intelligent throttling.
    
    Never blocks. Drops frames if queue is full.
    Streams to connected clients via WebSocket.
    """
    
    def __init__(
        self,
        max_queue_size: int = 10,
        target_fps: float = 15.0,  # Target preview FPS
    ):
        """
        Args:
            max_queue_size: Maximum queue size (drops if full)
            target_fps: Target frames per second
        """
        self.queue: Queue = Queue(maxsize=max_queue_size)
        self.target_fps = target_fps
        self.min_frame_interval = 1.0 / target_fps
        self._last_frame_time = 0.0
        self._dropped_frames = 0
        self._total_frames = 0
        
    async def push(
        self,
        image: np.ndarray,
        step_idx: int,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> bool:
        """
        Push preview frame to queue (non-blocking).
        
        Args:
            image: Image array [H, W, 3] uint8
            step_idx: Step index
            metadata: Optional metadata
            
        Returns:
            True if pushed, False if dropped
        """
        self._total_frames += 1
        
        # Throttle: respect target FPS
        current_time = time.time()
        time_since_last = current_time - self._last_frame_time
        
        if time_since_last < self.min_frame_interval:
            # Too soon, drop frame
            self._dropped_frames += 1
            return False
        
        frame = PreviewFrame(
            image=image,
            step_idx=step_idx,
            timestamp=current_time,
            metadata=metadata or {},
        )
        
        # Try to push (non-blocking)
        try:
            self.queue.put_nowait(frame)
            self._last_frame_time = current_time
            return True
        except asyncio.QueueFull:
            # Queue full, drop frame
            self._dropped_frames += 1
            return False
    
    async def get(self, timeout: Optional[float] = None) -> Optional[PreviewFrame]:
        """
        Get next preview frame from queue.
        
        Args:
            timeout: Max time to wait (None = no timeout)
            
        Returns:
            PreviewFrame or None if timeout
        """
        try:
            if timeout is None:
                return await self.queue.get()
            else:
                return await asyncio.wait_for(self.queue.get(), timeout=timeout)
        except asyncio.TimeoutError:
            return None
    
    def get_stats(self) -> Dict[str, Any]:
        """Get bus statistics"""
        return {
            "queue_size": self.queue.qsize(),
            "total_frames": self._total_frames,
            "dropped_frames": self._dropped_frames,
            "drop_rate": self._dropped_frames / max(1, self._total_frames),
            "target_fps": self.target_fps,
        }
    
    def clear(self):
        """Clear queue"""
        while not self.queue.empty():
            try:
                self.queue.get_nowait()
            except:
                pass


class FrameEncoder:
    """Encodes preview frames for WebSocket transmission"""
    
    @staticmethod
    def encode_frame(frame: PreviewFrame, format: str = "jpeg", quality: int = 85) -> str:
        """
        Encode frame to JSON-serializable format.
        
        Args:
            frame: PreviewFrame
            format: Image format ("jpeg", "png", "webp")
            quality: JPEG quality (1-100)
            
        Returns:
            JSON string
        """
        # Convert numpy array to PIL Image
        image = Image.fromarray(frame.image)
        
        # Encode to bytes
        buffer = io.BytesIO()
        image.save(buffer, format=format, quality=quality)
        image_bytes = buffer.getvalue()
        
        # Base64 encode
        image_b64 = base64.b64encode(image_bytes).decode('utf-8')
        
        # Create JSON payload
        payload = {
            "type": "preview_frame",
            "step_idx": frame.step_idx,
            "timestamp": frame.timestamp,
            "image": image_b64,
            "format": format,
            "metadata": frame.metadata,
        }
        
        return json.dumps(payload)
    
    @staticmethod
    def decode_frame(json_str: str) -> PreviewFrame:
        """
        Decode frame from JSON string.
        
        Args:
            json_str: JSON string from encode_frame
            
        Returns:
            PreviewFrame
        """
        payload = json.loads(json_str)
        
        # Decode image
        image_b64 = payload["image"]
        image_bytes = base64.b64decode(image_b64)
        image = Image.open(io.BytesIO(image_bytes))
        image_array = np.array(image)
        
        return PreviewFrame(
            image=image_array,
            step_idx=payload["step_idx"],
            timestamp=payload["timestamp"],
            metadata=payload.get("metadata", {}),
        )


async def stream_preview(
    preview_bus: PreviewBus,
    websocket,
    encoder: FrameEncoder,
):
    """
    Stream preview frames to WebSocket client.
    
    Args:
        preview_bus: PreviewBus instance
        websocket: WebSocket connection
        encoder: FrameEncoder instance
    """
    try:
        while True:
            frame = await preview_bus.get(timeout=1.0)
            
            if frame is None:
                # Send heartbeat
                await websocket.send(json.dumps({"type": "heartbeat"}))
                continue
            
            # Encode and send
            encoded = encoder.encode_frame(frame)
            await websocket.send(encoded)
            
    except asyncio.CancelledError:
        pass
    except Exception as e:
        print(f"Stream error: {e}")

