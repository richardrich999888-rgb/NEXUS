# INVENTION DISCLOSURE FORM — Patent #6

## Algebraic Causal Merge for Distributed State Synchronization

---

**Applicant:** SYNTRIASS Labs Private Limited  
**Inventor:** Katta Naga Sri Ganesh  
**Docket No.:** [To Be Assigned]  
**Related:** Independent  

---

## TECHNICAL FIELD

The present disclosure relates to distributed systems, conflict-free replicated data types (CRDTs), and causal consistency. More particularly, it relates to a three-way causal merge algorithm that produces a deterministic merged result for concurrent updates using vector clocks, provenance (e.g., Merkle DAG), and algebraic resolution.

---

## BACKGROUND

**1. The Problem**

Distributed systems with multiple nodes updating shared state face merge conflicts. Last-writer-wins (LWW) loses updates. Operational CRDTs require operation commutativity, which limits expressiveness. Three-way merge (local, remote, base) in version control is typically application-specific. A general-purpose causal merge that is idempotent, commutative, and deterministic for concurrent updates—with support for provenance tracking—is needed for distributed AI state, multi-node execution fabrics, and edge sync.

**2. Limitations of Prior Art**

- **LWW:** Arbitrary resolution; loses concurrent updates.
- **CRDTs:** Operation-based or state-based; merge logic is predefined per type.
- **Git-style merge:** Three-way but application-specific (text, trees); not a general causal algebra.
- **Vector clocks alone:** Order events but do not define merge semantics for concurrent state.

**3. Objects of the Invention**

It is an object to provide a causal merge that is idempotent (identical tensors merge to same) and that respects causal ordering (happens_before selects newer).

It is a further object to provide algebraic resolution for concurrent merges using vector clock merge, provenance LCA (lowest common ancestor), and deterministic data merge (e.g., hash-ordered LWW).

It is a further object to bind merged result to merged provenance and signature for auditability.

---

## SUMMARY OF THE INVENTION

In one aspect, the invention provides a method for merging two causal tensors (local, remote) at a node, comprising: if local and remote have the same id, returning local (idempotence); if local's vector clock happens_before remote's, returning remote (causal monotonicity); if remote's vector clock happens_before local's, returning local; otherwise (concurrent), merging vector clocks, computing LCA of provenance DAGs, merging data deterministically (e.g., by hash comparison), constructing merged provenance from both parents, and producing a signed merged tensor.

In another aspect, the invention provides a causal tensor structure comprising: a content-addressed id (CausalId); a vector clock mapping node ids to logical timestamps; a provenance structure (e.g., Merkle DAG) for ancestry; data; and a signature binding id, data, and provenance.

---

## DETAILED DESCRIPTION

### 1. CausalId

32-byte content-addressed identifier. `from_hash(data)`, `genesis()` for root.

**Code:** `nexus-core/src/causal.rs` — `CausalId` (lines 17–44).

### 2. VectorClock

BTreeMap<node_id, logical_time>. `tick(node_id)` increments. `merge(other)` takes element-wise max. `happens_before(other)` returns true iff self is strictly causally before other. `concurrent(other)` when neither happens_before the other.

**Code:** `nexus-core/src/causal.rs` — `VectorClock` (lines 56–133).

### 3. Provenance

Merkle-style DAG for ancestry. `lca(other)` returns lowest common ancestor of two provenance DAGs. Used to determine merge base for concurrent resolution.

**Code:** `nexus-core/src/causal.rs` — `Provenance` (lines 145–203).

### 4. CausalTensor

Fields: id (CausalId), data (Vec<u8>), clock (VectorClock), provenance (Provenance), signature. `merge(local, remote, node_id, clock, signing_key)` implements the three-way algorithm.

**Code:** `nexus-core/src/causal.rs` — `CausalTensor` (lines 250–422).

### 5. Merge Algorithm (Core)

1. **Idempotence:** local.id == remote.id ⇒ return local.
2. **Causal monotonicity:** local.clock happens_before remote.clock ⇒ return remote. remote.clock happens_before local.clock ⇒ return local.
3. **Concurrent merge:** Merge clocks (element-wise max). Compute lca = local.provenance.lca(remote.provenance). Merge data: `merge_data(local.data, remote.data, lca)` — default deterministic LWW by hash comparison. Create merged provenance with parents = [local.id, remote.id] (sorted, deduped). Construct new CausalTensor with merged data, merged clock, merged provenance; sign.

**Code:** `nexus-core/src/causal.rs` — `merge()` (lines 342–391), `merge_data()` (lines 393–409).

---

## CLAIMS (Draft)

**1.** A method for merging two causal tensors in a distributed system, comprising:

if the local and remote tensors have identical identifiers, returning the local tensor (idempotence);

if the local tensor's vector clock happens-before the remote tensor's vector clock, returning the remote tensor;

if the remote tensor's vector clock happens-before the local tensor's vector clock, returning the local tensor;

otherwise, when the tensors are concurrent: merging the vector clocks by taking element-wise maximum; computing a lowest common ancestor of the local and remote provenance structures from the provenance DAG structure of each; merging the data of the local and remote tensors deterministically using the lowest common ancestor; constructing a merged provenance structure having both the local tensor identifier and the remote tensor identifier as parents; and producing a signed merged causal tensor.

**2.** The method of claim 1, wherein the deterministic data merge uses a hash comparison to select one of the local or remote data when no application-specific merge logic is provided.

**3.** The method of claim 1, wherein the vector clock maps node identifiers to logical timestamps and wherein happens-before is defined as strict causal ordering.

**4.** The method of claim 1, wherein the provenance structure is a Merkle DAG and the lowest common ancestor is computed from the DAG structure.

**5.** A causal tensor structure for distributed state, comprising:

a content-addressed identifier;

a vector clock mapping node identifiers to logical timestamps;

a provenance structure representing ancestry of the tensor;

data; and

a signature binding the identifier, data, and provenance.

**6.** The structure of claim 5, wherein the provenance structure supports computation of a lowest common ancestor with another provenance structure for merge resolution.

---

## NOVELTY, DEFENSIBILITY, NON-OBVIOUSNESS & PRIOR ART

### Novelty

**Novelty assessment:** Combination-based novelty. Prior art includes vector clocks (Fidge, Mattern 1988), CRDTs (Shapiro et al. 2011), Merkle-CRDTs (Protocol Labs), dotted version vectors (Preguiça et al.), and Git-style three-way merge. The invention's novelty lies in: (a) causal tensor structure with CausalId + VectorClock + Provenance (Merkle DAG) + signature; (b) three-way merge algorithm: idempotence → causal monotonicity (happens_before) → concurrent merge with LCA of provenance, deterministic data merge, merged provenance from both parents; (c) merge_data() using hash-ordered LWW as default; (d) signed merged tensor for auditability. Merkle-CRDTs combine Merkle-DAG with CRDTs but do not disclose this specific merge flow with provenance LCA for concurrent resolution. No single reference combines vector-clock causal ordering, provenance LCA, and deterministic concurrent merge in this structure.

### Defensibility

**Design-around difficulty:** Moderate to High. The merge algorithm (idempotence → causal monotonicity → concurrent with LCA) is specific. A competitor could use different merge semantics (e.g., operation-based CRDT) but would need equivalent idempotent, commutative, deterministic behavior with provenance. Claims on "lowest common ancestor of provenance structures" and "merged provenance with both parents" narrow options. The CausalTensor struct (id, data, provenance, clock, signature) is a distinct apparatus claim.

### Non-Obviousness

**Inventive step:** A skilled person would not obviously combine (a) vector-clock causal ordering, (b) provenance DAG with LCA for merge base, and (c) deterministic data merge (hash-ordered LWW) for concurrent updates. CRDTs use merge functions but are type-specific. Merkle-CRDTs provide transport; they do not specify this merge algorithm. Git three-way merge is text/tree-specific. The general-purpose causal tensor with algebraic concurrent resolution is non-obvious.

### Prior Art (Closest References)

| Reference | Type | Jurisdiction | Description | Distinguishing Feature |
|-----------|------|--------------|-------------|------------------------|
| Shapiro et al. 2011 | Paper | — | Comprehensive CRDT study | State/op-based; type-specific merge; no provenance LCA |
| Merkle-CRDTs (Protocol Labs) | Paper | — | Merkle-DAG meets CRDTs | Transport layer; no disclosed merge with LCA |
| Dotted Version Vectors (Preguiça) | Paper | — | Causality for key-value stores | Version vectors; no provenance DAG; no three-way merge |
| Vector clocks (Fidge, Mattern) | Paper | — | Logical clocks for distributed events | Ordering only; no merge semantics |
| Git merge | Software | — | Three-way text/tree merge | Application-specific; not general causal tensor |
| Delta State CRDTs | Paper | — | Incremental CRDT sync | δ-CRDT; different merge approach |

### Jurisdiction-Specific Search Databases & Queries

| Jurisdiction | Database | URL | Suggested Search Queries |
|--------------|----------|-----|--------------------------|
| **United States** | USPTO Patent Public Search | https://www.uspto.gov/patents/search | "vector clock" AND "merge"; "CRDT" AND "causal"; "Merkle" AND "distributed" AND merge; "provenance" AND "concurrent" |
| **European Union** | Espacenet | https://worldwide.espacenet.com/ | causal merge distributed; vector clock CRDT; Merkle DAG conflict resolution |
| **India** | IPO Patent Search | https://ipindiaservices.gov.in/ | causal merge distributed; CRDT vector clock; conflict-free replication |
| **International** | PATENTSCOPE (WIPO) | https://patentscope.wipo.int/search/en/advancedSearch.jsf | FTXT:(vector clock) AND FTXT:(merge); FTXT:(CRDT) AND FTXT:(causal) |

---

## REFERENCE IMPLEMENTATION

| Component | File | Lines |
|-----------|------|-------|
| CausalId | nexus-core/src/causal.rs | 17–50 |
| VectorClock | nexus-core/src/causal.rs | 56–133 |
| Provenance | nexus-core/src/causal.rs | 145–203 |
| CausalTensor | nexus-core/src/causal.rs | 250–340 |
| merge() | nexus-core/src/causal.rs | 342–391 |
| merge_data() | nexus-core/src/causal.rs | 393–409 |

---

*End of IDF — Patent #6*
