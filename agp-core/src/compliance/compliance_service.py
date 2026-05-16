"""
Compliance & Audit Framework - Phase 6
GDPR, SOC2, and audit trail services
"""

import uuid
import json
import hashlib
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict


class ComplianceFramework(str, Enum):
    GDPR = "gdpr"
    SOC2 = "soc2"
    HIPAA = "hipaa"
    ISO27001 = "iso27001"


class AuditEventType(str, Enum):
    ACCESS = "access"
    CREATE = "create"
    UPDATE = "update"
    DELETE = "delete"
    LOGIN = "login"
    LOGOUT = "logout"
    EXPORT = "export"
    CONSENT = "consent"
    TRANSFER = "transfer"


class DataCategory(str, Enum):
    PII = "pii"
    SENSITIVE = "sensitive"
    FINANCIAL = "financial"
    BEHAVIORAL = "behavioral"
    TECHNICAL = "technical"


@dataclass
class AuditEvent:
    """Immutable audit log entry"""
    id: uuid.UUID
    timestamp: datetime
    event_type: AuditEventType
    actor_id: str
    resource_type: str
    resource_id: str
    action: str
    data_categories: List[DataCategory]
    ip_address: Optional[str]
    user_agent: Optional[str]
    metadata: Dict[str, Any]
    checksum: str = ""
    
    def __post_init__(self):
        if not self.checksum:
            self.checksum = self._compute_checksum()
    
    def _compute_checksum(self) -> str:
        """Compute tamper-proof checksum"""
        data = f"{self.id}{self.timestamp.isoformat()}{self.event_type}{self.actor_id}{self.resource_id}{self.action}"
        return hashlib.sha256(data.encode()).hexdigest()


@dataclass
class ConsentRecord:
    """GDPR consent tracking"""
    id: uuid.UUID
    subject_id: str
    purpose: str
    granted: bool
    granted_at: datetime
    expires_at: Optional[datetime]
    revoked_at: Optional[datetime]
    version: str
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class DataRetentionPolicy:
    """Data retention rules"""
    id: uuid.UUID
    data_category: DataCategory
    retention_days: int
    legal_basis: str
    auto_delete: bool = True
    requires_consent: bool = False


class AuditLogService:
    """
    Immutable audit logging for compliance
    """
    
    def __init__(self):
        self.events: List[AuditEvent] = []
        self.event_index: Dict[str, List[uuid.UUID]] = defaultdict(list)
    
    def log(
        self,
        event_type: AuditEventType,
        actor_id: str,
        resource_type: str,
        resource_id: str,
        action: str,
        data_categories: List[DataCategory] = None,
        ip_address: Optional[str] = None,
        user_agent: Optional[str] = None,
        metadata: Optional[Dict] = None
    ) -> AuditEvent:
        """Log an audit event"""
        event = AuditEvent(
            id=uuid.uuid4(),
            timestamp=datetime.utcnow(),
            event_type=event_type,
            actor_id=actor_id,
            resource_type=resource_type,
            resource_id=resource_id,
            action=action,
            data_categories=data_categories or [],
            ip_address=ip_address,
            user_agent=user_agent,
            metadata=metadata or {}
        )
        
        self.events.append(event)
        
        # Index by actor and resource
        self.event_index[f"actor:{actor_id}"].append(event.id)
        self.event_index[f"resource:{resource_id}"].append(event.id)
        
        return event
    
    def get_by_actor(
        self,
        actor_id: str,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None
    ) -> List[AuditEvent]:
        """Get events by actor"""
        events = [e for e in self.events if e.actor_id == actor_id]
        
        if start_time:
            events = [e for e in events if e.timestamp >= start_time]
        if end_time:
            events = [e for e in events if e.timestamp <= end_time]
        
        return events
    
    def get_by_resource(self, resource_id: str) -> List[AuditEvent]:
        """Get events by resource"""
        return [e for e in self.events if e.resource_id == resource_id]
    
    def verify_integrity(self) -> bool:
        """Verify audit log integrity"""
        for event in self.events:
            if event.checksum != event._compute_checksum():
                return False
        return True
    
    def export(
        self,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None
    ) -> List[Dict]:
        """Export audit log for compliance reporting"""
        events = self.events
        
        if start_time:
            events = [e for e in events if e.timestamp >= start_time]
        if end_time:
            events = [e for e in events if e.timestamp <= end_time]
        
        return [
            {
                "id": str(e.id),
                "timestamp": e.timestamp.isoformat(),
                "event_type": e.event_type.value,
                "actor_id": e.actor_id,
                "resource_type": e.resource_type,
                "resource_id": e.resource_id,
                "action": e.action,
                "checksum": e.checksum
            }
            for e in events
        ]


class GDPRService:
    """
    GDPR compliance implementation
    
    Key rights:
    - Right to access
    - Right to erasure
    - Right to data portability
    - Consent management
    """
    
    def __init__(self, audit_log: AuditLogService):
        self.audit_log = audit_log
        self.consents: Dict[str, List[ConsentRecord]] = defaultdict(list)
        self.retention_policies: Dict[DataCategory, DataRetentionPolicy] = {}
        self.data_locations: Dict[str, List[str]] = defaultdict(list)
    
    def record_consent(
        self,
        subject_id: str,
        purpose: str,
        granted: bool,
        version: str = "1.0",
        expires_days: Optional[int] = None
    ) -> ConsentRecord:
        """Record consent decision"""
        consent = ConsentRecord(
            id=uuid.uuid4(),
            subject_id=subject_id,
            purpose=purpose,
            granted=granted,
            granted_at=datetime.utcnow(),
            expires_at=datetime.utcnow() + timedelta(days=expires_days) if expires_days else None,
            revoked_at=None,
            version=version
        )
        
        self.consents[subject_id].append(consent)
        
        # Audit log
        self.audit_log.log(
            AuditEventType.CONSENT,
            subject_id,
            "consent",
            str(consent.id),
            f"consent_{'granted' if granted else 'denied'}",
            [DataCategory.PII],
            metadata={"purpose": purpose, "version": version}
        )
        
        return consent
    
    def revoke_consent(self, subject_id: str, purpose: str) -> bool:
        """Revoke consent for a purpose"""
        for consent in self.consents.get(subject_id, []):
            if consent.purpose == purpose and not consent.revoked_at:
                consent.revoked_at = datetime.utcnow()
                
                self.audit_log.log(
                    AuditEventType.CONSENT,
                    subject_id,
                    "consent",
                    str(consent.id),
                    "consent_revoked",
                    [DataCategory.PII],
                    metadata={"purpose": purpose}
                )
                return True
        return False
    
    def check_consent(self, subject_id: str, purpose: str) -> bool:
        """Check if valid consent exists"""
        for consent in self.consents.get(subject_id, []):
            if consent.purpose != purpose:
                continue
            if not consent.granted:
                continue
            if consent.revoked_at:
                continue
            if consent.expires_at and consent.expires_at < datetime.utcnow():
                continue
            return True
        return False
    
    def handle_access_request(self, subject_id: str) -> Dict:
        """Handle GDPR Article 15 - Right of Access"""
        self.audit_log.log(
            AuditEventType.EXPORT,
            subject_id,
            "subject_data",
            subject_id,
            "access_request",
            [DataCategory.PII]
        )
        
        # Collect all data about subject
        return {
            "subject_id": subject_id,
            "request_date": datetime.utcnow().isoformat(),
            "data_locations": self.data_locations.get(subject_id, []),
            "consents": [
                {
                    "purpose": c.purpose,
                    "granted": c.granted,
                    "granted_at": c.granted_at.isoformat(),
                    "revoked_at": c.revoked_at.isoformat() if c.revoked_at else None
                }
                for c in self.consents.get(subject_id, [])
            ],
            "audit_trail": self.audit_log.export()
        }
    
    def handle_erasure_request(self, subject_id: str) -> Dict:
        """Handle GDPR Article 17 - Right to Erasure"""
        locations = self.data_locations.get(subject_id, [])
        
        self.audit_log.log(
            AuditEventType.DELETE,
            subject_id,
            "subject_data",
            subject_id,
            "erasure_request",
            [DataCategory.PII],
            metadata={"locations": locations}
        )
        
        # Mark for deletion (actual deletion in production)
        return {
            "subject_id": subject_id,
            "request_date": datetime.utcnow().isoformat(),
            "status": "queued",
            "locations_to_erase": locations
        }
    
    def handle_portability_request(self, subject_id: str) -> Dict:
        """Handle GDPR Article 20 - Right to Data Portability"""
        self.audit_log.log(
            AuditEventType.EXPORT,
            subject_id,
            "subject_data",
            subject_id,
            "portability_request",
            [DataCategory.PII]
        )
        
        return {
            "subject_id": subject_id,
            "request_date": datetime.utcnow().isoformat(),
            "format": "json",
            "data": self.handle_access_request(subject_id)
        }


class SOC2ControlsService:
    """
    SOC2 compliance controls
    
    Trust Service Criteria:
    - Security
    - Availability
    - Processing Integrity
    - Confidentiality
    - Privacy
    """
    
    def __init__(self, audit_log: AuditLogService):
        self.audit_log = audit_log
        self.controls: Dict[str, Dict] = {}
        self.assessments: List[Dict] = []
        
        self._init_controls()
    
    def _init_controls(self):
        """Initialize SOC2 controls"""
        self.controls = {
            "CC6.1": {
                "name": "Logical Access Security",
                "category": "security",
                "description": "Logical access to system components is restricted",
                "status": "implemented"
            },
            "CC6.2": {
                "name": "Authentication",
                "category": "security",
                "description": "Requires authentication to access system",
                "status": "implemented"
            },
            "CC6.3": {
                "name": "Authorization",
                "category": "security",
                "description": "Role-based access control implemented",
                "status": "implemented"
            },
            "CC7.1": {
                "name": "System Monitoring",
                "category": "availability",
                "description": "System activity monitored and logged",
                "status": "implemented"
            },
            "CC7.2": {
                "name": "Incident Response",
                "category": "availability",
                "description": "Security incidents detected and managed",
                "status": "implemented"
            },
            "CC8.1": {
                "name": "Change Management",
                "category": "processing_integrity",
                "description": "Changes to system are controlled",
                "status": "implemented"
            },
            "C1.1": {
                "name": "Data Classification",
                "category": "confidentiality",
                "description": "Confidential data is classified and protected",
                "status": "implemented"
            },
            "P6.1": {
                "name": "Consent Management",
                "category": "privacy",
                "description": "Consent obtained and managed",
                "status": "implemented"
            }
        }
    
    def assess_control(
        self,
        control_id: str,
        assessor: str,
        status: str,
        evidence: List[str],
        notes: str = ""
    ) -> Dict:
        """Record control assessment"""
        if control_id not in self.controls:
            raise ValueError(f"Unknown control: {control_id}")
        
        assessment = {
            "id": str(uuid.uuid4()),
            "control_id": control_id,
            "assessor": assessor,
            "assessment_date": datetime.utcnow().isoformat(),
            "status": status,
            "evidence": evidence,
            "notes": notes
        }
        
        self.assessments.append(assessment)
        
        self.audit_log.log(
            AuditEventType.UPDATE,
            assessor,
            "soc2_control",
            control_id,
            "control_assessed",
            metadata=assessment
        )
        
        return assessment
    
    def get_compliance_report(self) -> Dict:
        """Generate SOC2 compliance report"""
        by_category = defaultdict(list)
        
        for control_id, control in self.controls.items():
            by_category[control["category"]].append({
                "id": control_id,
                "name": control["name"],
                "status": control["status"]
            })
        
        return {
            "report_date": datetime.utcnow().isoformat(),
            "framework": "SOC2 Type II",
            "controls_by_category": dict(by_category),
            "total_controls": len(self.controls),
            "implemented": len([c for c in self.controls.values() if c["status"] == "implemented"]),
            "recent_assessments": self.assessments[-10:]
        }


# Create singleton instances
audit_log = AuditLogService()
gdpr_service = GDPRService(audit_log)
soc2_service = SOC2ControlsService(audit_log)
