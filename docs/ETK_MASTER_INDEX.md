# ETK — Master Index (Irreducible Core)

**Purpose:** Single entry point for Schema, Verifier, Genesis, Claims, Threat Model, Transition Matrix, and Control Surface.  
**Audience:** Examiners, regulators, counsel, internal.  
**Status:** Locked v1.0.

---

## 1. Definition

**Execution Truth Kernel (ETK):** A passive, append-only system that generates verifiable cryptographic proofs that a specific execution occurred under specific constraints.

Not: enforcement, scheduling, intelligence, policy engines. **Truth capture only.**

---

## 2. Foundation Artifacts

| Artifact | Description | Location |
|----------|-------------|----------|
| **Schema v1.0** | ExecutionEvent_v1, ExecutionProof_v1; canonical serialization; field lock | Spec: (user-provided lock). Code: `nexus-etk/src/schema.rs` |
| **Verifier v1.0** | 7-phase verification; binary VERDICT; offline; no trust in runtime | Spec: (user-provided). Code: `nexus-etk/src/verifier.rs` |
| **Genesis v1.0** | execution_id derivation; genesis event; immutability from birth | Spec: (user-provided). Code: `nexus-etk/src/genesis.rs` |
| **Claims Mapping** | P3 family + dependents; examiner-survivable claim language | (user-provided claims doc) |
| **Threat Model v1.0** | T1–T8 threat classes; trust boundaries; non-goals | (user-provided threat doc) |
| **Transition Matrix** | 2025–2040; patent layer activation; industries; control points | `docs/ETK_TECHNOLOGY_TRANSITION_PATENT_MATRIX.md` |
| **Control Surface Map** | Hyperscaler, government, finance, energy, autonomy; structural weaknesses | `docs/ETK_CONTROL_SURFACE_MAP.md` |

---

## 3. Code Reference

| Component | Path |
|-----------|------|
| Schema (events, proof, enums, canonical bytes) | `nexus-etk/src/schema.rs` |
| Genesis (execution_id derivation, genesis event) | `nexus-etk/src/genesis.rs` |
| Event chain (append, hash linkage, finalize) | `nexus-etk/src/chain.rs` |
| Verifier (7 phases, VERDICT, error codes E1–E6) | `nexus-etk/src/verifier.rs` |
| Offline verifier CLI | `nexus-etk/src/bin/etk_verifier.rs` |
| Library surface | `nexus-etk/src/lib.rs` |

---

## 4. Invariants (Non-Negotiable)

- Append-only; no mutation or deletion of events.
- Deterministic serialization → same event → same hash.
- Opaque by design; no semantic interpretation inside ETK.
- Forward-compatible; new versions extend, never modify.
- Minimal surface; if a field doesn’t affect truth, it’s forbidden.
- Genesis is sole origin of execution_id; no Genesis → INVALID.
- Verification is offline; no API calls to producer required.

---

## 5. What ETK Explicitly Does Not Know

- Model weights, prompts, training data  
- Optimization logic, scheduling decisions  
- Policy semantics (only policy_ref hash)  

ETK only knows: *this ran, here, under that reference, and this happened.*

---

## 6. 30-Day Checkpoint (Build Target)

> “Here is a binary. Run any job. Kill the machine. I can still prove what ran and where.”

Delivered: library + offline verifier CLI; no UI, no dashboards, no integrations.

---

## 7. Where You Are Now

You have:

- Schema lock  
- Genesis definition  
- Verifier spec  
- Claim mapping  
- Threat model  
- Transition matrix  
- Control surface map  

This is the **complete irreducible core**. Everything else is application, not foundation.
