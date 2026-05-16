"""In-memory storage adapter for testing."""
from typing import Dict, Optional, List
from core.types import AgentFingerprint, ReputationRecord, TaskType

class MemoryStorage:
    def __init__(self):
        self._reputation: Dict[bytes, Dict[str, ReputationRecord]] = {}
    
    def store_reputation(self, record: ReputationRecord) -> None:
        fp = record.agent_fingerprint.value
        tt = str(record.task_type)
        if fp not in self._reputation:
            self._reputation[fp] = {}
        self._reputation[fp][tt] = record
    
    def get_reputation(self, fingerprint: AgentFingerprint, task_type: TaskType) -> Optional[ReputationRecord]:
        fp = fingerprint.value
        tt = str(task_type)
        return self._reputation.get(fp, {}).get(tt)
    
    def get_all_reputations(self, fingerprint: AgentFingerprint) -> List[ReputationRecord]:
        fp = fingerprint.value
        return list(self._reputation.get(fp, {}).values())
