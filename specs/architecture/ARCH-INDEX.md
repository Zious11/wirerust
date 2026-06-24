---
artifact: architecture-index
level: L4
version: "1.7"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-08
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified (all arch section files promoted); input-hash computed; SS-13/CAP-12 note added for clarity."
  - date: 2026-06-10
    actor: architect
    reason: "Pass-1 adversarial remediation (issue #8 F2): SS-15 BC count updated TBD→22; stale 'F2 in progress' comment replaced (all 22 SS-15 BCs now written)."
  - date: 2026-06-10
    actor: architect
    reason: "Issue #8 research-validated scope additions: SS-15 BC count updated 22→24 for BC-2.15.023 (ENABLE/DISABLE_UNSOLICITED→T0814) and BC-2.15.024 (malformed-frame anomaly→T0814). ADR-007 Decision 5 extended to match. No VP/catalog/count change."
  - date: 2026-06-12
    actor: architect
    reason: "F2 delta ARP security analyzer: SS-16 added to Subsystem Registry (CAP-16, analyzer/arp.rs, TBD BC count); ADR-008 added to ADR table; VP-024 to be added to arch section files in this burst."
  - date: 2026-06-13
    actor: architect
    reason: "Corpus-wide consistency audit remediation (CD-3/CD-4/CD-5): SS-04 BC count 54→55 (BC-2.04.055 added F2 issue #100); SS-09 BC count 6→7 (BC-2.09.007 added F2 issue #100); SS-16 BC count TBD→15 (all 15 BC-2.16.001..015 written, F2 issue #9 complete); stale inline comment on SS-16 row removed."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-12 corpus debt cleanup: Document Map '21 components C-1..C-21' corrected to '24 components C-1..C-24' (C-22 Modbus, C-23 ARP, C-24 DNP3 shipped); O-04 Architecture Debt entry corrected '9 MITRE techniques' → '8 MITRE techniques (SEEDED 25 − EMITTED 17 = 8; domain-debt.md, PRD all say 8; the 9 was pre-F2-ARP stale)' (F-1 and F-D12-H01)."
  - date: 2026-06-13
    actor: architect
    reason: "Pass-13 corpus remediation (F-A13-005): module-criticality.md added to Architecture Section Document Map with its actual path (.factory/specs/module-criticality.md). Previously absent from the Document Map despite being referenced by architecture peers."
  - date: 2026-06-13
    actor: architect
    reason: "O-01 closure propagation: Architecture Debt table row updated Open→CLOSED (21/22 sites wired STORY-097/098/099+STORY-102..110; BC-2.04.054 summary finding timestamp:None by design); open-item set enumeration updated to O-02..O-08 (O-01 now closed). Version bump 1.4→1.5."
  - date: 2026-06-17
    actor: architect
    reason: "F2 Phase Spec Evolution (issue #259): ADR-0003 row extended with v0.8.0 display-layer aggregation subsection (collapse identical findings, --no-collapse, K=3 evidence sampling, flat-mode-only scope). ADR 0003 date updated to reflect extension."
  - date: 2026-06-17
    actor: architect
    reason: "F2 Phase Spec Evolution (issue #62): ADR-0003 row extended with v0.9.0 render-mode enum subsection (FindingsRender enum, illegal-state elimination rationale, migration map, semver v0.9.0 consequence, Default omission decision)."
  - date: 2026-06-18
    actor: architect
    reason: "F3 scope correction (issue #62): ADR-0003 v0.9.0 migration-map code block corrected — original snippet used *mitre and no_collapse which are out of scope inside run_analyze; corrected to use the in-scope params show_mitre_grouping and collapse_findings (function signature lines 107-108). Prose added to make explicit that the --mitre/--no-collapse → bool resolution stays at the main() call site (lines 79-80), collapse_findings_from_flag is unchanged, and the run_analyze signature is unchanged. Behavior is identical; only the variable names/layer cited in the migration map are corrected."
  - date: 2026-06-19
    actor: architect
    reason: "F2 Phase Spec Evolution (FE-001 pcapng reader support): ADR-009 added to Architecture Decisions table — magic-byte auto-detection, Option A parser (pcap-file 2.0.0 PcapNgReader +0 crates), SHB/IDB/EPB/SPB block coverage, multi-IDB link-type-agreement policy, pure-core timestamp-conversion helper, BC-2.01.004 retired/inverted. SS-01 affected."
  - date: 2026-06-20
    actor: architect
    reason: "ADR-009 rev 9→rev 10 (Decision 23): first-SHB btl=8 maps to E-INP-008 (not E-INP-010). PcapNgParser::new raises InvalidField(invalid magic number) for btl-degenerate inputs — same arm as genuine invalid-BOM; indistinguishable at API level. No VP count change, no section file change, no subsystem change. BC-2.01.010 EC-008/AC-004b/PC5/Canonical Test Vectors require PO update. test_BC_2_01_010_shb_framing_rejection_e_inp_010 requires rename+reassertion by implementer (Decision 23 Implementer Directive). No code change needed — existing mapper is correct."
  - date: 2026-06-20
    actor: architect
    reason: "ADR-009 rev 10→rev 11 (Decision 24): IDB structural validation (reserved!=0, block length<8) is enforced by InterfaceDescriptionBlock::from_slice inside next_raw_block (parser.rs:103-105, interface_description.rs:47-49) before wirerust receives the RawBlock. wirerust string-matches InvalidField('InterfaceDescriptionBlock: reserved != 0') and InvalidField('InterfaceDescriptionBlock: block length < 8') in map_err to produce E-INP-008; unmatched InvalidField falls through to E-INP-010 catch-all. Regression guard test_BC_2_01_011_nonzero_reserved_e_inp_008 pins E-INP-008 and refutes E-INP-010. Explicit IDB sibling of Decision 23 (SHB string coupling). BC-2.01.011 PC4/EC-010 require PO correction (BC incorrectly says wirerust mirrors the check; correct: wirerust delegates to crate and remaps via string match). No code change needed; no VP count change; no section file change; no subsystem change."
  - date: 2026-06-21
    actor: architect
    reason: "ADR-009 rev 11→rev 12 (D-188, F5 Pass-1 fixes, PR #287 merged develop=97c66b0): Decision 25 — decode_epb_body extracted as pub #[doc(hidden)] pure-core function (VP-027 Kani anchor per footnote [^vp025-027-module-anchor]; F-F5P1-001 fix; genuine non-vacuous VP-027 proof now possible). Decision 26 — PcapSource.is_pcapng: bool discriminant field added to carry the pcap/pcapng format decision from from_pcap_reader branch point; format_zero_packet_notice reads is_pcapng instead of calling read_magic a second time, eliminating TOCTOU mislabel and redundant open (F-F5P1-003 fix; BC-2.01.009 PC6 Decision 19 implementation gap closed). SS-01 affected. No BC count change. BC-2.01.013 v1.10 (O-2 accepted-behavior note for SPB check-ordering asymmetry)."
  - date: 2026-06-22
    actor: spec-steward
    reason: "Maintenance sweep maint-2026-06-22 (F-MAJ-001): reconcile stale BC-count annotations in Subsystem Registry. SS-01 (PCAP Ingestion) 8→17 (17 active = 8 pre-FE-001 + 10 new BC-2.01.009..018 − 1 retired BC-2.01.004). SS-11 (Reporting) 29→34 (34 active = 29 + 5 grouped-collapse BC-2.11.030..034). BC-INDEX v1.69 is the authoritative source and was already correct; only the ARCH-INDEX annotation was stale. Version bump 1.5→1.6."
  - date: 2026-06-24
    actor: architect
    reason: "Feature Mode F2 (feature-enip-v0.11.0, issue #316): SS-17 added to Subsystem Registry (EtherNet/IP + CIP Analysis, CAP-17, analyzer/enip.rs, 24 BCs); ADR-010 added to Architecture Decisions table; Document Map component count updated 24→25 (C-25 EtherNet/IP + CIP); Bounded-Resource Design note extended with SS-17 carry-buffer/MAX_FINDINGS/MALFORMED_ANOMALY_THRESHOLD entries. Version bump 1.6→1.7."
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
input-hash: "ae3222a"
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
| `module-decomposition.md` | 25 components C-1..C-25 mapped to source files and SS-NN | ~1100 |
| `dependency-graph.md` | Import DAG, the one accepted cycle (L2<->L3), external crates | ~800 |
| `api-surface.md` | Public API: traits, structs, CLI surface, no network interfaces | ~900 |
| `verification-architecture.md` | Provable properties catalog, P0/P1 list, tooling selection | ~1100 |
| `purity-boundary-map.md` | Pure core vs effectful shell classification per module | ~800 |
| `tooling-selection.md` | Kani, proptest, cargo-fuzz, cargo-mutants rationale | ~600 |
| `verification-coverage-matrix.md` | VP-to-module coverage table | ~700 |
| `.factory/specs/module-criticality.md` | Module kill-rate tier classification (CRITICAL/HIGH/MEDIUM/LOW) for all 24 components | ~500 |


## Subsystem Registry

These are the canonical subsystem identifiers. Every behavioral contract
must carry exactly one of these values in its `subsystem:` frontmatter field.
The SS-NN numbering matches the PRD section scheme (bc-2.NN.NNN).

| SS-ID | Name | Capabilities | Primary Source Files | BC Count |
|-------|------|-------------|---------------------|----------|
| SS-01 | PCAP Ingestion | CAP-01 | reader.rs | 17 |
| SS-02 | Packet Decoding | CAP-02 + CAP-03 | decoder.rs | 15 |
| SS-04 | TCP Reassembly | CAP-04 | reassembly/{mod,flow,segment,handler,lifecycle,config,stats}.rs | 55 |
| SS-05 | Protocol Dispatch | CAP-05 | dispatcher.rs, analyzer/mod.rs | 9 |
| SS-06 | HTTP Analysis | CAP-06 | analyzer/http.rs | 26 |
| SS-07 | TLS Analysis | CAP-07 | analyzer/tls.rs | 37 |
| SS-08 | DNS Analysis | CAP-08 | analyzer/dns.rs | 4 |
| SS-09 | Finding Emission | CAP-09 | findings.rs | 7 |
| SS-10 | MITRE Mapping | CAP-10 | mitre.rs | 9 |
| SS-11 | Reporting | CAP-11 | reporter/{mod,json,terminal,csv}.rs | 34 |
| SS-12 | CLI / Entry | CAP-12 | main.rs, cli.rs, lib.rs, summary.rs | 21 |
| SS-13 | Absent Behaviors | CAP-12 | cli.rs (flag parse only) | 4 | <!-- intentional: SS-13 is a sub-classification of CAP-12 (absent/intentionally-excluded behaviors), not a separate capability; see prd.md §2.13 -->
| SS-14 | Modbus/ICS Analysis | CAP-14 | analyzer/modbus.rs | 25 | <!-- Feature cycle issue #7; ADR-005; BC-2.14.001..025 all written; F2 adversarial review complete -->
| SS-15 | DNP3/ICS Analysis | CAP-15 | analyzer/dnp3.rs | 24 | <!-- Feature cycle issue #8; ADR-007; BC-2.15.001..024 written (F2 complete + issue #8 research-validated scope additions: BC-2.15.023 ENABLE/DISABLE_UNSOLICITED→T0814, BC-2.15.024 malformed-frame anomaly→T0814) -->
| SS-16 | ARP Security Analysis | CAP-16 | analyzer/arp.rs | 15 |
| SS-17 | EtherNet/IP + CIP Analysis | CAP-17 | analyzer/enip.rs | 24 | <!-- Feature cycle feature-enip-v0.11.0 issue #316; ADR-010; BC-2.17.001..024 (product-owner writes in F2); TCP/44818 explicit messaging MVP; UDP/2222 deferred --> |

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
- L3/SS-14: `MAX_PENDING_TRANSACTIONS = 256` per Modbus flow (transaction correlation table); `MAX_FINDINGS = 10,000` shared constant
- L3/SS-15: carry buffer bounded to 292 bytes per DNP3 flow (max DNP3 link frame); `MAX_MASTER_ADDRS` (bounded master-address tracking per flow)
- L3/SS-17: carry buffer bounded to 600 bytes per ENIP flow (`MAX_ENIP_CARRY_BYTES = 600`); `MAX_FINDINGS = 10,000` shared constant; `MALFORMED_ANOMALY_THRESHOLD = 3` for T0814 windowed gate (ADR-010 Decision 3)
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
| ADR 0003 | 2026-04-09 (extended 2026-06-17 ×2) | Reporting pipeline layering (raw data / display-layer separation); v0.8.0 extension: display-layer aggregation — collapse identical findings in TerminalReporter, `--no-collapse` opt-out, K=3 evidence sampling, flat mode only for v0.8.0 (STORY-118); v0.9.0 extension: render-mode enum — `FindingsRender {Grouped, FlatCollapsed, FlatExpanded}` replaces bool pair, illegal-state elimination, `Default` omit decision, semver v0.9.0 (issue #62) | SS-06, SS-07, SS-09, SS-11 |
| ADR 0004 | 2026-05-19 | Process-wide warning atomics for one-shot bug tripwires | SS-04 |
| ADR 0005 | 2026-06-09 | Binary ICS protocol integration (Modbus TCP): port-only classification exception, PDU-oriented manual parsing, full transaction-correlation state, ICS-matrix MITRE representation | SS-05, SS-10, SS-14 |
| ADR 0006 | 2026-06-09 | Multi-technique Finding attribution: `mitre_technique: Option<String>` → `mitre_techniques: Vec<String>`; one-finding-N-tags aligned with Sigma/Elastic standard; volume control via aggregation not tag-suppression; v0.3.0 breaking schema change | SS-09, SS-10, SS-11, SS-14 |
| ADR 0007 | 2026-06-10 | DNP3 TCP integration (Issue #8): port-20000 Rule 6 port-fallback classification, `DispatchTarget::Dnp3`, carry-buffer + CRC-block-skip parse, FIR=1-only app-layer extract, corrected MITRE technique set (T1691.001+T0827 new; T0803/T0855 revoked in ics-attack-19.1), new `MitreTactic::IcsImpact` variant, VP-004 oracle extension, VP-007 SEEDED 21→23 | SS-05, SS-10, SS-15 |
| ADR 0008 | 2026-06-12 | ARP link-layer integration: `DecodedFrame` enum from `decode_packet` (Ip/Arp variants), `ArpFrame` struct, etherparse 0.20 `NetSlice::Arp`/`LaxNetSlice::Arp` match fix, `ArpAnalyzer` binding table (MAX_ARP_BINDINGS=65536 LRU), 5 detections (D1 spoof/D2 GARP/D3 storm/D11 malformed/D12 L2/L3 mismatch), MITRE T0830+T1557.002, VP-007 SEEDED 23→25, BC-2.02.009 revised | SS-02, SS-10, SS-16 |
| ADR 0009 | 2026-06-19 | pcapng capture-format reader support: magic-byte auto-detection (peek without consuming), Option A parser (pcap-file 2.0.0 PcapNgReader, +0 new crates), SHB/IDB/EPB/SPB block coverage, multi-IDB link-type-agreement policy, pure-core timestamp-conversion helper (if_tsresol/if_tsoffset), BC-2.01.004 retired/inverted. **Rev 12 (2026-06-21, D-188):** Decision 25 — `decode_epb_body` extracted as `pub #[doc(hidden)]` pure-core function (VP-027 Kani anchor; F-F5P1-001 fix, PR #287). Decision 26 — `PcapSource.is_pcapng: bool` discriminant field added; `format_zero_packet_notice` reads it instead of calling `read_magic` a second time (F-F5P1-003 fix; eliminates TOCTOU mislabel and redundant open). **Rev 13 (2026-06-22, D-192):** Decision 27 — 4 GiB file-size guard (MAX_PCAPNG_FILE_BYTES = 4_294_967_296) added to `from_file` before `read_to_end`; rejection surfaces as E-INP-014 (F6-SEC-A fix; PR #296 feddbd1). Decision 28 — `MAX_INTERFACE_TABLE_ENTRIES = 65535` cap added to IDB-parse loop; excess IDB rejected as E-INP-015 (F6-SEC-B fix; PR #296 feddbd1). Both guards apply only to the `from_file` / `from_pcap_reader` path; residual unbounded accumulation on `from_pcap_reader` STREAM path is latent debt (SEC-008, ADR-009 Decision 13 all-in-memory model scope). | SS-01 |
| ADR 0010 | 2026-06-24 | EtherNet/IP + CIP TCP integration (Issue #316): port-44818 Rule 7 port-fallback classification, `DispatchTarget::Enip`, two-level ENIP→CPF→CIP manual binary parser, 600-byte carry buffer cap (MAX_ENIP_CARRY_BYTES = 600, justified in Decision 3), ForwardOpen connection-lifecycle tracking in-scope for v0.11.0, UDP/2222 implicit I/O deferred, corrected MITRE technique set (T0858+T0816+T1693.001 new; T0857/T0855/T0856 revoked in ics-attack-19.1), new `MitreTactic::IcsExecution` variant, ForwardOpen technique gap documented (no dedicated ICS technique — T1692.001 only when connection demonstrably carries unauthorized command), VP-004 oracle extension, VP-007 SEEDED 25→28 EMITTED 17→19 | SS-05, SS-10, SS-17 |

ADRs 0001–0004 are canonical and reside in `docs/adr/`. ADR 0005 onwards reside in
`.factory/specs/architecture/decisions/`. Architecture section files reference them by ID
(e.g. "per ADR 0001") rather than inlining their content.


## Architecture Debt

Surviving items from the 10-smell ingestion audit (smells 1, 4, 5, 6, 7, 10 remain; others
closed by PRs #69-#98). See `module-decomposition.md` for per-smell status. High-level
summary:

| Item | Status | Severity |
|------|--------|----------|
| O-01: Finding.timestamp universally None | CLOSED (21/22 sites wired; BC-2.04.054 summary finding timestamp:None by design — STORY-097/098/099 + STORY-102..110) | Medium (forensic gap) |
| O-03: Thresholds not empirically calibrated | Open | Low (P2) |
| O-04: MITRE techniques staged but never emitted — post-v0.11.0: SEEDED 28 − EMITTED 19 = 9 catalogue-only (T1693.001 staged in v0.11.0 but no BC emits it yet; T1692.002/T0885/T0831/T0835/T0830 etc. pre-existing staged) | Open | Low (documentation) |
| O-05: reassembly/mod.rs still ~691 LOC | Open | Low (partially closed) |
| O-06: Weak-cipher evidence vec unbounded | Open | Medium (NFR-RES-023) |
| Smell #4: L2<->L3 trait cycle (ADR 0002 accepted) | Advisory | Low |
| Smell #10: Loose TLS gate (byte[2] unchecked) | Open | Low (theoretical) |

**Items tracked in domain-debt.md only (intentionally absent from this table):**
- O-02 (User-Agent absent-vs-empty asymmetry): a documented domain design decision,
  not an architecture defect. Only `Some("")` fires; absent UA (`None`) is intentionally
  ignored per research cited in http.rs:319-343. See domain-debt.md O-02.
- O-07 (rayon declared but never imported): build/dependency debt in Cargo.toml; has no
  architecture surface, no runtime impact, and no module boundary implication. Fix is a
  one-line Cargo.toml edit. See domain-debt.md O-07.
- O-08 (dns.rs module doc-comment describes unimplemented detection): stale/aspirational
  //! comment claims qname parsing, DGA-class entropy, NXDOMAIN spikes, and rare-TLD
  lookups; actual DnsAnalyzer only increments two counters and always returns Vec::new().
  Documentation debt only -- no architecture boundary implication. Fix is correcting the
  //! header. See domain-debt.md O-08.

O-01 is CLOSED (see table above). The remaining open-item set is O-02 through O-08. O-02, O-07, and O-08 are tracked in
`.factory/specs/domain/domain-debt.md` rather than this table because they fall outside
the architecture layer's scope.
