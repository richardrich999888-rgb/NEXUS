# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of NEXUS seriously. If you believe you have found a security vulnerability, please report it to us by following these steps:

1.  **Do not disclose the vulnerability publicly** until we have had a chance to address it.
2.  Send an email to **security@syntriass.com** with a detailed description of the vulnerability, including:
    *   Steps to reproduce.
    *   Potential impact.
    *   Any suggested mitigations.

We will acknowledge receipt of your report within 48 hours and provide a timeline for resolution.

## Security Model: Zero-Trust Substrate

NEXUS is built on a **Zero-Trust** architecture where security is intrinsic to the computation unit (PCU) and state object (USO).

1.  **Identity-Centric**: Every node and user is identified by a cryptographic `PrincipalId` (Ed25519).
2.  **Causal Integrity**: State transitions are signed and linked in a Causal DAG, preventing unauthorized state manipulation.
3.  **Content-Addressing**: Data integrity is verified via SHA-256 content hashes.
4.  **Runtime Isolation**: PCU execution is sandboxed using WASM, ensuring code cannot escape its defined context.

## Cryptography Standards

- **Signature Scheme**: Ed25519 (using `ed25519-dalek`)
- **Hashing**: SHA-256
- **Encryption**: TLS 1.3 / QUIC (for network transit)
