# INVENTION DISCLOSURE FORM — Patent #5

## Portable Computation Unit with Content-Addressed Verifiable Execution

---

**Applicant:** SYNTRIASS Labs Private Limited  
**Inventor:** Katta Naga Sri Ganesh  
**Docket No.:** [To Be Assigned]  
**Related:** Independent; integrates with Patent #1  

---

## TECHNICAL FIELD

The present disclosure relates to distributed computation, edge computing, and verifiable execution. More particularly, it relates to a portable computation unit that carries code, content-addressed data references, and intrinsic identity, and that routes to where data lives rather than moving data to code.

---

## BACKGROUND

**1. The Problem**

Cloud computing separates code (containers), identity (IAM), and data (S3). This fragmentation enables supply chain attacks and data sovereignty issues. Data is typically moved to compute; bandwidth and latency cost scale with data size. There is no single artifact that binds code, inputs, identity, and execution proof into a deterministic, content-addressed unit with a cryptographically derived identifier.

**2. Limitations of Prior Art**

- **Containers (Docker):** Bundle code and env; identity is external (IAM); no content-addressed routing.
- **Serverless:** Code and data separate; cold starts; no intrinsic identity in the payload.
- **Data locality:** Some systems co-locate compute and data but do not use content-addressed inputs to route the computation.

**3. Objects of the Invention**

It is an object to provide a Portable Computation Unit (PCU) comprising code, content-addressed input references, parameters, and identity in a single structure.

It is a further object to compute a deterministic PCU identifier from code hash, input hashes, parameters, and identity such that identical inputs produce identical IDs.

It is a further object to enable code-to-data routing wherein a PCU is routed to a node that has the input content locally, minimizing data transfer.

---

## SUMMARY OF THE INVENTION

In one aspect, the invention provides a portable computation data structure comprising: a code segment (e.g., WASM bytecode) with content hash; a plurality of content-addressed input references (content hashes); optional inline parameters; an identity context comprising a principal identifier; and a deterministic identifier computed as a cryptographic hash of the code hash, input hashes, parameters, and identity principal.

In another aspect, the invention provides a method for routing a portable computation unit comprising: maintaining a data locator mapping content hashes to node identifiers; for a PCU with a set of input content hashes, determining which nodes hold the required content; selecting a node that satisfies constraints (capabilities, load, capacity); and routing the PCU to that node for execution.

In a further aspect, the PCU identifier is computed as BLAKE3(code.hash || inputs[] || parameters || identity.principal), ensuring determinism and content-addressability.

---

## DETAILED DESCRIPTION

### 1. PCU Structure

Fields: id (ContentHash), code (WasmModule), inputs (Vec<ContentHash>), parameters (Vec<u8>), identity (IdentityContext), constraints (ExecutionConstraints), created_at, result (optional).

**Code:** `nexus-pcu/src/pcu.rs` — `PCU` struct (lines 124–164).

### 2. Deterministic PCU ID

`compute_id(code, inputs, parameters, identity)` hashes: code.hash, each input hash, parameters, identity.principal. Result is ContentHash (e.g., BLAKE3). Same inputs ⇒ same ID.

**Code:** `nexus-pcu/src/pcu.rs` — `compute_id()` (lines 199–220).

### 3. WasmModule

Bytecode with content hash, name, memory_pages. `content_hash()` returns hash. `is_valid_header()` checks WASM magic.

**Code:** `nexus-pcu/src/pcu.rs` — `WasmModule` (lines 22–73).

### 4. Code-to-Data Routing

`DataLocator` maps ContentHash → Vec<NodeId>. `route_pcu(pcu, locator)` finds nodes that have the PCU's input content. Selects node by capabilities, load, capacity. PCU routes TO the data, not vice versa.

**Code:** `nexus-pcu/src/routing.rs` — `DataLocator`, `NodeInfo`, `route_pcu` (lines 72–150).

### 5. Execution Result and Proof

After execution, PCU receives `PCUResult` comprising output, output_hash, duration_us, memory_used, and cryptographic ExecutionProof. Proof binds PCU ID, inputs, output, identity.

**Code:** `nexus-pcu/src/pcu.rs` — `PCUResult` (lines 152–164). `nexus-executor/src/proof.rs` — `ExecutionProof::create`.

---

## CLAIMS (Draft)

**1.** A portable computation data structure comprising:

a code segment with an associated content hash;

a plurality of content-addressed input references, each comprising a content hash;

an identity context comprising a principal identifier, wherein the identity context is intrinsic to the structure and serialized therewith, not resolved externally; and

a deterministic identifier computed as a cryptographic hash of the content hash of the code segment, the content hashes of the input references, and the principal identifier, wherein the structure is routable to a node based on which nodes hold content corresponding to the content-addressed input references.

**2.** The structure of claim 1, further comprising inline parameters, wherein the deterministic identifier further incorporates the parameters.

**3.** The structure of claim 1, wherein the code segment comprises WebAssembly bytecode.

**4.** The structure of claim 1, wherein the structure is routable to a node based on which nodes hold content corresponding to the content-addressed input references.

**5.** A method for routing a portable computation unit, comprising:

maintaining a mapping of content hashes to node identifiers;

for a portable computation unit comprising a set of input content hashes, determining nodes that hold content corresponding to the input content hashes;

selecting a node from the determined nodes based on capability, load, and capacity; and

routing the portable computation unit to the selected node for execution.

**6.** The method of claim 5, wherein routing is performed such that the computation is executed where the data resides, minimizing data transfer.

---

## NOVELTY, DEFENSIBILITY, NON-OBVIOUSNESS & PRIOR ART

### Novelty

**Novelty assessment:** Combination-based novelty. Prior art includes content-addressed storage (IPFS, CAS), containers (Docker), serverless (Lambda), and dataflow execution. The invention's novelty lies in: (a) a single portable computation structure bundling code, content-addressed input references, identity, and deterministic ID in one artifact; (b) PCU ID = hash(code || inputs[] || parameters || identity.principal) for determinism; (c) code-to-data routing—PCU routes to node holding input content, minimizing data transfer; (d) DataLocator.route() selecting nodes with ALL inputs, filtering by capabilities and capacity, sorting by load*latency. No single reference combines portable computation unit with content-addressed inputs, intrinsic identity, and data-locality routing.

### Defensibility

**Design-around difficulty:** High. The PCU struct + compute_id + DataLocator.route chain is specific. A competitor would need to route computation to data (not data to computation) using content hashes, with deterministic PCU ID. Claims on "deterministic identifier computed as hash of code hash, input hashes, identity" and "routing to node that holds content corresponding to input content hashes" create a narrow design space.

### Non-Obviousness

**Inventive step:** A skilled person would not obviously combine WASM bytecode, content-addressed input references, intrinsic identity, and routing logic that selects nodes by content locality. Containers and serverless move data to compute. Content-addressed storage (e.g., IPFS) does not bundle computation with content references. Dataflow patents (US11093223B2, US20220050728A1) orchestrate execution but do not define a portable computation unit with identity and content-addressed routing. The inversion (compute moves to data) is non-obvious.

### Prior Art (Closest References)

| Reference | Type | Jurisdiction | Description | Distinguishing Feature |
|-----------|------|--------------|-------------|------------------------|
| US20110231647A1 | Patent | USPTO | Content-addressable data processing (Level 3, Personalweb) | Data access; no portable computation unit; no routing |
| US20240020261 | Patent app | USPTO | Peer-to-peer route in reconfigurable computing (SambaNova) | Hardware routing; no PCU; no content-addressed inputs |
| Fission Everywhere Computer / Homestar | Product | — | Content-addressed WASM, IPVM, tasks by CID | No intrinsic identity in payload; routing by capacity, not data locality; no execution proof binding |
| US20220050728A1 | Patent app | USPTO | Dynamic data driven orchestration (IBM) | Workload orchestration; no PCU structure; no identity in payload |
| US11093223B2 | Patent | USPTO | Procedural code to dataflow graph (Ab Initio) | Dataflow conversion; no portable unit; no content routing |
| Docker / Containers | Product | — | Code + env bundle | Identity external (IAM); no content-addressed routing |
| WebAssembly spec | Standard | W3C | Portable bytecode | Code format only; no PCU; no routing |

### Jurisdiction-Specific Search Databases & Queries

| Jurisdiction | Database | URL | Suggested Search Queries |
|--------------|----------|-----|--------------------------|
| **United States** | USPTO Patent Public Search | https://www.uspto.gov/patents/search | "content-address" AND "computation"; "code to data" OR "compute to data"; "portable" AND "WASM" AND routing |
| **European Union** | Espacenet | https://worldwide.espacenet.com/ | content-addressed computation; data locality routing execution; portable computation unit |
| **India** | IPO Patent Search | https://ipindiaservices.gov.in/ | content-addressed routing; portable computation; data locality execution |
| **International** | PATENTSCOPE (WIPO) | https://patentscope.wipo.int/search/en/advancedSearch.jsf | FTXT:(content-address) AND FTXT:(computation); FTXT:(portable) AND FTXT:(routing) |

---

## REFERENCE IMPLEMENTATION

| Component | File | Lines |
|-----------|------|-------|
| PCU struct | nexus-pcu/src/pcu.rs | 124–164 |
| compute_id | nexus-pcu/src/pcu.rs | 199–220 |
| WasmModule | nexus-pcu/src/pcu.rs | 22–73 |
| DataLocator | nexus-pcu/src/routing.rs | 72–100 |
| NodeInfo | nexus-pcu/src/routing.rs | 19–65 |
| route_pcu | nexus-pcu/src/routing.rs | 100+ |
| PCUResult | nexus-pcu/src/pcu.rs | 152–164 |

---

*End of IDF — Patent #5*
