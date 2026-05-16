"""
AGP-OS: Networking Stack
WebSocket-based inter-kernel communication for distributed AGP-OS.
"""

import asyncio
import json
import structlog
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum

logger = structlog.get_logger()

class MessageType(Enum):
    """Types of network messages"""
    HEARTBEAT = "heartbeat"
    SYSCALL = "syscall"
    PROCESS_MIGRATE = "process_migrate"
    PROC_QUERY = "proc_query"
    BROADCAST = "broadcast"

@dataclass
class NetworkMessage:
    """A message sent between kernels"""
    type: MessageType
    source_kernel: str
    target_kernel: str
    payload: Dict
    timestamp: float = field(default_factory=lambda: datetime.now().timestamp())
    message_id: str = ""
    
    def to_json(self) -> str:
        return json.dumps({
            "type": self.type.value,
            "source_kernel": self.source_kernel,
            "target_kernel": self.target_kernel,
            "payload": self.payload,
            "timestamp": self.timestamp,
            "message_id": self.message_id
        })
    
    @classmethod
    def from_json(cls, data: str) -> 'NetworkMessage':
        d = json.loads(data)
        return cls(
            type=MessageType(d["type"]),
            source_kernel=d["source_kernel"],
            target_kernel=d["target_kernel"],
            payload=d["payload"],
            timestamp=d["timestamp"],
            message_id=d.get("message_id", "")
        )

@dataclass
class KernelPeer:
    """A remote kernel peer"""
    kernel_id: str
    address: str
    port: int
    connected: bool = False
    last_heartbeat: float = 0
    process_count: int = 0

class NetworkManager:
    """
    Manages network connections between AGP-OS kernels.
    Enables distributed process management and cross-kernel communication.
    """
    
    def __init__(self, kernel_id: str, port: int = 8765):
        self.kernel_id = kernel_id
        self.port = port
        self.peers: Dict[str, KernelPeer] = {}
        self.message_handlers: Dict[MessageType, Callable] = {}
        self.server = None
        self.running = False
        
        # Register default handlers
        self._register_default_handlers()
    
    def _register_default_handlers(self):
        """Register default message handlers"""
        self.message_handlers[MessageType.HEARTBEAT] = self._handle_heartbeat
        self.message_handlers[MessageType.PROC_QUERY] = self._handle_proc_query
    
    async def _handle_heartbeat(self, msg: NetworkMessage) -> Dict:
        """Handle heartbeat from peer"""
        from src.os.kernel import kernel
        peer_id = msg.source_kernel
        
        if peer_id in self.peers:
            self.peers[peer_id].last_heartbeat = datetime.now().timestamp()
            self.peers[peer_id].connected = True
        
        return {
            "status": "alive",
            "process_count": len(kernel.process_table),
            "kernel_id": self.kernel_id
        }
    
    async def _handle_proc_query(self, msg: NetworkMessage) -> Dict:
        """Handle /proc query from remote kernel"""
        from src.os.kernel import kernel
        
        pid = msg.payload.get("pid")
        if pid and pid in kernel.process_table:
            pcb = kernel.process_table[pid]
            return {
                "pid": pid,
                "name": pcb.name,
                "state": pcb.state.value,
                "priority": pcb.priority
            }
        else:
            # Return all processes
            return {
                "processes": [
                    {"pid": p.pid, "name": p.name, "state": p.state.value}
                    for p in kernel.process_table.values()
                ]
            }
    
    def register_peer(self, kernel_id: str, address: str, port: int):
        """Register a remote kernel peer"""
        self.peers[kernel_id] = KernelPeer(
            kernel_id=kernel_id,
            address=address,
            port=port
        )
        logger.info("peer_registered", peer=kernel_id, address=f"{address}:{port}")
    
    async def send_message(self, target_kernel: str, msg_type: MessageType, payload: Dict) -> Optional[Dict]:
        """Send a message to a remote kernel"""
        if target_kernel not in self.peers:
            logger.warning("unknown_peer", target=target_kernel)
            return None
        
        peer = self.peers[target_kernel]
        
        message = NetworkMessage(
            type=msg_type,
            source_kernel=self.kernel_id,
            target_kernel=target_kernel,
            payload=payload,
            message_id=f"{self.kernel_id}_{datetime.now().timestamp()}"
        )
        
        # Simulate network call (in real implementation, use WebSocket)
        logger.info("network_send", target=target_kernel, type=msg_type.value)
        
        # For now, return simulated response
        return {"status": "sent", "message_id": message.message_id}
    
    async def broadcast(self, msg_type: MessageType, payload: Dict):
        """Broadcast message to all peers"""
        for peer_id in self.peers:
            await self.send_message(peer_id, msg_type, payload)
    
    def register_handler(self, msg_type: MessageType, handler: Callable):
        """Register a custom message handler"""
        self.message_handlers[msg_type] = handler
    
    async def start_server(self):
        """Start the network server"""
        self.running = True
        logger.info("network_server_start", port=self.port, kernel_id=self.kernel_id)
        # In real implementation: start WebSocket server
    
    async def stop_server(self):
        """Stop the network server"""
        self.running = False
        logger.info("network_server_stop", kernel_id=self.kernel_id)
    
    def get_peer_status(self) -> List[Dict]:
        """Get status of all peers"""
        return [
            {
                "kernel_id": p.kernel_id,
                "address": f"{p.address}:{p.port}",
                "connected": p.connected,
                "last_heartbeat": p.last_heartbeat,
                "process_count": p.process_count
            }
            for p in self.peers.values()
        ]

# Global network manager (initialized with default kernel ID)
network_manager = NetworkManager("kernel_main")
