"""
Compliance module
"""

from .compliance_service import (
    AuditLogService, GDPRService, SOC2ControlsService,
    AuditEvent, AuditEventType, ConsentRecord, DataCategory,
    ComplianceFramework,
    audit_log, gdpr_service, soc2_service
)

__all__ = [
    "AuditLogService",
    "GDPRService",
    "SOC2ControlsService",
    "AuditEvent",
    "AuditEventType",
    "ConsentRecord",
    "DataCategory",
    "ComplianceFramework",
    "audit_log",
    "gdpr_service",
    "soc2_service"
]
