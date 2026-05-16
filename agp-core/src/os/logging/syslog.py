"""
AGP-OS: System Logger (syslog)
Centralized logging with audit trail and metrics.
"""

import structlog
import json
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from collections import deque
import threading

logger = structlog.get_logger()

class LogLevel(Enum):
    DEBUG = 0
    INFO = 1
    WARNING = 2
    ERROR = 3
    CRITICAL = 4

@dataclass
class LogEntry:
    """A log entry"""
    timestamp: datetime
    level: LogLevel
    source: str  # Module or PID
    message: str
    data: Dict = field(default_factory=dict)
    
    def to_dict(self) -> Dict:
        return {
            "timestamp": self.timestamp.isoformat(),
            "level": self.level.name,
            "source": self.source,
            "message": self.message,
            "data": self.data
        }
    
    def to_json(self) -> str:
        return json.dumps(self.to_dict())

@dataclass
class AuditEntry:
    """An audit trail entry for security-relevant events"""
    timestamp: datetime
    actor_pid: int
    action: str
    target: str
    result: str
    details: Dict = field(default_factory=dict)

class SystemLogger:
    """
    Centralized system logger with rotation and filtering.
    """
    
    def __init__(self, max_entries: int = 10000):
        self.entries: deque = deque(maxlen=max_entries)
        self.audit_trail: deque = deque(maxlen=5000)
        self.level_filter = LogLevel.DEBUG
        self.lock = threading.Lock()
        
        # Subscribers for real-time log streaming
        self.subscribers: List[callable] = []
    
    def log(self, level: LogLevel, source: str, message: str, **data):
        """Log a message"""
        if level.value < self.level_filter.value:
            return
        
        entry = LogEntry(
            timestamp=datetime.utcnow(),
            level=level,
            source=source,
            message=message,
            data=data
        )
        
        with self.lock:
            self.entries.append(entry)
        
        # Notify subscribers
        for subscriber in self.subscribers:
            try:
                subscriber(entry)
            except:
                pass
        
        # Also log to structlog
        log_func = getattr(logger, level.name.lower(), logger.info)
        log_func(message, source=source, **data)
    
    def debug(self, source: str, message: str, **data):
        self.log(LogLevel.DEBUG, source, message, **data)
    
    def info(self, source: str, message: str, **data):
        self.log(LogLevel.INFO, source, message, **data)
    
    def warning(self, source: str, message: str, **data):
        self.log(LogLevel.WARNING, source, message, **data)
    
    def error(self, source: str, message: str, **data):
        self.log(LogLevel.ERROR, source, message, **data)
    
    def critical(self, source: str, message: str, **data):
        self.log(LogLevel.CRITICAL, source, message, **data)
    
    def audit(self, actor_pid: int, action: str, target: str, result: str, **details):
        """Record an audit trail entry"""
        entry = AuditEntry(
            timestamp=datetime.utcnow(),
            actor_pid=actor_pid,
            action=action,
            target=target,
            result=result,
            details=details
        )
        
        with self.lock:
            self.audit_trail.append(entry)
        
        logger.info("audit", actor=actor_pid, action=action, target=target, result=result)
    
    def get_logs(self, level: Optional[LogLevel] = None, source: Optional[str] = None, 
                 limit: int = 100) -> List[Dict]:
        """Get recent logs with optional filtering"""
        with self.lock:
            logs = list(self.entries)
        
        if level:
            logs = [e for e in logs if e.level.value >= level.value]
        if source:
            logs = [e for e in logs if e.source == source]
        
        return [e.to_dict() for e in logs[-limit:]]
    
    def get_audit_trail(self, actor_pid: Optional[int] = None, 
                        action: Optional[str] = None, limit: int = 100) -> List[Dict]:
        """Get audit trail with optional filtering"""
        with self.lock:
            entries = list(self.audit_trail)
        
        if actor_pid:
            entries = [e for e in entries if e.actor_pid == actor_pid]
        if action:
            entries = [e for e in entries if e.action == action]
        
        return [
            {
                "timestamp": e.timestamp.isoformat(),
                "actor_pid": e.actor_pid,
                "action": e.action,
                "target": e.target,
                "result": e.result,
                "details": e.details
            }
            for e in entries[-limit:]
        ]
    
    def subscribe(self, callback: callable):
        """Subscribe to real-time log events"""
        self.subscribers.append(callback)
    
    def set_level(self, level: LogLevel):
        """Set minimum log level"""
        self.level_filter = level

class MetricsCollector:
    """
    Collects and aggregates system metrics.
    """
    
    def __init__(self):
        self.metrics: Dict[str, deque] = {}
        self.counters: Dict[str, int] = {}
        self.gauges: Dict[str, float] = {}
        self.lock = threading.Lock()
    
    def record(self, name: str, value: float):
        """Record a metric value"""
        with self.lock:
            if name not in self.metrics:
                self.metrics[name] = deque(maxlen=1000)
            self.metrics[name].append({
                "timestamp": datetime.utcnow().timestamp(),
                "value": value
            })
    
    def increment(self, name: str, value: int = 1):
        """Increment a counter"""
        with self.lock:
            self.counters[name] = self.counters.get(name, 0) + value
    
    def set_gauge(self, name: str, value: float):
        """Set a gauge value"""
        with self.lock:
            self.gauges[name] = value
    
    def get_metric(self, name: str, last_n: int = 100) -> List[Dict]:
        """Get recent metric values"""
        with self.lock:
            if name not in self.metrics:
                return []
            return list(self.metrics[name])[-last_n:]
    
    def get_counter(self, name: str) -> int:
        """Get counter value"""
        return self.counters.get(name, 0)
    
    def get_gauge(self, name: str) -> Optional[float]:
        """Get gauge value"""
        return self.gauges.get(name)
    
    def get_all_stats(self) -> Dict:
        """Get all metrics summary"""
        with self.lock:
            return {
                "counters": dict(self.counters),
                "gauges": dict(self.gauges),
                "metric_names": list(self.metrics.keys())
            }

# Global instances
syslog = SystemLogger()
metrics = MetricsCollector()
