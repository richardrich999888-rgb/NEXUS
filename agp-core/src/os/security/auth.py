"""
AGP-OS: Security Module
JWT authentication, access control, and rate limiting.
"""

import jwt
import secrets
import hashlib
import time
import structlog
from typing import Dict, List, Optional, Set
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from collections import defaultdict

logger = structlog.get_logger()

# JWT Configuration
JWT_SECRET = secrets.token_hex(32)  # Generate on first boot
JWT_ALGORITHM = "HS256"
JWT_EXPIRY_HOURS = 24

class Permission(Enum):
    """System permissions"""
    SYSCALL_EXEC = "syscall:exec"
    SYSCALL_FORK = "syscall:fork"
    SYSCALL_KILL = "syscall:kill"
    SYSCALL_MALLOC = "syscall:malloc"
    PROCESS_READ = "process:read"
    PROCESS_WRITE = "process:write"
    FS_READ = "fs:read"
    FS_WRITE = "fs:write"
    KERNEL_ADMIN = "kernel:admin"
    IPC_SEND = "ipc:send"
    IPC_RECEIVE = "ipc:receive"

@dataclass
class Token:
    """JWT Token wrapper"""
    token: str
    kernel_id: str
    process_id: Optional[int]
    permissions: List[str]
    issued_at: datetime
    expires_at: datetime
    
    def is_expired(self) -> bool:
        return datetime.utcnow() > self.expires_at

class AuthManager:
    """
    Handles JWT token generation and verification.
    """
    
    def __init__(self, secret: str = None):
        self.secret = secret or JWT_SECRET
        self.revoked_tokens: Set[str] = set()
    
    def generate_token(self, kernel_id: str, process_id: int = None,
                       permissions: List[str] = None) -> Token:
        """Generate a new JWT token"""
        now = datetime.utcnow()
        expires = now + timedelta(hours=JWT_EXPIRY_HOURS)
        
        if permissions is None:
            # Default permissions for processes
            permissions = [
                Permission.SYSCALL_EXEC.value,
                Permission.PROCESS_READ.value,
                Permission.FS_READ.value,
                Permission.IPC_SEND.value,
                Permission.IPC_RECEIVE.value
            ]
        
        payload = {
            "kernel_id": kernel_id,
            "process_id": process_id,
            "permissions": permissions,
            "iat": now.timestamp(),
            "exp": expires.timestamp(),
            "jti": secrets.token_hex(16)  # Token ID for revocation
        }
        
        token_str = jwt.encode(payload, self.secret, algorithm=JWT_ALGORITHM)
        
        logger.info("token_generated", kernel_id=kernel_id, process_id=process_id)
        
        return Token(
            token=token_str,
            kernel_id=kernel_id,
            process_id=process_id,
            permissions=permissions,
            issued_at=now,
            expires_at=expires
        )
    
    def verify_token(self, token_str: str) -> Optional[Token]:
        """Verify and decode a JWT token"""
        try:
            payload = jwt.decode(token_str, self.secret, algorithms=[JWT_ALGORITHM])
            
            # Check if revoked
            if payload.get("jti") in self.revoked_tokens:
                logger.warning("token_revoked", jti=payload.get("jti"))
                return None
            
            return Token(
                token=token_str,
                kernel_id=payload["kernel_id"],
                process_id=payload.get("process_id"),
                permissions=payload.get("permissions", []),
                issued_at=datetime.fromtimestamp(payload["iat"]),
                expires_at=datetime.fromtimestamp(payload["exp"])
            )
            
        except jwt.ExpiredSignatureError:
            logger.warning("token_expired")
            return None
        except jwt.InvalidTokenError as e:
            logger.warning("token_invalid", error=str(e))
            return None
    
    def revoke_token(self, token_str: str):
        """Revoke a token"""
        try:
            payload = jwt.decode(token_str, self.secret, algorithms=[JWT_ALGORITHM],
                               options={"verify_exp": False})
            self.revoked_tokens.add(payload.get("jti"))
            logger.info("token_revoked", jti=payload.get("jti"))
        except:
            pass
    
    def has_permission(self, token: Token, permission: Permission) -> bool:
        """Check if token has a specific permission"""
        return permission.value in token.permissions or \
               Permission.KERNEL_ADMIN.value in token.permissions

class AccessControl:
    """
    Access Control Lists (ACLs) for resources.
    """
    
    def __init__(self):
        # Resource -> {permission -> set of authorized PIDs}
        self.acls: Dict[str, Dict[str, Set[int]]] = defaultdict(lambda: defaultdict(set))
    
    def grant(self, resource: str, permission: Permission, pid: int):
        """Grant permission to a process"""
        self.acls[resource][permission.value].add(pid)
        logger.info("acl_grant", resource=resource, permission=permission.value, pid=pid)
    
    def revoke(self, resource: str, permission: Permission, pid: int):
        """Revoke permission from a process"""
        self.acls[resource][permission.value].discard(pid)
        logger.info("acl_revoke", resource=resource, permission=permission.value, pid=pid)
    
    def check(self, resource: str, permission: Permission, pid: int) -> bool:
        """Check if process has permission on resource"""
        # Check specific permission
        if pid in self.acls[resource][permission.value]:
            return True
        
        # Check wildcard
        if pid in self.acls[resource]["*"]:
            return True
        
        return False
    
    def get_acl(self, resource: str) -> Dict[str, List[int]]:
        """Get ACL for a resource"""
        return {k: list(v) for k, v in self.acls[resource].items()}

class RateLimiter:
    """
    Token bucket rate limiter per process.
    """
    
    def __init__(self, tokens_per_second: float = 100, bucket_size: int = 1000):
        self.tokens_per_second = tokens_per_second
        self.bucket_size = bucket_size
        self.buckets: Dict[int, float] = {}  # PID -> current tokens
        self.last_update: Dict[int, float] = {}  # PID -> last update time
    
    def _refill(self, pid: int):
        """Refill tokens based on elapsed time"""
        now = time.time()
        last = self.last_update.get(pid, now)
        elapsed = now - last
        
        current = self.buckets.get(pid, self.bucket_size)
        new_tokens = min(self.bucket_size, current + elapsed * self.tokens_per_second)
        
        self.buckets[pid] = new_tokens
        self.last_update[pid] = now
    
    def acquire(self, pid: int, tokens: int = 1) -> bool:
        """
        Try to acquire tokens.
        Returns True if allowed, False if rate limited.
        """
        self._refill(pid)
        
        if self.buckets.get(pid, 0) >= tokens:
            self.buckets[pid] -= tokens
            return True
        
        logger.warning("rate_limited", pid=pid, requested=tokens, available=self.buckets.get(pid, 0))
        return False
    
    def get_remaining(self, pid: int) -> float:
        """Get remaining tokens for a process"""
        self._refill(pid)
        return self.buckets.get(pid, self.bucket_size)
    
    def reset(self, pid: int):
        """Reset rate limit for a process"""
        self.buckets[pid] = self.bucket_size
        self.last_update[pid] = time.time()

# Global instances
auth_manager = AuthManager()
access_control = AccessControl()
rate_limiter = RateLimiter()
