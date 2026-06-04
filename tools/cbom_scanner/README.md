# Syntriass CBOM Scanner

The Syntriass CBOM Scanner creates a local cryptographic bill of materials for
post-quantum migration planning. It inventories quantum-vulnerable classical
crypto indicators such as RSA, ECDSA, Ed25519, ECDHE, JWT signing algorithms,
PEM/SSH key material, TLS curve references, and database/backup artifacts that
may represent long-life confidentiality exposure.

This is a static inventory tool. It does not upload data, decrypt secrets,
connect to databases, or claim that a system is compromised.

## Quick Start

```bash
python3 tools/cbom_scanner/cbom_scan.py . \
  --json cbom-report.json \
  --markdown cbom-report.md
```

To make CI fail when high-risk findings are detected:

```bash
python3 tools/cbom_scanner/cbom_scan.py . --fail-on high
```

## Why This Matters

Defence and critical-infrastructure operators need to know where RSA/ECC-era
cryptography is embedded before migrating to NIST PQC standards such as ML-KEM,
ML-DSA, and SLH-DSA. The scanner supports the first migration step: discovery
and prioritization.

## Conservative Boundaries

- Findings are indicators, not proof of exploitable compromise.
- Certificate algorithm parsing is intentionally shallow in this stdlib MVP.
- Network traffic, HSMs, cloud KMS inventory, and live database metadata are not
  scanned yet.
- PQC migration readiness must be validated with integration tests and target
  environment constraints.
