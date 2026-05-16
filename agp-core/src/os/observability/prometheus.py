"""
AGP-OS: Prometheus Observability
Export metrics for monitoring and alerting.
"""

import time
import structlog
from typing import Dict, List
from collections import defaultdict
import threading

logger = structlog.get_logger()

# Check for prometheus client
try:
    from prometheus_client import (
        Counter, Gauge, Histogram, Summary,
        generate_latest, CONTENT_TYPE_LATEST,
        CollectorRegistry, REGISTRY
    )
    HAS_PROMETHEUS = True
except ImportError:
    HAS_PROMETHEUS = False
    logger.info("prometheus_not_installed", message="pip install prometheus-client for metrics")

class PrometheusMetrics:
    """
    Prometheus metrics exporter for AGP-OS.
    """
    
    def __init__(self, registry=None):
        self.registry = registry or (REGISTRY if HAS_PROMETHEUS else None)
        self.metrics = {}
        
        if HAS_PROMETHEUS:
            self._init_metrics()
    
    def _init_metrics(self):
        """Initialize Prometheus metrics"""
        
        # Process metrics
        self.metrics["process_count"] = Gauge(
            "agpos_process_count",
            "Number of processes by state",
            ["state"],
            registry=self.registry
        )
        
        self.metrics["process_priority"] = Gauge(
            "agpos_process_priority",
            "Process priority",
            ["pid", "name"],
            registry=self.registry
        )
        
        # Syscall metrics
        self.metrics["syscall_total"] = Counter(
            "agpos_syscall_total",
            "Total syscall count by type and result",
            ["type", "result"],
            registry=self.registry
        )
        
        self.metrics["syscall_duration"] = Histogram(
            "agpos_syscall_duration_seconds",
            "Syscall duration in seconds",
            ["type"],
            buckets=[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
            registry=self.registry
        )
        
        # Token metrics
        self.metrics["tokens_used"] = Counter(
            "agpos_tokens_used_total",
            "Total tokens used by process",
            ["pid", "name"],
            registry=self.registry
        )
        
        self.metrics["token_quota"] = Gauge(
            "agpos_token_quota",
            "Token quota per process",
            ["pid", "name"],
            registry=self.registry
        )
        
        # IPC metrics
        self.metrics["ipc_messages"] = Counter(
            "agpos_ipc_messages_total",
            "IPC messages sent",
            ["type"],
            registry=self.registry
        )
        
        # Network metrics
        self.metrics["network_peers"] = Gauge(
            "agpos_network_peers",
            "Number of connected peers",
            registry=self.registry
        )
        
        self.metrics["network_messages"] = Counter(
            "agpos_network_messages_total",
            "Network messages sent/received",
            ["direction"],
            registry=self.registry
        )
        
        # Resource metrics
        self.metrics["memory_usage"] = Gauge(
            "agpos_memory_pages",
            "Memory pages used by process",
            ["pid"],
            registry=self.registry
        )
        
        # Circuit breaker metrics
        self.metrics["circuit_state"] = Gauge(
            "agpos_circuit_state",
            "Circuit breaker state (0=closed, 1=half-open, 2=open)",
            ["name"],
            registry=self.registry
        )
        
        # Kernel metrics
        self.metrics["kernel_uptime"] = Gauge(
            "agpos_kernel_uptime_seconds",
            "Kernel uptime in seconds",
            registry=self.registry
        )
        
        self.metrics["checkpoint_count"] = Counter(
            "agpos_checkpoints_total",
            "Total checkpoints created",
            registry=self.registry
        )
        
        logger.info("prometheus_metrics_initialized")
    
    def record_syscall(self, syscall_type: str, duration: float, success: bool):
        """Record a syscall execution"""
        if not HAS_PROMETHEUS:
            return
        
        result = "success" if success else "failure"
        self.metrics["syscall_total"].labels(type=syscall_type, result=result).inc()
        self.metrics["syscall_duration"].labels(type=syscall_type).observe(duration)
    
    def update_process_count(self, state_counts: Dict[str, int]):
        """Update process count by state"""
        if not HAS_PROMETHEUS:
            return
        
        for state, count in state_counts.items():
            self.metrics["process_count"].labels(state=state).set(count)
    
    def update_process_priority(self, pid: int, name: str, priority: float):
        """Update process priority gauge"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["process_priority"].labels(pid=str(pid), name=name).set(priority)
    
    def record_tokens_used(self, pid: int, name: str, tokens: int):
        """Record token usage"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["tokens_used"].labels(pid=str(pid), name=name).inc(tokens)
    
    def update_token_quota(self, pid: int, name: str, quota: int):
        """Update token quota gauge"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["token_quota"].labels(pid=str(pid), name=name).set(quota)
    
    def record_ipc_message(self, msg_type: str):
        """Record an IPC message"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["ipc_messages"].labels(type=msg_type).inc()
    
    def update_network_peers(self, count: int):
        """Update peer count"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["network_peers"].set(count)
    
    def record_network_message(self, direction: str):
        """Record network message (sent/received)"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["network_messages"].labels(direction=direction).inc()
    
    def update_circuit_state(self, name: str, state: int):
        """Update circuit breaker state"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["circuit_state"].labels(name=name).set(state)
    
    def update_kernel_uptime(self, seconds: float):
        """Update kernel uptime"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["kernel_uptime"].set(seconds)
    
    def record_checkpoint(self):
        """Record a checkpoint creation"""
        if not HAS_PROMETHEUS:
            return
        
        self.metrics["checkpoint_count"].inc()
    
    def get_metrics(self) -> bytes:
        """Get metrics in Prometheus format"""
        if not HAS_PROMETHEUS:
            return b"# Prometheus client not installed\n"
        
        return generate_latest(self.registry)
    
    def get_content_type(self) -> str:
        """Get content type for metrics response"""
        if not HAS_PROMETHEUS:
            return "text/plain"
        
        return CONTENT_TYPE_LATEST

# Global metrics instance
prom_metrics = PrometheusMetrics()
