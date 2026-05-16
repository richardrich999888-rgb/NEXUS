"""
AGP-OS: Database Persistence Layer
SQLite-backed persistent storage for kernel state.
"""

import sqlite3
import json
import structlog
from typing import Dict, List, Optional, Any
from datetime import datetime
from pathlib import Path
from contextlib import contextmanager
import threading

logger = structlog.get_logger()

class Database:
    """
    SQLite database for persistent kernel state.
    Thread-safe with connection pooling.
    """
    
    def __init__(self, db_path: str = "/tmp/agp-os/kernel.db"):
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self.local = threading.local()
        self._init_schema()
    
    def _get_connection(self) -> sqlite3.Connection:
        """Get thread-local database connection"""
        if not hasattr(self.local, 'conn') or self.local.conn is None:
            self.local.conn = sqlite3.connect(
                str(self.db_path),
                check_same_thread=False
            )
            self.local.conn.row_factory = sqlite3.Row
        return self.local.conn
    
    @contextmanager
    def transaction(self):
        """Context manager for transactions"""
        conn = self._get_connection()
        try:
            yield conn
            conn.commit()
        except Exception as e:
            conn.rollback()
            raise
    
    def _init_schema(self):
        """Initialize database schema"""
        conn = self._get_connection()
        
        conn.executescript("""
            -- Process Control Blocks
            CREATE TABLE IF NOT EXISTS processes (
                pid INTEGER PRIMARY KEY,
                agent_id TEXT NOT NULL,
                name TEXT NOT NULL,
                state TEXT NOT NULL,
                priority REAL NOT NULL,
                nice INTEGER DEFAULT 0,
                quota_tokens INTEGER DEFAULT 100000,
                created_at REAL NOT NULL,
                total_runtime REAL DEFAULT 0,
                cpu_cycles INTEGER DEFAULT 0,
                tokens_used INTEGER DEFAULT 0,
                memory_pages INTEGER DEFAULT 0,
                disk_bytes INTEGER DEFAULT 0,
                last_scheduled_at REAL,
                updated_at REAL NOT NULL
            );
            
            -- Checkpoints
            CREATE TABLE IF NOT EXISTS checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL NOT NULL,
                pid_counter INTEGER NOT NULL,
                kernel_state TEXT NOT NULL,
                created_at REAL NOT NULL
            );
            
            -- Audit Log
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL NOT NULL,
                actor_pid INTEGER,
                action TEXT NOT NULL,
                target TEXT,
                result TEXT,
                details TEXT,
                created_at REAL NOT NULL
            );
            
            -- System Logs
            CREATE TABLE IF NOT EXISTS system_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp REAL NOT NULL,
                level TEXT NOT NULL,
                source TEXT NOT NULL,
                message TEXT NOT NULL,
                data TEXT,
                created_at REAL NOT NULL
            );
            
            -- Message Queue (durable)
            CREATE TABLE IF NOT EXISTS message_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sender_pid INTEGER NOT NULL,
                receiver_pid INTEGER NOT NULL,
                priority INTEGER DEFAULT 1,
                data TEXT NOT NULL,
                delivered INTEGER DEFAULT 0,
                created_at REAL NOT NULL
            );
            
            -- Shared Memory Metadata
            CREATE TABLE IF NOT EXISTS shared_memory (
                segment_id TEXT PRIMARY KEY,
                owner_pid INTEGER NOT NULL,
                size INTEGER NOT NULL,
                created_at REAL NOT NULL
            );
            
            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_processes_state ON processes(state);
            CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_log(actor_pid);
            CREATE INDEX IF NOT EXISTS idx_mq_receiver ON message_queue(receiver_pid, delivered);
            CREATE INDEX IF NOT EXISTS idx_logs_level ON system_logs(level);
        """)
        
        conn.commit()
        logger.info("database_initialized", path=str(self.db_path))
    
    # Process Operations
    
    def save_process(self, pcb) -> bool:
        """Save or update a process"""
        with self.transaction() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO processes 
                (pid, agent_id, name, state, priority, nice, quota_tokens,
                 created_at, total_runtime, cpu_cycles, tokens_used,
                 memory_pages, disk_bytes, last_scheduled_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                pcb.pid, pcb.agent_id, pcb.name, pcb.state.value,
                pcb.priority, pcb.nice, pcb.quota_tokens, pcb.created_at,
                pcb.total_runtime, pcb.usage.cpu_cycles, pcb.usage.tokens_used,
                pcb.usage.memory_pages, pcb.usage.disk_bytes,
                pcb.last_scheduled_at, datetime.utcnow().timestamp()
            ))
        return True
    
    def load_process(self, pid: int) -> Optional[Dict]:
        """Load a process by PID"""
        conn = self._get_connection()
        row = conn.execute(
            "SELECT * FROM processes WHERE pid = ?", (pid,)
        ).fetchone()
        return dict(row) if row else None
    
    def load_all_processes(self) -> List[Dict]:
        """Load all processes"""
        conn = self._get_connection()
        rows = conn.execute("SELECT * FROM processes").fetchall()
        return [dict(row) for row in rows]
    
    def delete_process(self, pid: int) -> bool:
        """Delete a process"""
        with self.transaction() as conn:
            conn.execute("DELETE FROM processes WHERE pid = ?", (pid,))
        return True
    
    # Checkpoint Operations
    
    def save_checkpoint(self, checkpoint) -> int:
        """Save a checkpoint"""
        with self.transaction() as conn:
            cursor = conn.execute("""
                INSERT INTO checkpoints (timestamp, pid_counter, kernel_state, created_at)
                VALUES (?, ?, ?, ?)
            """, (
                checkpoint.timestamp.timestamp(),
                checkpoint.pid_counter,
                json.dumps(checkpoint.process_states),
                datetime.utcnow().timestamp()
            ))
            return cursor.lastrowid
    
    def load_latest_checkpoint(self) -> Optional[Dict]:
        """Load most recent checkpoint"""
        conn = self._get_connection()
        row = conn.execute("""
            SELECT * FROM checkpoints ORDER BY timestamp DESC LIMIT 1
        """).fetchone()
        return dict(row) if row else None
    
    # Audit Operations
    
    def log_audit(self, actor_pid: int, action: str, target: str, 
                  result: str, details: Dict = None):
        """Log an audit event"""
        with self.transaction() as conn:
            conn.execute("""
                INSERT INTO audit_log (timestamp, actor_pid, action, target, result, details, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                datetime.utcnow().timestamp(),
                actor_pid, action, target, result,
                json.dumps(details) if details else None,
                datetime.utcnow().timestamp()
            ))
    
    def get_audit_log(self, limit: int = 100, actor_pid: int = None) -> List[Dict]:
        """Get audit log entries"""
        conn = self._get_connection()
        if actor_pid:
            rows = conn.execute(
                "SELECT * FROM audit_log WHERE actor_pid = ? ORDER BY timestamp DESC LIMIT ?",
                (actor_pid, limit)
            ).fetchall()
        else:
            rows = conn.execute(
                "SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT ?",
                (limit,)
            ).fetchall()
        return [dict(row) for row in rows]
    
    # Message Queue Operations
    
    def enqueue_message(self, sender_pid: int, receiver_pid: int, 
                        data: Any, priority: int = 1) -> int:
        """Enqueue a durable message"""
        with self.transaction() as conn:
            cursor = conn.execute("""
                INSERT INTO message_queue (sender_pid, receiver_pid, priority, data, created_at)
                VALUES (?, ?, ?, ?, ?)
            """, (
                sender_pid, receiver_pid, priority,
                json.dumps(data),
                datetime.utcnow().timestamp()
            ))
            return cursor.lastrowid
    
    def dequeue_message(self, receiver_pid: int) -> Optional[Dict]:
        """Dequeue a message for a receiver"""
        with self.transaction() as conn:
            row = conn.execute("""
                SELECT * FROM message_queue 
                WHERE receiver_pid = ? AND delivered = 0
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
            """, (receiver_pid,)).fetchone()
            
            if row:
                conn.execute(
                    "UPDATE message_queue SET delivered = 1 WHERE id = ?",
                    (row['id'],)
                )
                return dict(row)
        return None
    
    # Statistics
    
    def get_stats(self) -> Dict:
        """Get database statistics"""
        conn = self._get_connection()
        
        process_count = conn.execute(
            "SELECT COUNT(*) FROM processes"
        ).fetchone()[0]
        
        checkpoint_count = conn.execute(
            "SELECT COUNT(*) FROM checkpoints"
        ).fetchone()[0]
        
        audit_count = conn.execute(
            "SELECT COUNT(*) FROM audit_log"
        ).fetchone()[0]
        
        pending_messages = conn.execute(
            "SELECT COUNT(*) FROM message_queue WHERE delivered = 0"
        ).fetchone()[0]
        
        return {
            "process_count": process_count,
            "checkpoint_count": checkpoint_count,
            "audit_count": audit_count,
            "pending_messages": pending_messages,
            "db_path": str(self.db_path)
        }

# Global database instance
db = Database()
