"""
AGP-OS: Inter-Process Communication Module
"""

from .message_queue import (
    MessageQueue,
    MessageQueueManager,
    Message,
    MessagePriority,
    mq_manager
)

from .signals import (
    Signal,
    SignalInfo,
    SignalHandler,
    signal_handler
)

from .shared_memory import (
    SharedMemorySegment,
    SharedMemoryManager,
    shm_manager
)

__all__ = [
    # Message Queues
    "MessageQueue",
    "MessageQueueManager",
    "Message",
    "MessagePriority",
    "mq_manager",
    
    # Signals
    "Signal",
    "SignalInfo",
    "SignalHandler",
    "signal_handler",
    
    # Shared Memory
    "SharedMemorySegment",
    "SharedMemoryManager",
    "shm_manager"
]
