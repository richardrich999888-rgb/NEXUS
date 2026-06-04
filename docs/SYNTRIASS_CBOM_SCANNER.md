# Syntriass CBOM Scanner Pitch Note

## Problem

Defence and national infrastructure operators cannot migrate to post-quantum
cryptography if they do not know where RSA, ECDSA, Ed25519, ECDHE, JWT signing
profiles, database backups, and legacy crypto libraries are embedded.

The near-term operational issue is not only a future cryptographically relevant
quantum computer. The immediate issue is migration blindness: long-life data,
mission records, device identities, tactical messaging, and software update
chains may already depend on crypto that must be inventoried and prioritized.

## MVP

`tools/cbom_scanner/cbom_scan.py` is a local, static cryptographic bill of
materials scanner. It produces JSON and Markdown reports suitable for review.

The scanner detects:

- PEM and SSH RSA/ECDSA/Ed25519 key indicators.
- JWT `RS*`, `ES*`, and `EdDSA` algorithm references.
- TLS ECDHE, X25519, NIST curve, and RSA references.
- Classical crypto library dependencies.
- PQC library references that need production-flow validation.
- Database, dump, and backup artifacts that may represent HNDL exposure.

## Demo Command

```bash
python3 tools/cbom_scanner/cbom_scan.py . \
  --json reports/cbom.json \
  --markdown reports/cbom.md
```

## Defence Wedge

This is the lead magnet for a larger Syntriass Vault & Comm proposal:

- First show the customer where classical crypto and long-life data risks exist.
- Then offer PQC migration wrappers for identity, messaging, audit records, and
  database envelope encryption.
- Keep the claim conservative: current scanner is discovery and prioritization;
  production enforcement requires integration work.

## Hard Boundary

This tool does not claim compromise detection, classified network accreditation,
or full PQC migration. It is the discovery layer that makes the migration
problem visible.
