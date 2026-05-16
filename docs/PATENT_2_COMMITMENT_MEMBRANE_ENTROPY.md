# PATENT APPLICATION

## Commitment Membrane for Consequence-Bounded Execution Using Entropy

---

**Applicant:** SYNTRIASS Labs Private Limited  
**Inventor:** Katta Naga Sri Ganesh  
**Docket No.:** [To Be Assigned]  
**Filing Date:** [To Be Assigned]  

---

## ABSTRACT

A system and method for consequence-bounded execution authorization. A commitment membrane module receives a decision comprising an action identifier, an agent identifier, and a consequence tier. An entropy meter maintains a finite entropy budget and consumes entropy in an amount determined by the consequence tier when the budget is sufficient. An authority registry verifies the agent holds a required scope for the action. For consequence tiers at or above a threshold, a trust accumulator verifies the agent's accumulated trust score meets a minimum. The membrane returns an allow or deny result before any execution of the action. Denial produces no execution and no side effects. As used herein, entropy denotes a finite, consumable execution budget representing irreversible action cost, and is distinct from cryptographic randomness.

---

## CROSS-REFERENCES TO RELATED APPLICATIONS

[To Be Inserted—Patent #1 if filed]

---

## TECHNICAL FIELD

The present disclosure relates to execution authorization in high-consequence autonomous systems, and more particularly to risk-based authorization that permits or denies execution before any side effects occur, based on a consumable execution budget, authority scope, and trust history.

---

## BACKGROUND

**1. Field of Endeavor**

Autonomous software agents and artificial intelligence systems increasingly perform actions that produce irreversible effects in physical or digital domains. Such actions include, without limitation: financial transactions, physical actuation, data destruction, and software deployment. Once executed, these actions have permanent consequences. Authorization decisions must therefore be made prior to execution, and denial must result in no side effects whatsoever.

**2. Limitations of the Prior Art**

Identity-based access control systems (such as OAuth, OpenID Connect, or IAM) authorize an entity based on identity. Such systems may not account for cumulative cost of repeated actions, consequence severity of each action, or history of prior compliance or misuse. An authenticated agent with valid credentials may exhaust system capacity or perform harmful sequences of actions because identity alone may not bound execution.

Permission or capability systems (such as RBAC or ABAC) authorize based on permitted actions. Such systems may not limit the rate or volume of high-consequence actions, distinguish trivial from irreversible actions, or require a spendable resource proportional to consequence. An agent with broad permissions may cause harm through volume or through actions that are individually permitted but collectively dangerous.

Existing authorization systems that operate at the API or network layer may not provide a unified gate that: (a) consumes a finite execution budget proportional to consequence, (b) combines authority and trust as joint conditions, and (c) ensures denial occurs prior to any execution, producing zero side effects.

**3. Objects of the Invention**

It is an object of the present invention to provide a commitment membrane that serves as a single enforcement point invoked before execution.

It is a further object to use entropy as a finite, consumable execution budget, wherein entropy cost increases with consequence severity.

It is a further object to require joint satisfaction of entropy budget, authority scope, and trust threshold (for high-consequence tiers) before allowing execution.

It is a further object to ensure that denial produces no execution and no side effects of the requested action.

---

## SUMMARY OF THE INVENTION

In one aspect, the invention provides a system for consequence-bounded execution authorization comprising: a commitment membrane module configured to receive a decision comprising an action identifier, an agent identifier, and a consequence tier; an entropy meter operatively coupled to the commitment membrane module and configured to maintain a finite entropy budget and to consume entropy in an amount determined by the consequence tier when the budget is sufficient; an authority registry operatively coupled to the commitment membrane module and configured to verify that the agent holds a required scope for the action; and logic configured to return an allow result or a deny result before any execution of the action, wherein a deny result produces no execution and no side effects of the action.

In another aspect, the invention provides a method for authorizing execution of an action comprising: receiving a decision comprising an agent identifier and a consequence tier; attempting to consume entropy from a finite budget in an amount based on the consequence tier; if consumption fails, returning a deny result and performing no execution; verifying the agent has a required authority scope; if verification fails, returning a deny result and performing no execution; for consequence tiers at or above a threshold, verifying the agent's trust score meets a minimum; if trust verification fails, returning a deny result and performing no execution; and if all checks pass, returning an allow result, whereby execution occurs only when the allow result is returned.

As used herein, "entropy" denotes a finite, consumable execution budget representing the cost of irreversible or high-consequence action. Entropy is not cryptographic randomness. Entropy is depleted when actions are authorized and may replenish over time at a configurable rate.

---

## BRIEF DESCRIPTION OF THE DRAWINGS

**FIG. 1** is a block diagram of a system for consequence-bounded execution authorization according to an embodiment.

**FIG. 2** is a flowchart illustrating a low-tier action authorization flow.

**FIG. 3** is a flowchart illustrating a high-tier action authorization flow including a trust threshold check.

**FIG. 4** is a flowchart illustrating an entropy exhaustion denial flow.

---

## DETAILED DESCRIPTION OF THE INVENTION

### 1. Overview

With reference to **FIG. 1**, a system **100** for consequence-bounded execution authorization includes a commitment membrane module **110**, an entropy meter **120**, an authority registry **130**, and a trust accumulator **140**. The commitment membrane module **110** serves as a single enforcement point invoked before any execution of an action. A caller **102** (e.g., an execution kernel, scheduler, or orchestrator) submits a decision **104** to the commitment membrane module **110** and receives a crossing result **106** indicating allow or deny. The caller **102** executes the action only when the crossing result **106** indicates allow.

### 2. Decision Structure

The decision **104** comprises, at minimum: a decision identifier, an action identifier, an agent identifier, and a consequence tier. The consequence tier indicates the severity of the action and determines the entropy cost. In one embodiment, consequence tiers include: TRIVIAL (e.g., read-only operations), LOW (e.g., reversible writes), MEDIUM (e.g., significant changes), HIGH (e.g., critical operations), and CRITICAL (e.g., irreversible, high-impact operations). Entropy cost increases monotonically with consequence tier. For example, in one implementation, TRIVIAL costs 1 unit, LOW costs 10 units, MEDIUM costs 100 units, HIGH costs 500 units, and CRITICAL costs 2000 units. In another implementation, cost is proportional to the square of the tier value.

### 3. Entropy Meter

The entropy meter **120** maintains a finite entropy budget. The budget may be initialized to a maximum value (e.g., 10000 units) and replenished over time at a configurable rate (e.g., 100 units per minute). The entropy meter **120** provides a spend operation that: (a) determines the entropy cost for the given consequence tier, (b) refills the budget based on elapsed time, (c) compares the available budget to the cost, and (d) if the budget is sufficient, decrements the budget by the cost and returns success; otherwise returns failure without decrementing the budget.

### 4. Authority Registry

The authority registry **130** stores agent authorities, each comprising an agent identifier and a set of scopes. Scopes may be literal (e.g., "execute:deploy") or wildcard (e.g., "execute:*" matching any action under the execute namespace). The authority registry **130** provides a verify operation that checks whether a given agent holds a required scope. The authority registry **130** further supports revocation, whereby an agent's authority may be invalidated such that subsequent verification fails.

### 5. Trust Accumulator

The trust accumulator **140** maintains a per-agent trust score based on crossing history. Successful crossings may increase trust slightly; denied crossings may decrease trust. For consequence tiers at or above a configurable threshold (e.g., HIGH), the commitment membrane module **110** requires the agent's trust score to meet a minimum (e.g., 0.6). If the trust score is below the minimum, the membrane returns a deny result.

### 6. Crossing Request Flow

The commitment membrane module **110** receives a crossing request via a request_crossing operation, which accepts the decision **104** and a required scope. The membrane evaluates conditions in order:

**(a) Entropy Check.** The membrane invokes the entropy meter's spend operation with the consequence tier. If spend returns failure, the membrane returns a deny result with reason ENTROPY_EXHAUSTED. No entropy is consumed. No execution occurs.

**(b) Authority Check.** If entropy consumption succeeds, the membrane invokes the authority registry's verify operation with the agent identifier and required scope. If verify returns false, the membrane returns a deny result with reason AUTHORITY_DENIED. No execution occurs.

**(c) Trust Check (for high-consequence tiers).** If the consequence tier is at or above the threshold (e.g., HIGH), the membrane retrieves the agent's trust score from the trust accumulator. If the trust score is below the minimum (e.g., 0.6), the membrane returns a deny result with reason TRUST_INSUFFICIENT. No execution occurs.

**(d) Allow.** If all checks pass, the membrane returns an allow result. The caller may then proceed to execute the action.

### 7. Crossing Result

The crossing result **106** comprises: an allowed flag (true or false), a decision identifier, an entropy_spent value, an authority_verified flag, and optionally a reason string. When allowed is false, the caller must not execute the action. Denial produces no execution and no side effects of the requested action. The membrane does not perform execution; it only authorizes or denies. Execution is the responsibility of the caller, which must condition execution on the allowed flag being true.

### 8. Reference Implementation

One reference implementation is provided in the NEXUS codebase: `agp-core/src/telos/membrane.py` (CommitmentMembrane, EntropyMeter, AuthorityRegistry, TrustAccumulator, request_crossing) and `telos-protocol/src/entropy.rs` (ConsequenceTier, multiplier). The foregoing description of the detailed description is illustrative and not limiting. One skilled in the art will appreciate that various modifications and variations may be made without departing from the spirit and scope of the invention.

---

## NOVELTY, DEFENSIBILITY, NON-OBVIOUSNESS & PRIOR ART

### Novelty

**Novelty assessment:** Individual + combination novelty. "Entropy" as finite consumable execution budget (distinct from cryptographic randomness) is individually novel in the authorization context. The combination of (a) entropy meter with tiered cost, (b) authority registry with scope verification, (c) trust accumulator for high-consequence tiers, and (d) single crossing gate returning allow/deny before any execution is novel. OAuth, RBAC, and rate limiters do not use entropy as consequence-proportional spendable budget. No single reference discloses entropy-as-execution-budget with authority and trust in a pre-execution membrane.

### Defensibility

**Design-around difficulty:** Very High. The entropy + authority + trust combination at a single crossing point is specific. Defining "entropy" narrowly (consumable execution budget, not randomness) strengthens claims. Competitors using rate limits or tokens would need to argue around the consequence-tier proportionality and joint satisfaction of three conditions.

### Non-Obviousness

**Inventive step:** Using entropy (as defined) for execution authorization is not obvious from OAuth, rate limiters, or capability systems. Combining consequence-proportional budget consumption with authority scope and trust threshold for high tiers is non-obvious. The replenishment-over-time model for irreversible-action cost is not taught by prior art.

### Prior Art (Closest References)

| Reference | Type | Jurisdiction | Description | Distinguishing Feature |
|-----------|------|--------------|-------------|------------------------|
| OAuth 2.0 / OIDC | Standard | — | Identity/scope-based API authorization | No entropy budget; no consequence tier |
| US20080184367A1 | Patent | USPTO | Entropy for malware detection | Entropy = data randomness; not consumable execution budget |
| US9280644B2 (Apple) | Patent | USPTO | Entitlements for resource restriction | Resource caps; not consequence-tiered consumable budget |
| RBAC / ABAC | Standard | — | Role/attribute-based access | No consumable budget; no trust accumulator |
| Rate limiters / Token bucket | — | — | Request rate limiting | Rate-based; not consequence-proportional |
| IAM / Policy engines | Product | — | Identity and access management | Identity/attributes; no entropy |
| Governable AI (GAI) | Paper | — | Cryptographic rule enforcement | REM; no entropy budget; different structure |

### Jurisdiction-Specific Search Databases & Queries

| Jurisdiction | Database | URL | Suggested Search Queries |
|--------------|----------|-----|--------------------------|
| **United States** | USPTO Patent Public Search | https://www.uspto.gov/patents/search | "entropy" AND "authorization" AND execution; "consumable budget" AND consequence; "consequence tier" AND authorization |
| **European Union** | Espacenet | https://worldwide.espacenet.com/ | entropy execution authorization; consequence-bounded execution; consumable budget action |
| **India** | IPO Patent Search | https://ipindiaservices.gov.in/ | entropy authorization execution; consequence tier membrane; execution budget |
| **International** | PATENTSCOPE (WIPO) | https://patentscope.wipo.int/search/en/advancedSearch.jsf | FTXT:(entropy) AND FTXT:(execution) AND FTXT:(authorization); FTXT:(consequence) AND FTXT:(budget) |

---

## CLAIMS

**1.** A system for consequence-bounded execution authorization, comprising:

a commitment membrane module configured to receive a decision comprising an action identifier, an agent identifier, and a consequence tier;

an entropy meter operatively coupled to the commitment membrane module, the entropy meter configured to maintain a finite entropy budget and to consume entropy in an amount determined by the consequence tier when the budget is sufficient, wherein entropy denotes a finite, consumable execution budget representing cost of irreversible action, distinct from cryptographic randomness, and wherein entropy cost increases monotonically with consequence tier;

an authority registry operatively coupled to the commitment membrane module, the authority registry configured to verify that the agent holds a required scope for the action; and

logic configured to return an allow result or a deny result before the caller performs any execution of the action, wherein a deny result produces no execution and no side effects of the action.

**2.** The system of claim 1, wherein the entropy meter is configured to replenish the entropy budget over time at a configurable rate.

**3.** The system of claim 1, wherein the consequence tier is one of a predefined set of tiers, and wherein entropy cost increases monotonically with consequence tier.

**4.** The system of claim 1, further comprising a trust accumulator operatively coupled to the commitment membrane module, the trust accumulator configured to maintain a per-agent trust score based on crossing history.

**5.** The system of claim 4, wherein for consequence tiers at or above a configured threshold, the logic is configured to require the agent's trust score to meet a minimum before returning the allow result.

**6.** The system of claim 5, wherein the trust accumulator is configured to increase the trust score upon an allowed crossing and to decrease the trust score upon a denied crossing.

**7.** The system of claim 1, wherein the authority registry is configured to support scope strings with wildcard matching.

**8.** The system of claim 1, wherein the authority registry is configured to support revocation of an agent's authority.

**9.** The system of claim 1, wherein the commitment membrane module is invoked by an execution kernel or scheduler before transitioning a process or task to a runnable state, and wherein a deny result prevents such transition.

**10.** A method for authorizing execution of an action, comprising:

receiving a decision comprising an agent identifier and a consequence tier;

attempting to consume entropy from a finite budget in an amount based on the consequence tier;

if consumption fails, returning a deny result and performing no execution;

verifying the agent has a required authority scope;

if verification fails, returning a deny result and performing no execution;

for consequence tiers at or above a threshold, verifying the agent's trust score meets a minimum;

if trust verification fails, returning a deny result and performing no execution; and

if all checks pass, returning an allow result, whereby execution occurs only when the allow result is returned.

**11.** The method of claim 10, wherein entropy denotes a finite, consumable execution budget representing cost of irreversible or high-consequence action, and is distinct from cryptographic randomness.

**12.** The method of claim 10, wherein the entropy budget is replenished over time at a configurable rate.

**13.** The method of claim 10, wherein the consequence tier is one of a predefined set of tiers, and wherein entropy cost increases monotonically with consequence tier.

**14.** The method of claim 10, wherein the trust score is updated based on allowed and denied crossing results.

**15.** The method of claim 10, wherein the required authority scope supports wildcard matching.

---

## FIGURE DESCRIPTIONS

### FIG. 1 — Block Diagram

**FIG. 1** shows caller **102**, decision **104**, crossing result **106**, commitment membrane module **110**, entropy meter **120**, authority registry **130**, and trust accumulator **140**. Arrows indicate data flow: decision **104** from caller **102** to commitment membrane module **110**; crossing result **106** from commitment membrane module **110** to caller **102**; entropy meter **120**, authority registry **130**, and trust accumulator **140** operatively coupled to commitment membrane module **110**.

### FIG. 2 — Low-Tier Action Flow

**FIG. 2** illustrates: (1) Caller constructs decision with tier LOW; (2) Caller invokes request_crossing; (3) Entropy meter spend(LOW)—if budget sufficient, decrement and proceed; (4) Authority registry verify—if scope OK, proceed; (5) Tier < HIGH, skip trust check; (6) Return allow result; (7) Caller executes action.

### FIG. 3 — High-Tier Action Flow with Trust Check

**FIG. 3** illustrates: (1) Caller constructs decision with tier HIGH; (2) Caller invokes request_crossing; (3) Entropy meter spend(HIGH)—if budget sufficient, decrement and proceed; (4) Authority registry verify—if scope OK, proceed; (5) Tier ≥ HIGH, retrieve trust score; (6) If trust ≥ 0.6, return allow result; if trust < 0.6, return deny result with reason TRUST_INSUFFICIENT; (7) If allow, caller executes action.

### FIG. 4 — Entropy Exhaustion Denial Flow

**FIG. 4** illustrates: (1) Caller constructs decision with tier CRITICAL; (2) Caller invokes request_crossing; (3) Entropy meter spend(CRITICAL)—budget insufficient; (4) Return deny result with reason ENTROPY_EXHAUSTED, entropy_spent=0; (5) Trust accumulator records denial; (6) Caller does not execute; no side effects.

---

## INVENTOR & ASSIGNEE

| Field | Value |
|-------|-------|
| Inventor | Katta Naga Sri Ganesh |
| Assignee | SYNTRIASS Labs Private Limited |
| Primary Reference Implementation | `agp-core/src/telos/membrane.py`, `telos-protocol/src/entropy.rs` |

---

*End of Patent Application*
