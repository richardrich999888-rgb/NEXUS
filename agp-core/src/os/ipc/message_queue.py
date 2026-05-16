"""
AGP-OS: Inter-Process Communication (IPC)
Message queues for agent-to-agent communication.
"""

import asyncio
import structlog
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
import heapq

logger = structlog.get_logger()

class MessagePriority(Enum):
    LOW = 0
    NORMAL = 1
    HIGH = 2
    URGENT = 3

@dataclass
class Message:
    """A message sent between processes"""
    sender_pid: int
    receiver_pid: int
    data: Any
    priority: MessagePriority = MessagePriority.NORMAL
    timestamp: float = field(default_factory=lambda: datetime.now().timestamp())
    
    def __lt__(self, other):
        """For priority queue ordering"""
        if self.priority.value != other.priority.value:
            return self.priority.value > other.priority.value  # Higher priority first
        return self.timestamp < other.timestamp

class MessageQueue:
    """
    Message queue for inter-process communication.
    Supports priority-based delivery and blocking/non-blocking reads.
    """
    
    def __init__(self, queue_id: str):
        self.queue_id = queue_id
        self.messages: List[Message] = []
        self.waiters: List[asyncio.Future] = []
        self.max_size = 1000
    
    def send(self, message: Message) -> bool:
        """
        Send a message to the queue.
        Returns True if successful, False if queue is full.
        """
        if len(self.messages) >= self.max_size:
            logger.warning("message_queue_full", queue_id=self.queue_id)
            return False
        
        heapq.heappush(self.messages, message)
        
        # Wake up any waiting receivers
        if self.waiters:
            waiter = self.waiters.pop(0)
            if not waiter.done():
                waiter.set_result(True)
        
        logger.info(
            "message_sent",
            queue_id=self.queue_id,
            sender=message.sender_pid,
            receiver=message.receiver_pid,
            priority=message.priority.name
        )
        return True
    
    async def receive(self, block: bool = True, timeout: Optional[float] = None) -> Optional[Message]:
        """
        Receive a message from the queue.
        If block=True, waits for a message.
        If timeout is set, waits at most that many seconds.
        """
        # Check if message available
        if self.messages:
            message = heapq.heappop(self.messages)
            logger.info(
                "message_received",
                queue_id=self.queue_id,
                sender=message.sender_pid,
                receiver=message.receiver_pid
            )
            return message
        
        # Non-blocking mode
        if not block:
            return None
        
        # Blocking mode - wait for message
        future = asyncio.Future()
        self.waiters.append(future)
        
        try:
            if timeout:
                await asyncio.wait_for(future, timeout=timeout)
            else:
                await future
            
            # Message should be available now
            if self.messages:
                return heapq.heappop(self.messages)
            return None
            
        except asyncio.TimeoutError:
            # Remove from waiters if still there
            if future in self.waiters:
                self.waiters.remove(future)
            return None
    
    def peek(self) -> Optional[Message]:
        """Peek at next message without removing it"""
        return self.messages[0] if self.messages else None
    
    def size(self) -> int:
        """Get current queue size"""
        return len(self.messages)
    
    def clear(self):
        """Clear all messages"""
        self.messages.clear()

class MessageQueueManager:
    """
    Global manager for all message queues.
    Processes can create queues and send/receive messages.
    """
    
    def __init__(self):
        self.queues: Dict[str, MessageQueue] = {}
    
    def create_queue(self, queue_id: str) -> MessageQueue:
        """Create a new message queue"""
        if queue_id in self.queues:
            return self.queues[queue_id]
        
        queue = MessageQueue(queue_id)
        self.queues[queue_id] = queue
        logger.info("queue_created", queue_id=queue_id)
        return queue
    
    def get_queue(self, queue_id: str) -> Optional[MessageQueue]:
        """Get an existing queue"""
        return self.queues.get(queue_id)
    
    def delete_queue(self, queue_id: str) -> bool:
        """Delete a queue"""
        if queue_id in self.queues:
            del self.queues[queue_id]
            logger.info("queue_deleted", queue_id=queue_id)
            return True
        return False
    
    def send_message(
        self,
        sender_pid: int,
        receiver_pid: int,
        data: Any,
        priority: MessagePriority = MessagePriority.NORMAL
    ) -> bool:
        """
        Send a message to a process.
        Creates a queue named "pid_<receiver_pid>" if it doesn't exist.
        """
        queue_id = f"pid_{receiver_pid}"
        queue = self.get_queue(queue_id) or self.create_queue(queue_id)
        
        message = Message(
            sender_pid=sender_pid,
            receiver_pid=receiver_pid,
            data=data,
            priority=priority
        )
        
        return queue.send(message)
    
    async def receive_message(
        self,
        receiver_pid: int,
        block: bool = True,
        timeout: Optional[float] = None
    ) -> Optional[Message]:
        """
        Receive a message for a process.
        """
        queue_id = f"pid_{receiver_pid}"
        queue = self.get_queue(queue_id)
        
        if not queue:
            if not block:
                return None
            # Create queue and wait
            queue = self.create_queue(queue_id)
        
        return await queue.receive(block=block, timeout=timeout)
    
    def broadcast(
        self,
        sender_pid: int,
        pids: List[int],
        data: Any,
        priority: MessagePriority = MessagePriority.NORMAL
    ):
        """Broadcast a message to multiple processes"""
        for pid in pids:
            self.send_message(sender_pid, pid, data, priority)

# Global message queue manager
mq_manager = MessageQueueManager()
