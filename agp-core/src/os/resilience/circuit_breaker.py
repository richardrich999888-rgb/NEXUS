"""
AGP-OS: Resilience Module
Circuit breaker and retry logic for fault tolerance.
"""

import time
import asyncio
import structlog
from typing import Callable, Optional, Any
from dataclasses import dataclass
from enum import Enum
from functools import wraps

logger = structlog.get_logger()

class CircuitState(Enum):
    CLOSED = "closed"      # Normal operation
    OPEN = "open"          # Failing, reject requests
    HALF_OPEN = "half_open"  # Testing recovery

@dataclass
class CircuitStats:
    """Statistics for a circuit breaker"""
    total_calls: int = 0
    failures: int = 0
    successes: int = 0
    last_failure: float = 0
    last_success: float = 0

class CircuitBreaker:
    """
    Circuit breaker pattern for fault tolerance.
    Opens after threshold failures, closes after recovery.
    """
    
    def __init__(self, name: str, failure_threshold: int = 5,
                 recovery_timeout: float = 30.0):
        self.name = name
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.state = CircuitState.CLOSED
        self.stats = CircuitStats()
        self.consecutive_failures = 0
        self.last_state_change = time.time()
    
    def _check_recovery(self):
        """Check if we should attempt recovery"""
        if self.state == CircuitState.OPEN:
            elapsed = time.time() - self.last_state_change
            if elapsed >= self.recovery_timeout:
                self.state = CircuitState.HALF_OPEN
                logger.info("circuit_half_open", name=self.name)
    
    def can_execute(self) -> bool:
        """Check if the circuit allows execution"""
        self._check_recovery()
        
        if self.state == CircuitState.CLOSED:
            return True
        elif self.state == CircuitState.HALF_OPEN:
            return True  # Allow one request to test
        else:
            return False
    
    def record_success(self):
        """Record a successful call"""
        self.stats.total_calls += 1
        self.stats.successes += 1
        self.stats.last_success = time.time()
        self.consecutive_failures = 0
        
        if self.state == CircuitState.HALF_OPEN:
            self.state = CircuitState.CLOSED
            self.last_state_change = time.time()
            logger.info("circuit_closed", name=self.name)
    
    def record_failure(self):
        """Record a failed call"""
        self.stats.total_calls += 1
        self.stats.failures += 1
        self.stats.last_failure = time.time()
        self.consecutive_failures += 1
        
        if self.consecutive_failures >= self.failure_threshold:
            if self.state != CircuitState.OPEN:
                self.state = CircuitState.OPEN
                self.last_state_change = time.time()
                logger.warning("circuit_open", name=self.name, 
                             failures=self.consecutive_failures)
    
    def get_stats(self) -> dict:
        """Get circuit breaker statistics"""
        return {
            "name": self.name,
            "state": self.state.value,
            "total_calls": self.stats.total_calls,
            "failures": self.stats.failures,
            "successes": self.stats.successes,
            "consecutive_failures": self.consecutive_failures
        }

def with_circuit_breaker(breaker: CircuitBreaker):
    """Decorator to wrap function with circuit breaker"""
    def decorator(func):
        @wraps(func)
        async def async_wrapper(*args, **kwargs):
            if not breaker.can_execute():
                raise CircuitOpenError(f"Circuit {breaker.name} is open")
            
            try:
                result = await func(*args, **kwargs)
                breaker.record_success()
                return result
            except Exception as e:
                breaker.record_failure()
                raise
        
        @wraps(func)
        def sync_wrapper(*args, **kwargs):
            if not breaker.can_execute():
                raise CircuitOpenError(f"Circuit {breaker.name} is open")
            
            try:
                result = func(*args, **kwargs)
                breaker.record_success()
                return result
            except Exception as e:
                breaker.record_failure()
                raise
        
        if asyncio.iscoroutinefunction(func):
            return async_wrapper
        return sync_wrapper
    return decorator

class CircuitOpenError(Exception):
    """Raised when circuit is open"""
    pass

class RetryConfig:
    """Configuration for retry logic"""
    
    def __init__(self, max_retries: int = 3, base_delay: float = 1.0,
                 max_delay: float = 30.0, exponential: bool = True):
        self.max_retries = max_retries
        self.base_delay = base_delay
        self.max_delay = max_delay
        self.exponential = exponential
    
    def get_delay(self, attempt: int) -> float:
        """Get delay for retry attempt"""
        if self.exponential:
            delay = self.base_delay * (2 ** attempt)
        else:
            delay = self.base_delay
        
        return min(delay, self.max_delay)

def with_retry(config: RetryConfig = None, retryable_exceptions: tuple = (Exception,)):
    """Decorator to add retry logic with exponential backoff"""
    if config is None:
        config = RetryConfig()
    
    def decorator(func):
        @wraps(func)
        async def async_wrapper(*args, **kwargs):
            last_exception = None
            
            for attempt in range(config.max_retries + 1):
                try:
                    return await func(*args, **kwargs)
                except retryable_exceptions as e:
                    last_exception = e
                    
                    if attempt < config.max_retries:
                        delay = config.get_delay(attempt)
                        logger.warning("retry_attempt", 
                                      func=func.__name__,
                                      attempt=attempt + 1,
                                      delay=delay,
                                      error=str(e))
                        await asyncio.sleep(delay)
            
            raise last_exception
        
        @wraps(func)
        def sync_wrapper(*args, **kwargs):
            last_exception = None
            
            for attempt in range(config.max_retries + 1):
                try:
                    return func(*args, **kwargs)
                except retryable_exceptions as e:
                    last_exception = e
                    
                    if attempt < config.max_retries:
                        delay = config.get_delay(attempt)
                        logger.warning("retry_attempt",
                                      func=func.__name__,
                                      attempt=attempt + 1,
                                      delay=delay,
                                      error=str(e))
                        time.sleep(delay)
            
            raise last_exception
        
        if asyncio.iscoroutinefunction(func):
            return async_wrapper
        return sync_wrapper
    return decorator

# Pre-configured circuit breakers
llm_circuit = CircuitBreaker("llm_provider", failure_threshold=3, recovery_timeout=60)
network_circuit = CircuitBreaker("network", failure_threshold=5, recovery_timeout=30)
db_circuit = CircuitBreaker("database", failure_threshold=3, recovery_timeout=15)
