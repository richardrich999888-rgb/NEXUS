"""
SWARM IMMUNITY - GOSSIP PROTOCOL
Cryptographically signed threat exchange (IDF-006).
"""
import time
import json
import base64
from typing import Dict, Any, Optional
from dataclasses import dataclass

# Using generic hashing/signing for MVP bridge
# In production this bridges to multi-asi-immune Rust crate
import hashlib
import hmac

@dataclass
class ThreatReport:
    """A signed claim about a threat."""
    reporter_id: str
    threat_hash: str # Content hash of the threat pattern
    threat_type: str
    severity: float
    timestamp: float
    signature: str = ""
    
    def sign(self, secret_key: str):
        """Sign the report (HMAC for MVP)."""
        payload = f"{self.reporter_id}:{self.threat_hash}:{self.severity}:{self.timestamp}"
        self.signature = hmac.new(
            secret_key.encode(), 
            payload.encode(), 
            hashlib.sha256
        ).hexdigest()
        
    def verify(self, secret_key: str) -> bool:
        """Verify the signature."""
        payload = f"{self.reporter_id}:{self.threat_hash}:{self.severity}:{self.timestamp}"
        expected = hmac.new(
            secret_key.encode(), 
            payload.encode(), 
            hashlib.sha256
        ).hexdigest()
        return hmac.compare_digest(self.signature, expected)
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "reporter_id": self.reporter_id,
            "threat_hash": self.threat_hash,
            "threat_type": self.threat_type,
            "severity": self.severity,
            "timestamp": self.timestamp,
            "signature": self.signature
        }
        
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ThreatReport':
        return cls(**data)


class GossipProtocol:
    """
    Handles peer-to-peer exchange of threat intelligence.
    """
    def __init__(self, agent_id: str, shared_secret: str):
        self.agent_id = agent_id
        self.secret = shared_secret # In real mesh, this is Public Key
        self.known_reports: Dict[str, ThreatReport] = {}
        
    def create_report(self, threat_type: str, threat_data: Any, severity: float) -> ThreatReport:
        """Generate a signed threat report."""
        # Hash the threat data to create a unique ID
        data_str = str(threat_data)
        threat_hash = hashlib.sha256(data_str.encode()).hexdigest()
        
        report = ThreatReport(
            reporter_id=self.agent_id,
            threat_hash=threat_hash,
            threat_type=threat_type,
            severity=severity,
            timestamp=time.time()
        )
        report.sign(self.secret)
        self.known_reports[threat_hash] = report
        return report
        
    def receive_gossip(self, report_data: Dict[str, Any]) -> Optional[ThreatReport]:
        """Process incoming gossip."""
        try:
            report = ThreatReport.from_dict(report_data)
            
            # 1. Verify Signature
            if not report.verify(self.secret):
                return None
                
            # 2. Check deduplication
            if report.threat_hash in self.known_reports:
                return None # Already known
                
            # 3. Store valid report
            self.known_reports[report.threat_hash] = report
            return report
            
        except Exception:
            return None
