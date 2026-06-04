#!/usr/bin/env python3
"""
AURA Trust offline verification evidence harness.

This is a proposal evidence script, not an accredited secure information
platform. It demonstrates the packet-level behavior promised for the iDEX
prototype: signed mission packet acceptance, tamper rejection, replay rejection,
freshness rejection, and ETK-compatible audit record emission.
"""

from __future__ import annotations

import base64
import hashlib
import json
import time
from dataclasses import dataclass, field
from typing import Any, Dict, List, Tuple

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Dict[str, Any]) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


@dataclass
class MissionPacket:
    source_id: str
    payload: Dict[str, Any]
    timestamp_utc: int
    nonce: str
    sequence_number: int
    provenance: List[str]
    policy_class: str
    signature_b64: str = ""

    def unsigned_dict(self) -> Dict[str, Any]:
        return {
            "source_id": self.source_id,
            "payload_hash": sha256_hex(canonical_json(self.payload)),
            "timestamp_utc": self.timestamp_utc,
            "nonce": self.nonce,
            "sequence_number": self.sequence_number,
            "provenance": self.provenance,
            "policy_class": self.policy_class,
        }

    def signing_bytes(self) -> bytes:
        return canonical_json(self.unsigned_dict())

    def sign(self, private_key: Ed25519PrivateKey) -> "MissionPacket":
        self.signature_b64 = base64.b64encode(private_key.sign(self.signing_bytes())).decode()
        return self


@dataclass
class OfflineTrustStore:
    public_keys: Dict[str, Ed25519PublicKey]
    seen_nonces: set[Tuple[str, str]] = field(default_factory=set)
    highest_sequence: Dict[str, int] = field(default_factory=dict)


class AURATrustVerifier:
    def __init__(self, trust_store: OfflineTrustStore, freshness_seconds: int = 300):
        self.trust_store = trust_store
        self.freshness_seconds = freshness_seconds
        self.audit_records: List[Dict[str, Any]] = []

    def verify(self, packet: MissionPacket, now_utc: int) -> Tuple[bool, str, Dict[str, Any]]:
        reason = "ACCEPTED"

        if packet.source_id not in self.trust_store.public_keys:
            reason = "UNKNOWN_SOURCE"
            return self._record(packet, False, reason)

        if abs(now_utc - packet.timestamp_utc) > self.freshness_seconds:
            reason = "STALE_PACKET"
            return self._record(packet, False, reason)

        nonce_key = (packet.source_id, packet.nonce)
        if nonce_key in self.trust_store.seen_nonces:
            reason = "REPLAYED_NONCE"
            return self._record(packet, False, reason)

        highest = self.trust_store.highest_sequence.get(packet.source_id, -1)
        if packet.sequence_number <= highest:
            reason = "REPLAYED_SEQUENCE"
            return self._record(packet, False, reason)

        try:
            signature = base64.b64decode(packet.signature_b64)
            self.trust_store.public_keys[packet.source_id].verify(signature, packet.signing_bytes())
        except (InvalidSignature, ValueError):
            reason = "SIGNATURE_INVALID_OR_TAMPERED"
            return self._record(packet, False, reason)

        self.trust_store.seen_nonces.add(nonce_key)
        self.trust_store.highest_sequence[packet.source_id] = packet.sequence_number
        return self._record(packet, True, reason)

    def _record(self, packet: MissionPacket, accepted: bool, reason: str) -> Tuple[bool, str, Dict[str, Any]]:
        unsigned = packet.unsigned_dict()
        record = {
            "schema": "AURA_TRUST_AUDIT_V0",
            "source_id": packet.source_id,
            "packet_hash": sha256_hex(packet.signing_bytes()),
            "payload_hash": unsigned["payload_hash"],
            "provenance_root": sha256_hex(canonical_json({"provenance": packet.provenance})),
            "policy_ref": sha256_hex(packet.policy_class.encode()),
            "result": "ACCEPT" if accepted else "REJECT",
            "reason": reason,
        }
        record["audit_record_hash"] = sha256_hex(canonical_json(record))
        self.audit_records.append(record)
        return accepted, reason, record


def build_packet(source_id: str, key: Ed25519PrivateKey, sequence: int, nonce: str, timestamp: int) -> MissionPacket:
    return MissionPacket(
        source_id=source_id,
        payload={
            "mission_id": "OC19-AURA-DEMO",
            "grid": "redacted-grid-square",
            "classification": "simulation-only",
            "sensor_report": {"track_count": 3, "confidence": 0.82},
        },
        timestamp_utc=timestamp,
        nonce=nonce,
        sequence_number=sequence,
        provenance=["sensor-node-alpha", "edge-filter-v1", "mission-packetizer"],
        policy_class="DEFENCE_SIMULATION_PACKET_V0",
    ).sign(key)


def assert_test(name: str, condition: bool, details: str = "") -> None:
    status = "PASS" if condition else "FAIL"
    print(f"{status}: {name}{' - ' + details if details else ''}")
    if not condition:
        raise AssertionError(name)


def main() -> None:
    now = int(time.time())
    private_key = Ed25519PrivateKey.generate()
    public_key = private_key.public_key()
    public_key_bytes = public_key.public_bytes(Encoding.Raw, PublicFormat.Raw)
    source_id = "mission-source-alpha"

    verifier = AURATrustVerifier(
        OfflineTrustStore(public_keys={source_id: Ed25519PublicKey.from_public_bytes(public_key_bytes)})
    )

    valid_packet = build_packet(source_id, private_key, 1, "nonce-001", now)
    accepted, reason, valid_audit = verifier.verify(valid_packet, now)
    assert_test("valid signed mission packet accepted", accepted and reason == "ACCEPTED")
    assert_test("audit record hash generated", len(valid_audit["audit_record_hash"]) == 64)

    tampered_packet = build_packet(source_id, private_key, 2, "nonce-002", now)
    tampered_packet.payload["sensor_report"]["track_count"] = 99
    accepted, reason, _ = verifier.verify(tampered_packet, now)
    assert_test("tampered payload rejected", not accepted and reason == "SIGNATURE_INVALID_OR_TAMPERED")

    replay_packet = build_packet(source_id, private_key, 3, "nonce-003", now)
    accepted, reason, _ = verifier.verify(replay_packet, now)
    assert_test("first packet with fresh nonce accepted", accepted and reason == "ACCEPTED")
    accepted, reason, _ = verifier.verify(replay_packet, now)
    assert_test("replayed nonce rejected", not accepted and reason == "REPLAYED_NONCE")

    stale_packet = build_packet(source_id, private_key, 4, "nonce-004", now - 1000)
    accepted, reason, _ = verifier.verify(stale_packet, now)
    assert_test("stale packet rejected", not accepted and reason == "STALE_PACKET")

    old_sequence_packet = build_packet(source_id, private_key, 2, "nonce-005", now)
    accepted, reason, _ = verifier.verify(old_sequence_packet, now)
    assert_test("replayed sequence rejected", not accepted and reason == "REPLAYED_SEQUENCE")

    unknown_packet = build_packet("unknown-source", private_key, 1, "nonce-006", now)
    accepted, reason, _ = verifier.verify(unknown_packet, now)
    assert_test("unknown source rejected", not accepted and reason == "UNKNOWN_SOURCE")

    summary = {
        "accepted_records": sum(1 for r in verifier.audit_records if r["result"] == "ACCEPT"),
        "rejected_records": sum(1 for r in verifier.audit_records if r["result"] == "REJECT"),
        "audit_records": len(verifier.audit_records),
        "last_audit_hash": verifier.audit_records[-1]["audit_record_hash"],
    }
    print("SUMMARY:", json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
