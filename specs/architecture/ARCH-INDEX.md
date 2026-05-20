---
artifact: architecture-index
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
origin: brownfield
deployment_topology: single-service
traces_to: .factory/specs/prd.md
inputs:
  - .factory/specs/domain/domain-spec.md
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/semport/wirerust/wirerust-pass-1-deep-architecture-r3.md
  - docs/adr/0001-content-first-stream-dispatch.md
  - docs/adr/0002-modular-protocol-analyzers.md
  - docs/adr/0003-reporting-pipeline-layering.md
  - docs/adr/0004-process-wide-warning-atomics.md
input-hash: "[md5-TBD]"
---

# wirerust Architecture Index

> **Brownfield Mode:** This architecture describes the shipped system as of develop HEAD
> (post remediation-cycle PRs #69-#98, reconciled against 0082a0c). Do not treat this
> as aspirational design -- it is a formal record of what exists.

## Deployment Topology

`single-service` -- wirerust is a single statically-linked binary compiled from one Rust
crate. There is one deployment target, one tech stack (Rust 2024 / MSRV 1.91), and zero
network interfaces. The binary IS the complete deployment unit.


## Architecture Section Document Map

| File | Contents | Tokens (est.) |
|------|----------|---------------|
| `system-overview.md` | 5-layer pipeline narrative, data flow, key constraints | ~900 |
| `module-decomposition.md` | 20 components C-1..C-20 mapped to source files and SS-NN | ~1100 |
| `dependency-graph.md` | Import DAG, the one accepted cycle (L2<->L3), external crates | ~800 |
| `api-surface.md` | Public API: traits, structs, CLI surface, no network interfaces | ~900 |
| `verification-architecture.md` | Provable properties catalog, P0/P1 list, tooling selection | ~1100 |
| `purity-boundary-map.md` | Pure core vs effectful shell classification per module | ~800 |
| `tooling-selection.md` | Kani, proptest, cargo-fuzz, cargo-mutants rationale | ~600 |
| `verification-coverage-matrix.md` | VP-to-module coverage table | ~700 |


## Subsystem Registry

These are the canonical subsystem identifiers. Every behavioral contract
must carry exactly one of these values in its `subsystem:` frontmatter field.
The SS-NN numbering matches the PRD section scheme (bc-2.NN.NNN).

| SS-ID | Name | Capabilities | Primary Source Files | BC Count |
|-------|------|-------------|---------------------|----------|
| SS-01 | PCAP Ingestion | CAP-01 | reader.rs | 8 |
| SS-02 | Packet Decoding | CAP-02 + CAP-03 | decoder.rs | 15 |
| SS-04 | TCP Reassembly | CAP-04 | reassembly/{mod,flow,segment,handler,lifecycle,config,stats}.rs | 54 |
| SS-05 | Protocol Dispatch | CAP-05 | dispatcher.rs | 9 |
| SS-06 | HTTP Analysis | CAP-06 | analyzer/http.rs | 26 |
| SS-07 | TLS Analysis | CAP-07 | analyzer/tls.rs | 37 |
| SS-08 | DNS Analysis | CAP-08 | analyzer/dns.rs | 4 |
| SS-09 | Finding Emission | CAP-09 | findings.rs | 6 |
| SS-10 | MITRE Mapping | CAP-10 | mitre.rs | 9 |
| SS-11 | Reporting | CAP-11 | reporter/{mod,json,terminal,csv}.rs | 24 |
| SS-12 | CLI / Entry | CAP-12 | main.rs, cli.rs, lib.rs, summary.rs | 21 |
| SS-13 | Absent Behaviors | N/A (intentionally unwired) | cli.rs (flag parse only) | 4 |

> SS-03 is intentionally absent. See "CAP-03 / ss-02 Ruling" below.


## CAP-03 / ss-02 Ruling (PO Open Question Resolved)

**Question:** Should CAP-03 (Packet Decoding, L2-L4 header parsing) get its own subsystem
(ss-03), or should it be merged into ss-02 (Link-Type Gating)?

**Decision: MERGE is accepted. CAP-03 is part of ss-02.**

**Reasoning:**

1. **Single component, single file.** Both CAP-02 and CAP-03 are implemented entirely by
   C-5 (`decoder.rs`). There is no other source file involved in either capability. A
   subsystem boundary that splits one component into two subsystems would be artificial.

2. **Inseparable invocation.** Link-type gating is performed inside `decode_packet()` --
   the same function that performs L2-L4 header parsing. The gate and the decoding are
   a single pass through etherparse's layer-by-layer parse chain. They cannot be tested
   or verified independently.

3. **BC alignment already exists.** The PRD at section 2.2/2.3 explicitly co-locates
   CAP-03 BCs with CAP-02 under the BC-2.02.NNN namespace and notes "no separate ss-03
   directory is required". The BC-INDEX.md header for ss-02 already reads
   "Link-Type Gating / Packet Decoding (CAP-02 + CAP-03)".

4. **ss-03 gap does not break the numbering scheme.** The subsystem registry simply omits
   ss-03. BC numbering (bc-2.NN.NNN) uses the NN to identify the capability, not the
   subsystem ordinal -- ss-04 (TCP Reassembly) maps to CAP-04, not to subsystem ordinal 3.

**Binding rule:** All BC-2.02.NNN contracts (covering both link-type gating and packet
decoding) carry `subsystem: SS-02` in their frontmatter.


## Cross-Cutting Concerns

### Forensic Fidelity (INV-4 / ADR 0003)

All data from packet payloads flows raw (post-`from_utf8_lossy`) through every layer until
the reporter boundary. No analyzer, Finding constructor, or library boundary may apply
display-layer escaping. This is the most cross-cutting invariant -- it affects every
analyzer, the Finding struct, and all reporters.

### Bounded-Resource Design

Three independent caps operate at different layers:
- L2/SS-04: `MAX_FINDINGS = 10,000` on `TcpReassembler.findings` (with finalize bypass)
- L3/SS-06: `MAX_HEADER_BUF = 65,536` bytes per direction in HTTP header buffer
- L3/SS-07: `MAX_BUF = 65,536` bytes per direction in TLS buffer; `MAX_RECORD_PAYLOAD`
- L3/SS-06+07: `MAX_MAP_ENTRIES` on aggregate counter maps; `MAX_URIS = 10,000`
- L1/SS-04: `max_flows` and `memcap` configurable via `ReassemblyConfig`

### Single-Threaded Synchronous Execution

wirerust has zero async runtime, zero threads, and zero inter-process communication.
The entire pipeline is a synchronous function call chain from `run_analyze()` in main.rs.
This is not a constraint to work around -- it is the intentional design enabling formal
verification of state-machine properties.

### No Network I/O

wirerust reads only local files. There are zero syscalls to bind(), connect(), socket(),
or any network-related call. This is the basis for the "offline" forensic-tool guarantee.


## Architecture Decision Records

| ADR | Date | Decision | Subsystems Affected |
|-----|------|----------|---------------------|
| ADR 0001 | 2026-04-07 | Content-first stream dispatch (port-based fallback only) | SS-05 |
| ADR 0002 | 2026-04-07 | Modular protocol analyzer pattern (two-trait split) | SS-05, SS-06, SS-07, SS-08 |
| ADR 0003 | 2026-04-09 | Reporting pipeline layering (raw data / display-layer separation) | SS-06, SS-07, SS-09, SS-11 |
| ADR 0004 | 2026-05-19 | Process-wide warning atomics for one-shot bug tripwires | SS-04 |

All four ADRs are canonical. They reside in `docs/adr/` and are not duplicated here.
Architecture section files reference them by ID (e.g. "per ADR 0001") rather than
inlining their content.


## Architecture Debt

Surviving items from the 10-smell ingestion audit (smells 1, 4, 5, 6, 7, 10 remain; others
closed by PRs #69-#98). See `module-decomposition.md` for per-smell status. High-level
summary:

| Item | Status | Severity |
|------|--------|----------|
| O-01: Finding.timestamp universally None | Open | Medium (forensic gap) |
| O-03: Thresholds not empirically calibrated | Open | Low (P2) |
| O-04: 9 MITRE techniques staged but never emitted | Open | Low (documentation) |
| O-05: reassembly/mod.rs still ~691 LOC | Open | Low (partially closed) |
| O-06: Weak-cipher evidence vec unbounded | Open | Medium (NFR-RES-023) |
| Smell #4: L2<->L3 trait cycle (ADR 0002 accepted) | Advisory | Low |
| Smell #10: Loose TLS gate (byte[2] unchecked) | Open | Low (theoretical) |
