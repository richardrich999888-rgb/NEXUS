# INVENTION DISCLOSURE FORM — Patent #4

## Multi-ASI Identity and Reputation-Based Execution Denial

---

**Applicant:** SYNTRIASS Labs Private Limited  
**Inventor:** Katta Naga Sri Ganesh  
**Docket No.:** [To Be Assigned]  
**Related:** Depends on Patent #1 (Execution Law)  

---

## TECHNICAL FIELD

The present disclosure relates to distributed multi-agent systems and artificial superintelligence (ASI) networks, and more particularly to execution denial based on cryptographic identity, defection detection, and transitive reputation aggregation.

---

## BACKGROUND

**1. The Problem**

In open swarms of autonomous agents, Sybil attacks (fake nodes), defecting nodes (Byzantine behavior), and nodes with poor reputation may attempt execution. Centralized PKI or certificate authorities are single points of failure. Prior art does not combine: (a) cryptographic agent identity, (b) defection tracking with cumulative severity and isolation threshold, and (c) transitive reputation aggregation (weighting observers by querier trust) into a single execution-denial gate.

**2. Limitations of Prior Art**

- **PKI:** Identity verification only; no defection or reputation.
- **BFT:** Tolerates failures; does not model reputation decay or transitive trust.
- **Reputation systems:** Often centralized or pairwise; do not use transitive weighting across observers.
- **Blocklists:** Static; do not accumulate defection severity or support threshold-based isolation.

**3. Objects of the Invention**

It is an object to provide a defection tracker that records defection type and severity, accumulates severity per node, and isolates nodes exceeding a threshold.

It is a further object to provide transitive reputation aggregation wherein a querier's reputation of a target is computed from observers' opinions weighted by the querier's trust in each observer.

It is a further object to integrate identity, defection, and reputation into a single `allow_execution_by` check that denies execution for isolated or low-reputation principals.

---

## SUMMARY OF THE INVENTION

In one aspect, the invention provides a system comprising: a defection tracker that records defection events with type and severity, accumulates severity per node, and returns whether a node should be isolated based on a configurable threshold; a reputation aggregator that computes transitive reputation using observer-weighted formula R_j(A_i) = Σ_k (r_kj * r_ki) / Σ_k (r_kj); and an allow_execution_by function that denies execution if the principal is isolated or if aggregated reputation is below a minimum threshold.

In another aspect, the invention provides a method comprising: upon an execution request from a principal, checking whether the principal is isolated due to defection; if isolated, returning deny; otherwise, computing aggregated reputation of the principal from the perspective of the node; if reputation is below threshold, returning deny; otherwise, returning allow.

---

## DETAILED DESCRIPTION

### 1. AsiId and Identity

Cryptographic 32-byte identity for each ASI node. Used as key for defection records and reputation.

**Code:** `multi-asi-immune/src/identity/keypair.rs` — `AsiId` (lines 14–159).

### 2. Defection Tracker

Defection types: Unresponsive, Contradictory, ConstraintViolation, InvalidSignatures, FalseThreatReports, IdentityForgery. Each has a severity [0,1]. Records accumulate per node. `cumulative_severity(node)` sums severity of all records. `should_isolate(node)` returns true if cumulative severity ≥ isolation_threshold (default 1.5).

**Code:** `multi-asi-immune/src/enforcement/defection.rs` — `DefectionType`, `DefectionRecord`, `DefectionTracker` (lines 8–110).

### 3. Reputation Aggregator

Direct observations: observer → target → ReputationScore. ReputationScore has time-decay. Aggregated reputation: R_j(A_i) = Σ_k (r_kj * r_ki) / Σ_k (r_kj), weighting each observer's opinion by querier's trust in that observer.

**Code:** `multi-asi-immune/src/reputation/aggregation.rs` — `ReputationAggregator`, `get_aggregated()` (lines 12–100). `multi-asi-immune/src/reputation/score.rs` — `ReputationScore`.

### 4. allow_execution_by

Checks: (1) If principal is isolated (defection.should_isolate), return Err. (2) Get aggregated reputation of principal from this node's perspective. (3) If reputation < min_reputation, return Err. (4) Otherwise Ok(()).

**Code:** `multi-asi-immune/src/node/state.rs` — `allow_execution_by()` (lines 255–273).

### 5. ImmuneGuard Integration

`ImmuneGuard` implements `ExecutionGuard`. It calls `allow_execution_by` (or equivalent) with the PCU identity. Deny flows to the mandatory execution gate (Patent #1).

**Code:** `nexus-executor/src/guards/immune.rs` — `ImmuneGuard`.

---

## CLAIMS (Draft)

**1.** A system for execution denial in a multi-agent network, comprising:

a defection tracker configured to record defection events comprising a node identifier, a defection type, and a severity, to accumulate severity per node, and to determine whether a node should be isolated when cumulative severity meets or exceeds a threshold;

a reputation aggregator configured to maintain direct observations from observers about targets and to compute aggregated reputation of a target from a querier's perspective using transitive weighting, wherein each observer's opinion is weighted by the querier's trust in that observer and by a confidence factor derived from observation count; and

an execution gate configured to be invoked at an execution boundary before any computation of a computation unit, and to deny execution for a principal when the principal is isolated or when the principal's aggregated reputation is below a minimum threshold.

**2.** The system of claim 1, wherein the defection type is one of Unresponsive, Contradictory, ConstraintViolation, InvalidSignatures, FalseThreatReports, and IdentityForgery, each with an associated severity value.

**3.** The system of claim 1, wherein the aggregated reputation is computed as R_j(A_i) = Σ_k (r_kj * r_ki) / Σ_k (r_kj), where r_kj is the querier's trust in observer k and r_ki is observer k's reputation of target i.

**4.** The system of claim 1, wherein reputation scores decay over time, requiring continuous proof of benevolence.

**5.** A method for denying execution based on identity and reputation, comprising:

upon an execution request from a principal, checking whether the principal is isolated due to defection;

if isolated, returning deny;

computing aggregated reputation of the principal from the perspective of the receiving node using transitive weighting;

if reputation is below a threshold, returning deny; and

otherwise, returning allow.

---

## NOVELTY, DEFENSIBILITY, NON-OBVIOUSNESS & PRIOR ART

### Novelty

**Novelty assessment:** Combination-based novelty. Prior art includes transitive trust (FIRE, sybilproof protocols), reputation decay (SecuredTrust), and defection detection (Byzantine tolerance). The invention's novelty lies in: (a) combining defection tracker with cumulative severity and isolation threshold in a single execution-denial gate; (b) transitive reputation formula R_j(A_i) = Σ_k (r_kj * r_ki) / Σ_k (r_kj) with confidence-weighted observer opinions; (c) `allow_execution_by` as a single check that denies for both isolation and low reputation; (d) integration with mandatory execution guard (Patent #1) such that denial is structural. No single reference combines defection severity accumulation, transitive reputation aggregation, and execution-level denial.

### Defensibility

**Design-around difficulty:** High. The DefectionTracker.should_isolate() + ReputationAggregator.get_aggregated() + allow_execution_by() chain is tightly coupled. A competitor would need equivalent defection types with severity, cumulative threshold, transitive aggregation with observer weighting, and execution gate integration. Claims specifying the formula, isolation threshold, and execution denial strengthen defensibility.

### Non-Obviousness

**Inventive step:** A skilled person would not obviously combine (a) defection types (Unresponsive, Contradictory, IdentityForgery, etc.) with severity weights and cumulative isolation, (b) transitive reputation with querier-weighted observer opinions and confidence, and (c) execution-level denial at the guard boundary. The FIRE model uses witness reputation but does not tie it to execution denial. Byzantine tolerance does not model defection severity accumulation or reputation decay. The combination is non-obvious.

### Prior Art (Closest References)

| Reference | Type | Jurisdiction | Description | Distinguishing Feature |
|-----------|------|--------------|-------------|------------------------|
| FIRE (Huynh et al.) | Paper | — | Integrated trust/reputation: interaction, role, witness, certified | No execution denial; no defection severity; no isolation threshold |
| Sybilproof transitive trust | Paper | — | Transitive trust protocols for risky interactions | No defection tracker; no cumulative severity |
| SecuredTrust | Paper | — | Trust decay, malicious agent detection | Trust decay; no transitive aggregation; no execution gate |
| Computational trust review (Sabater, Sierra) | Paper | — | Survey of trust/reputation in multi-agent systems | Survey; no integrated execution denial |
| BFT / Byzantine | — | — | Fault tolerance | Tolerates failures; no reputation or defection tracking |

### Jurisdiction-Specific Search Databases & Queries

| Jurisdiction | Database | URL | Suggested Search Queries |
|--------------|----------|-----|--------------------------|
| **United States** | USPTO Patent Public Search | https://www.uspto.gov/patents/search | "transitive reputation" AND "multi-agent"; "defection" AND "isolation" AND distributed; "reputation decay" AND execution |
| **European Union** | Espacenet | https://worldwide.espacenet.com/ | transitive trust AND reputation AND agent; defection detection AND isolation; Byzantine AND reputation |
| **India** | IPO Patent Search | https://ipindiaservices.gov.in/ | multi-agent reputation; distributed trust isolation; defection Byzantine |
| **International** | PATENTSCOPE (WIPO) | https://patentscope.wipo.int/search/en/advancedSearch.jsf | FTXT:(transitive reputation) AND FTXT:(multi-agent); FTXT:(defection) AND FTXT:(isolation) |

---

## REFERENCE IMPLEMENTATION

| Component | File | Lines |
|-----------|------|-------|
| AsiId | multi-asi-immune/src/identity/keypair.rs | 14–159 |
| DefectionTracker | multi-asi-immune/src/enforcement/defection.rs | 40–110 |
| DefectionRecord, DefectionType | multi-asi-immune/src/enforcement/defection.rs | 8–55 |
| ReputationAggregator | multi-asi-immune/src/reputation/aggregation.rs | 12–100 |
| allow_execution_by | multi-asi-immune/src/node/state.rs | 255–273 |
| ImmuneGuard | nexus-executor/src/guards/immune.rs | 13–50 |

---

*End of IDF — Patent #4*
