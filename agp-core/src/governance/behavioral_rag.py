"""
AGP-CORE: Behavioral RAG Layer
Stores and retrieves agent behaviors for governance.
"""

import uuid
import hashlib
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum

import structlog

logger = structlog.get_logger()

class ActionType(Enum):
    """Types of agent actions"""
    EXECUTE = "execute"
    DECIDE = "decide"
    COLLABORATE = "collaborate"
    SYSCALL = "syscall"
    COMMUNICATE = "communicate"

class Outcome(Enum):
    """Outcome of an action"""
    SUCCESS = "success"
    FAILURE = "failure"
    PARTIAL = "partial"
    BLOCKED = "blocked"

@dataclass
class BehaviorRecord:
    """
    A record of agent behavior stored in RAG.
    This is the fundamental unit of governance data.
    """
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    agent_id: str = ""
    agent_name: str = ""
    action_type: ActionType = ActionType.EXECUTE
    input_summary: str = ""
    input_hash: str = ""
    output_summary: str = ""
    output_hash: str = ""
    outcome: Outcome = Outcome.SUCCESS
    tokens_used: int = 0
    latency_ms: int = 0
    context: Dict = field(default_factory=dict)
    embedding: List[float] = field(default_factory=list)
    timestamp: datetime = field(default_factory=datetime.utcnow)
    
    def to_text(self) -> str:
        """Convert behavior to text for embedding"""
        return f"{self.agent_name} {self.action_type.value}: {self.input_summary} → {self.outcome.value}"
    
    def to_dict(self) -> Dict:
        """Serialize for storage"""
        return {
            "id": self.id,
            "agent_id": self.agent_id,
            "agent_name": self.agent_name,
            "action_type": self.action_type.value,
            "input_summary": self.input_summary,
            "input_hash": self.input_hash,
            "output_summary": self.output_summary,
            "output_hash": self.output_hash,
            "outcome": self.outcome.value,
            "tokens_used": self.tokens_used,
            "latency_ms": self.latency_ms,
            "context": self.context,
            "timestamp": self.timestamp.isoformat()
        }

class BehavioralRAG:
    """
    RAG layer for storing and retrieving agent behaviors.
    Foundation for Agent Governance Protocol.
    """
    
    def __init__(self):
        # Use existing RAG engine for embeddings and storage
        from src.ml.rag_engine import RAGEngine
        self.rag = RAGEngine()
        
        # In-memory behavior index (for fast agent-specific lookups)
        self.behaviors: Dict[str, List[BehaviorRecord]] = {}  # agent_id -> behaviors
        self.all_behaviors: List[BehaviorRecord] = []
        
        logger.info("behavioral_rag_initialized")
    
    def store_behavior(self, record: BehaviorRecord) -> str:
        """
        Store a behavior record in RAG.
        Returns the behavior ID.
        """
        # Generate embedding for the behavior
        text = record.to_text()
        embedding = self.rag.embedding_service.embed_text(text)
        record.embedding = embedding
        
        # Store in RAG engine
        self.rag.add_knowledge(
            text=text,
            category="behavior",
            metadata={
                "behavior_id": record.id,
                "agent_id": record.agent_id,
                "action_type": record.action_type.value,
                "outcome": record.outcome.value,
                "tokens_used": record.tokens_used,
                "timestamp": record.timestamp.isoformat()
            }
        )
        
        # Index by agent
        if record.agent_id not in self.behaviors:
            self.behaviors[record.agent_id] = []
        self.behaviors[record.agent_id].append(record)
        self.all_behaviors.append(record)
        
        logger.info("behavior_stored", 
                   agent=record.agent_name, 
                   action=record.action_type.value,
                   outcome=record.outcome.value)
        
        return record.id
    
    def retrieve_by_agent(self, agent_id: str, limit: int = 50) -> List[BehaviorRecord]:
        """
        Retrieve behaviors for a specific agent.
        Returns most recent behaviors first.
        """
        behaviors = self.behaviors.get(agent_id, [])
        return sorted(behaviors, key=lambda b: b.timestamp, reverse=True)[:limit]
    
    def retrieve_similar(self, query: str, limit: int = 10, 
                         agent_id: Optional[str] = None) -> List[BehaviorRecord]:
        """
        Retrieve behaviors similar to a query.
        Optionally filter by agent.
        """
        # Get embedding for query
        query_embedding = self.rag.embedding_service.embed_text(query)
        
        # Search in FAISS
        results = self.rag.faiss_store.search(query_embedding, k=limit * 2)
        
        # Filter and map back to behavior records
        matched_behaviors = []
        for result in results:
            meta = result.get("metadata", {})
            behavior_id = meta.get("behavior_id")
            
            if agent_id and meta.get("agent_id") != agent_id:
                continue
            
            # Find the behavior record
            for b in self.all_behaviors:
                if b.id == behavior_id:
                    matched_behaviors.append(b)
                    break
            
            if len(matched_behaviors) >= limit:
                break
        
        return matched_behaviors
    
    def get_agent_stats(self, agent_id: str) -> Dict:
        """
        Get statistics for an agent's behavioral history.
        """
        behaviors = self.behaviors.get(agent_id, [])
        
        if not behaviors:
            return {
                "total_actions": 0,
                "success_rate": 0.0,
                "failure_rate": 0.0,
                "avg_latency_ms": 0,
                "total_tokens": 0
            }
        
        successes = sum(1 for b in behaviors if b.outcome == Outcome.SUCCESS)
        failures = sum(1 for b in behaviors if b.outcome == Outcome.FAILURE)
        total = len(behaviors)
        
        return {
            "total_actions": total,
            "success_rate": successes / total,
            "failure_rate": failures / total,
            "avg_latency_ms": sum(b.latency_ms for b in behaviors) / total,
            "total_tokens": sum(b.tokens_used for b in behaviors),
            "action_breakdown": {
                at.value: sum(1 for b in behaviors if b.action_type == at)
                for at in ActionType
            }
        }
    
    def record_from_syscall(self, agent_id: str, agent_name: str,
                            syscall_type: str, args: Dict,
                            result: Dict, latency_ms: int) -> BehaviorRecord:
        """
        Create a behavior record from a syscall execution.
        """
        # Determine outcome
        if result.get("success") or result.get("status") == "success":
            outcome = Outcome.SUCCESS
        elif result.get("error"):
            if "EACCES" in str(result.get("error", "")):
                outcome = Outcome.BLOCKED
            else:
                outcome = Outcome.FAILURE
        else:
            outcome = Outcome.PARTIAL
        
        # Create input hash
        input_text = f"{syscall_type}:{str(args)[:100]}"
        input_hash = hashlib.sha256(input_text.encode()).hexdigest()[:16]
        
        # Create output hash
        output_text = str(result)[:200]
        output_hash = hashlib.sha256(output_text.encode()).hexdigest()[:16]
        
        record = BehaviorRecord(
            agent_id=agent_id,
            agent_name=agent_name,
            action_type=ActionType.SYSCALL,
            input_summary=f"SYSCALL:{syscall_type}",
            input_hash=input_hash,
            output_summary=output_text[:100],
            output_hash=output_hash,
            outcome=outcome,
            tokens_used=result.get("tokens_used", 0),
            latency_ms=latency_ms,
            context={"syscall_type": syscall_type, "args": args}
        )
        
        self.store_behavior(record)
        return record

# Global instance
behavioral_rag = BehavioralRAG()
