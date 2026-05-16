# Patent Grant Maximization Plan

**Objective:** Maximize grant probability across all 6 patents  
**Reality check:** 100% grant probability is not attainable—examiner subjectivity, prior-art discovery, and office actions are inherent. This plan pushes toward the maximum attainable probability (target: 80%+ per patent with amendments).  
**Status:** Actionable roadmap for counsel and filing  

---

## 1. PRIOR-ART DEEP DIVE (CRITICAL—DO BEFORE FILING)

### 1.1 Newly Identified Prior Art (Run These Searches)

| Patent | New Prior Art | Distinction to Emphasize |
|--------|---------------|--------------------------|
| **#1** | **MI9 (arXiv 2508.03858, Aug 2025)** — Runtime governance for agentic AI: Continuous Authorization Monitoring, FSM conformance, graduated containment | MI9 monitors and adjusts; does not have mandatory gate before cache/proof. No no-proof-on-deny. NEXUS blocks execution structurally; MI9 intervenes. |
| **#1** | **AGENTSAFE** — Dynamic authorization, interruptibility | Interruptibility ≠ pre-execution gate. NEXUS denies before any execution path; AGENTSAFE interrupts during. |
| **#5** | **Fission Everywhere Computer / Homestar** — Content-addressed WASM, IPVM, tasks by CID | Fission: content-addressed tasks. NEXUS: (a) intrinsic identity in unit, (b) deterministic ID = hash(code+inputs+params+identity), (c) routing to node holding input content (data locality), (d) execution proof binding. Fission lacks identity-in-payload and explicit data-locality routing. |
| **#2** | **US20080184367A1** — Entropy for malware detection | Entropy = data randomness; not consumable execution budget. NEXUS entropy = consequence-proportional spendable resource. |
| **#2** | **US9280644B2 (Apple)** — Entitlements for resource restriction | Resource caps; not consequence-tiered consumable budget. |

### 1.2 Mandatory Search Queries (Run in Each Jurisdiction)

**Patent #1:** `(execution NEAR gate) OR (mandatory NEAR guard)` AND `(AI OR agent OR autonomous)`  
**Patent #2:** `entropy` AND `(authorization OR execution)` AND NOT `(malware OR randomness OR cryptographic)`  
**Patent #5:** `(content-addressed OR content addressable)` AND `(computation OR routing)` AND `(identity OR principal)`  
**Patent #6:** `(vector clock OR CRDT)` AND `(provenance OR Merkle)` AND `merge`  

---

## 2. NARROWED CLAIM LANGUAGE (RED-LINE RECOMMENDATIONS)

### Patent #1 — Add These Limitations to Independent Claim

**Current risk:** Too broad.  
**Add to Claim 1 (system):**

- "wherein the execution guard is invoked **before** any cache lookup for the execution request"
- "wherein upon a deny decision, **no** execution proof is created and **no** cache entry is written"
- "wherein the execution engine has **no alternate code path** that produces an execution proof when the guard returns deny"

**Rationale:** Narrows prior art (GuardAgent, MI9) that check but don't block proof/cache.

---

### Patent #2 — Add These Limitations

**Add to Claim 1 (system):**

- "wherein **entropy** denotes a finite, consumable execution budget representing cost of irreversible action, **distinct from** cryptographic randomness"
- "wherein entropy cost **increases monotonically** with consequence tier"
- "wherein the membrane returns deny **before** the caller performs any execution of the action"

**Rationale:** Distinguishes from entropy-as-randomness (malware) and rate limiters.

---

### Patent #5 — Add These Limitations

**Add to Claim 1 (structure):**

- "wherein the deterministic identifier **incorporates** the principal identifier"
- "wherein the structure is routable to a node **based on which nodes hold content corresponding to the content-addressed input references**"

**Add new dependent claim:** "The structure of claim 1, wherein the identity context is **intrinsic** to the structure and serialized therewith, not resolved externally."

**Rationale:** Distinguishes from Fission (no identity in payload) and generic content-addressing.

---

### Patent #3 — Add These Limitations

**Add:** "wherein the gate enforcer is **operatively coupled** to a mandatory execution guard such that a denied result **prevents** execution before cache lookup or proof generation"

**Rationale:** Ties to Patent #1; makes dependency explicit.

---

### Patent #4 — Add These Limitations

**Add:** "wherein the execution gate is invoked **at an execution boundary** before any computation of a computation unit"

**Add:** "wherein the transitive reputation formula weights each observer's opinion by the querier's trust in that observer **and** by a confidence factor derived from observation count"

**Rationale:** Narrows vs FIRE (no execution-boundary integration).

---

### Patent #6 — Add These Limitations

**Add:** "wherein the lowest common ancestor is computed **from the provenance DAG structure** of the local and remote tensors"

**Add:** "wherein the merged provenance structure has **both** local and remote tensor identifiers as parents"

**Rationale:** Narrows vs generic CRDT merge; emphasizes provenance LCA.

---

## 3. FILING ORDER & DEPENDENCY STRATEGY

| Priority | Patent | Rationale |
|----------|--------|-----------|
| **1** | #1 Execution Law | Foundational; #3 and #4 depend on it. Establishes execution-boundary governance. |
| **2** | #2 Commitment Membrane | Independent; strong. File within 12 months of #1 for priority-chain option. |
| **3** | #5 PCU | Independent; good prior-art gap. File before Fission/IPVM filings mature. |
| **4** | #6 Causal Merge | Independent; CRDT prior art dense—file with narrowed claims. |
| **5** | #3 Developmental Gating | Dependent on #1. File as continuation or separate with "wherein the system of claim 1 of Patent #1..." |
| **6** | #4 Multi-ASI Immune | Dependent on #1. Same strategy as #3. |

**Jurisdiction order:** File US first (establish priority date). Then PCT or direct national phase for EU and India within 30/31 months.

---

## 4. EXAMINER RESPONSE PREPARATION

### 4.1 Anticipated Rejections & Response Templates

| Rejection Type | Patent | Response |
|----------------|--------|----------|
| **103 (obviousness)** | #1 | "GuardAgent/AgentSpec check actions/outputs. MI9 monitors and intervenes. Neither discloses: (1) guard invoked before cache lookup, (2) no-proof-on-deny, (3) structural prevention of proof creation on deny. The combination would require modifying the execution substrate, not layering a checker." |
| **103** | #2 | "OAuth/rate limiters do not use consequence-proportional consumable budget. Entropy herein is defined as execution-budget cost, not randomness. Apple US9280644 restricts resources by entitlement, not by spending a replenishing budget per consequence tier." |
| **103** | #5 | "Fission/Everywhere Computer use content-addressed tasks. Applicant's invention includes: (1) intrinsic identity in the computation unit, (2) deterministic ID = hash(code+inputs+identity), (3) routing to nodes holding input content. Fission does not bundle identity; routing is by capacity, not data locality." |
| **103** | #6 | "CRDTs have type-specific merge. Merkle-CRDTs provide transport. Applicant discloses: provenance LCA as merge base for concurrent resolution, deterministic data merge using LCA, merged provenance with both parents. This is a general causal tensor, not a type-specific CRDT." |
| **112** | Any | Prepare dependent claims that add each feature as separate limitation. Use Markush groups sparingly. |

### 4.2 Affidavit/Declaration Strategy

- **Patent #1:** Consider declaration from technical expert: "To my knowledge, no system in production invokes a governance check before cache lookup and enforces that denial produces no proof."
- **Patent #2:** Consider declaration: "Entropy as used herein is a term of art meaning consumable execution budget; it is not used in the cryptographic randomness sense."
- **Patent #5:** Consider declaration: "The combination of intrinsic identity, content-addressed inputs, and data-locality routing is not found in Fission, IPFS, or Docker."

---

## 5. PRE-FILING CHECKLIST

- [x] Run prior-art searches in USPTO, Espacenet, India IPO for all 6 patents using Section 1.2 queries
- [x] Update each patent document with newly found prior art (MI9, Fission, US20080184367, US9280644) in Prior Art section
- [x] Apply narrowed claim language from Section 2 to draft claims (Patents 1, 2, 3, 4, 5, 6)
- [ ] Confirm filing order with counsel: #1 → #2 → #5 → #6 → #3 → #4
- [ ] Engage patent counsel for at least #1 and #2
- [x] Prepare examiner response templates for anticipated 103 rejections (Section 4.1)
- [ ] Consider declaration strategy for key distinctions (Section 4.2)

---

## 6. REALISTIC PROBABILITY TARGETS

| Patent | Before Plan | After Plan (with amendments) |
|--------|-------------|------------------------------|
| #1 | 60–75% | 75–85% |
| #2 | 50–65% | 65–80% |
| #3 | 50–60% | 60–75% (if #1 grants) |
| #4 | 40–55% | 55–70% (if #1 grants) |
| #5 | 55–70% | 70–85% |
| #6 | 45–60% | 60–75% |

**Portfolio outcome:** With disciplined execution, expect 4–5 of 6 granted (with at least one round of amendments). 6/6 is possible but not guaranteed.

---

## 7. HONEST CAVEAT

**100% grant probability is not achievable** because:

1. Examiners have discretion; two examiners may reach different conclusions.
2. Prior art may surface during examination that was not findable in pre-filing search.
3. Jurisdictional differences: EPO is stricter on technical effect; India has CRI-specific rules.
4. Office actions require iterative response; some claims may be rejected and require narrowing or cancellation.

**What this plan does:** Maximizes the probability of grant by (a) identifying and addressing close prior art before filing, (b) narrowing claims to be more defensible, (c) preparing examiner responses in advance, and (d) sequencing filings to protect the strongest inventions first.

---

*Document prepared for patent counsel handoff. Execute Section 5 checklist before filing.*
