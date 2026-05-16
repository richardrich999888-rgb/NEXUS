"""
AGP-OS Memory Management (Context Paging)
Manages the "RAM" (Context Window) of Agent Processes.
"""

import structlog
from typing import List, Dict, Optional
from datetime import datetime

from src.os.process import ProcessControlBlock
from src.ml.rag_engine import rag_engine

logger = structlog.get_logger()

class ContextPage:
    """A page of memory (conversation history)"""
    def __init__(self, content: str, timestamp: float, access_count: int = 0):
        self.content = content
        self.timestamp = timestamp
        self.access_count = access_count
        self.dirty = False # If modified, needs write-back to RAG

class ContextManager:
    """
    Manages the Context Window (RAM) for processes.
    Implements Paging (Swap to RAG).
    """
    
    def __init__(self, page_size: int = 500): # 500 tokens per page approx
        self.page_size = page_size
        # Map PID -> List[ContextPage]
        self.active_pages: Dict[int, List[ContextPage]] = {}
    
    def allocate(self, pcb: ProcessControlBlock):
        """Allocate initial memory for a process"""
        self.active_pages[pcb.pid] = []
        logger.info("memory_alloc", pid=pcb.pid)

    def write(self, pcb: ProcessControlBlock, content: str):
        """
        Write content to process memory.
        If full, trigger SWAP OUT.
        """
        if pcb.pid not in self.active_pages:
            self.allocate(pcb)
            
        pages = self.active_pages[pcb.pid]
        
        # Create new page
        new_page = ContextPage(content, datetime.now().timestamp())
        pages.append(new_page)
        
        # Check limit (e.g., 10 pages max in RAM)
        MAX_PAGES = 10
        if len(pages) > MAX_PAGES:
            self.swap_out(pcb, pages[0]) # FIFO Eviction (Simple)
            pages.pop(0)
            
        # Update PCB usage
        pcb.usage.memory_pages = len(pages)

    def swap_out(self, pcb: ProcessControlBlock, page: ContextPage):
        """
        Swap a page out to Disk (RAG Vector Store).
        """
        logger.info("memory_swap_out", pid=pcb.pid, timestamp=page.timestamp)
        
        # Persist to RAG
        rag_engine.add_knowledge(
            text=page.content,
            category=f"memory_swap_pid_{pcb.pid}",
            metadata={
                "pid": pcb.pid,
                "agent": pcb.name,
                "timestamp": page.timestamp,
                "type": "swap_file"
            }
        )
        
        # Update Disk Usage
        pcb.usage.disk_bytes += len(page.content)

    def page_fault(self, pcb: ProcessControlBlock, query: str) -> str:
        """
        Handle Page Fault: Retrieve context from Swap (RAG)
        """
        logger.info("memory_page_fault", pid=pcb.pid, query=query)
        
        # Retrieve from RAG
        # We assume RAG retrieval is the "Page In" operation
        context = rag_engine.retrieve_context(
            query, 
            limit=3, 
            # Filter by specific process memory if possible, or relevant knowledge
        )
        
        return "\n".join(context)

# Global Context Manager
context_manager = ContextManager()
