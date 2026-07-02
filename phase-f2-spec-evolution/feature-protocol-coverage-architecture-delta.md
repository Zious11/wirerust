---
document_type: architecture-delta
feature_id: feature-protocol-coverage
gate: D-320
title: "F2 Architecture Delta — Protocol Coverage Catalog (SS-18)"
status: draft
producer: architect
created: 2026-07-01
branch: develop
traces_to:
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/phase-f1-delta-analysis/feature-protocol-coverage-delta-analysis.md
  - .factory/phase-f1-delta-analysis/feature-protocol-coverage-research.md
---

# F2 Architecture Delta — Protocol Coverage Catalog (feature-protocol-coverage)

## 1. Overview

This document records the complete design-layer changes made in the F2 spec evolution
phase for cycle `feature-protocol-coverage` (human gate D-320). It covers new
subsystem SS-18, new component C-26, new ADR-012, and three new verification properties
VP-041, VP-042, and VP-043.

**F2P1 adversarial remediation (2026-07-01):** Eight design-layer findings from F2
adversarial Pass-1 (F-F2P1-003, 005, 006, 007, 008, 010, 011, 012) corrected in
ADR-012, SS-18, VP-INDEX, verification-architecture.md, verification-coverage-matrix.md,
ARCH-INDEX.md, and this document. Design decisions recorded in §10 for product-owner
BC alignment.

This document does NOT author behavioral contracts (BC-2.18.001..004 / BC-2.05.010..011
/ BC-2.12.022..024) — those are the product-owner's next deliverable. It provides the
subsystem ID, data model, module boundary, and anchoring guidance the product-owner
needs.

Approved scope (human gate D-320):
- OQ-1: hand-curated static catalog ~30 entries (7 supported + 9 ICS Tier-1 +
  5 L2-flagged + 9 IT); every entry category-tagged; port-102 collision documented
- OQ-2: dynamic detection via CoverageGapsSummary report section using Suricata tri-state
  vocabulary (known-supported / known-unsupported / unknown); grouped by transport+port
- OQ-3: `protocols` CLI subcommand; terminal table + global --json flag honored
- OQ-4: dynamic gap detection gated behind `--coverage-gaps` flag; NOT auto under --all
- OQ-5: TCP+UDP dynamic detection this cycle (D-320 approved scope; BACnet/IP UDP/47808
  IS flaggable); `(TransportProto, u16)` key distinguishes TCP vs UDP on same port;
  L2/multicast port-undetectable caveats remain (GOOSE/SV/PROFINET-RT/EtherCAT/Ethernet POWERLINK)

## 2. New Subsystem: SS-18 (Protocol Coverage Catalog)

**Subsystem ID:** SS-18
**Name:** Protocol Coverage Catalog
**Capabilities:** CAP-18
**Primary source file:** src/protocols.rs (new)
**Component:** C-26

### 2.1 Architecture Section File

Created: `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` (v1.0, draft)

This file is the canonical specification for SS-18. It defines:
- The `KnownProtocol` data model and `ProtocolCategory` / `Transport` enums
- The full ~30-entry catalog table
- The `SUPPORTED_PORTS` compile-time constant derivation approach
- Port 102 four-way collision (S7comm / S7comm-plus / IEC 61850 MMS / ICCP-TASE.2)
- L2/multicast `port_detectable: false` entries (GOOSE, SV, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK)
- Dynamic detection scope (TCP+UDP; L2/multicast structurally absent; port-102 collision caveat)
- Bounded-resource note for `unclassified_port_counts` HashMap (SS-05)

### 2.2 Data Model (src/protocols.rs — PURE CORE)

```rust
// NO L2 variant in ProtocolCategory — L2 detection is expressed by
// transport: Transport::LinkLayer + port_detectable: false (F-F2P1-003)
pub enum ProtocolCategory { ICS, IT }
pub enum Transport { Tcp, Udp, LinkLayer }

pub struct KnownProtocol {
    pub name:            &'static str,
    pub category:        ProtocolCategory,
    pub transport:       Transport,
    pub canonical_ports: &'static [u16],   // empty for LinkLayer-transport protocols
    pub ethertype:       Option<u16>,       // Some only for LinkLayer-transport protocols
    pub port_detectable: bool,              // false for LinkLayer/multicast entries
    pub description:     &'static str,
}

// Compile-time constant: classify() TCP port-fallback rules + decode-loop DNS path (port 53, no DispatchTarget)
const SUPPORTED_PORTS: &[u16] = &[502, 20000, 44818, 443, 8443, 80, 8080, 53];
// ARP is supported via DecodedFrame::Arp; flagged separately in supported_protocols()

pub static KNOWN_PROTOCOLS: &[KnownProtocol] = &[ /* ~30 entries */ ];

pub fn supported_protocols() -> Vec<&'static KnownProtocol> { ... }
pub fn unsupported_protocols() -> Vec<&'static KnownProtocol> { ... }
pub fn all_protocols() -> &'static [KnownProtocol] { KNOWN_PROTOCOLS }
```

**Purity boundary:** The entire module is pure core. No I/O, no global mutable state.

### 2.3 Subsystem Boundaries

SS-18 depends on nothing within wirerust's source tree. It is a leaf in the dependency
graph. Consuming subsystems:
- SS-12 (cli.rs / main.rs) — adds `protocols` subcommand and `--coverage-gaps` flag
- SS-05 (dispatcher.rs) — adds `unclassified_port_counts: HashMap<(TransportProto, u16), u64>`
  for TCP None-target flow tracking (key: `(Tcp, lower_port)` where `lower_port =
  min(src_port, dst_port)`); SS-12 (main.rs) adds a UDP unclassified counter with the
  same key type for UDP datagrams in the decode loop (key: `(Udp, min(src_port,
  dst_port))` — symmetric normalization prevents ephemeral-port noise; F-F2P1-006);
  SUPPORTED_PORTS relationship documented but SS-05 does NOT import from protocols.rs at
  runtime; `TransportProto` is a minimal `{Tcp, Udp}` enum in dispatcher.rs (not from
  protocols.rs, which has a third `LinkLayer` variant)

## 3. New ADR: ADR-012

**ADR ID:** ADR-012
**File:** `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`
**Status:** accepted
**Subsystems affected:** SS-18, SS-05, SS-12

Key decisions recorded:
1. Hand-curated static compile-time array (not external file) — zero I/O, verifiable
2. Suricata tri-state vocabulary (known-supported / known-unsupported / unknown)
3. Port-based detection caveats: TCP+UDP scope; port-102 four-way TCP collision;
   L2/multicast NOT port-detectable (only structural limitation remaining); heuristic
   disclaimer
4. ICS + core-IT scope (not ICS-only, not all-IANA)
5. SUPPORTED_PORTS compile-time constant = classify() TCP port-fallback rules + decode-loop DNS path (port 53); NOT a pure mirror of classify() — port 53 has no DispatchTarget variant; VP-041 guards supported_protocols()-vs-SUPPORTED_PORTS (ADR-012 Decision 5, F-F2P5-001)
6. TCP+UDP dynamic detection (D-320 OQ-5); (TransportProto, u16) key; BACnet/IP
   UDP/47808 IS flaggable; L2/multicast still port-undetectable (structural)
7. Category tagging (ICS / IT) — NO L2 variant in ProtocolCategory; L2 detection
   expressed by transport:LinkLayer + port_detectable:false (F-F2P1-003); `--ics-only`
   category filter NOT shipping this cycle (retained for display and future filtering
   only; F-F2P1-010)
8. `--coverage-gaps` explicit flag (NOT auto under `--all`)
9. `CoverageGapsSummary` as report section (NOT Finding entries)

## 4. New Verification Properties

### VP-041 (proptest, P1, draft — src/protocols.rs)

**Property:** Catalog oracle cross-check — supported classification agrees with
independent oracle (F-F2P1-008, DF-KANI-NONVACUITY-001).

The original partition/disjoint formulation was vacuously true because
`unsupported = KNOWN \ supported` holds by construction. VP-041 is reframed to use
an independent oracle that does NOT call `supported_protocols()` or
`unsupported_protocols()`:

```rust
// Independent oracle (does not call supported_protocols() / unsupported_protocols())
let oracle_supported: bool =
    entry.canonical_ports.iter().any(|p| SUPPORTED_PORTS.contains(p))
    || entry.name == "ARP";
// Assertion: catalog classification must agree with oracle
assert_eq!(is_supported_by_catalog(entry), oracle_supported);
```

**Harnesses:** 2 proptest harnesses (proptest_vp041_oracle_cross_check + proptest_vp041_partition_invariant)
- `proptest_vp041_oracle_cross_check` — for every KNOWN_PROTOCOLS entry, independently
  computes `oracle_supported` without calling `supported_protocols()` /
  `unsupported_protocols()`; non-vacuous and falsifiable if `supported_protocols()`
  diverges from the SUPPORTED_PORTS-intersection rule; does NOT detect
  `classify()`-vs-`SUPPORTED_PORTS` drift (ADR-012 Decision 5 — that gap is a documented
  convention, intentionally non-enforced at compile time)
- `proptest_vp041_partition_invariant` — supported∪unsupported = KNOWN_PROTOCOLS
  (partition completeness) AND supported∩unsupported = ∅ (disjoint)

**Traces:** BC-2.18.003, BC-2.18.004

### VP-042 (proptest, P1, draft — dispatcher.rs)

**Property:** Per-port unclassified-flow count accumulation exactness — TCP dispatcher
`on_flow_close` path only.
- `unclassified_port_counts.values().sum() == N` after N None-target on_flow_close calls
- Per-port count equals input frequency; key is `(Tcp, min(src_port, dst_port))`
  (F-F2P1-006 symmetric normalization)
- Classified-flow on_flow_close does NOT increment any counter

**SCOPE NOTE (F-F2P1-011):** VP-042 covers only `dispatcher.rs::on_flow_close`. UDP
packets in the `main.rs` decode loop are NOT routed through `on_flow_close` and are
NOT reachable by VP-042. UDP exactness/monotonicity for the decode-loop path is
covered by VP-043.

**Harnesses:** 3 proptest harnesses
- `proptest_vp042_total_count_equals_n`
- `proptest_vp042_per_port_count_equals_frequency`
- `proptest_vp042_no_count_spurious_on_classified_flows`

**VP-004 re-validation:** VP-004 (Kani, dispatcher classify(), P0, verified) MUST be
re-run at F6 as regression confirmation. The `classify()` function is unchanged, but
the new `unclassified_port_counts` HashMap field changes the `StreamDispatcher` struct;
Kani proof bounds must be re-checked.

**Traces:** BC-2.05.010, BC-2.05.011

### VP-043 (proptest, P1, draft — main.rs) — NEW, F-F2P1-011

**Property:** UDP decode-loop unclassified-packet count accumulation exactness (OQ-5
UDP path gap).

The `main.rs` decode loop processes UDP datagrams independently of `dispatcher.rs`.
Packets failing `dns_analyzer.can_decode()` are counted as unclassified; the key is
`(TransportProto::Udp, min(src_port, dst_port))`. VP-042 cannot reach this path.
VP-043 closes the OQ-5 UDP exactness/monotonicity gap jointly with VP-042 (VP-042
covers dispatcher path, VP-043 covers decode-loop path).

- Total unclassified count == N after N non-DNS UDP packets through decode loop
- No increment for classified UDP (DNS)
- Key is `(TransportProto::Udp, min(src_port, dst_port))` — symmetric normalization

**Harnesses:** 2 proptest harnesses
- `proptest_vp043_total_count_equals_n` — N non-DNS UDP packets yield total == N
- `proptest_vp043_no_increment_on_classified_udp` — classified UDP does NOT increment
  the counter

**Traces:** BC-2.05.010, BC-2.05.011

## 5. ARCH-INDEX.md Updates

Updated: `.factory/specs/architecture/ARCH-INDEX.md` (v2.5 → v2.6)

Changes applied:
- Version 2.5 → 2.6
- Modified field: new 2026-07-01 entry (comprehensive)
- Document Map: component count 25 → 26; `ss-18-protocol-coverage-catalog.md` row added
- Subsystem Registry: SS-18 row added
- Architecture Decision Records: ADR-012 row added
- Bounded-Resource Design: `unclassified_port_counts` HashMap note added (SS-18/SS-05)

## 6. VP-INDEX.md Updates

Updated: `.factory/specs/verification-properties/VP-INDEX.md` (v2.28 → v2.30 initial;
v2.30 → v2.31 after F2P1 adversarial remediation)

### 6.1 Initial F2 Design Layer (D-320) — v2.28 → v2.30

- total_vps: 40 → 42
- p1_count: 26 → 28
- proptest_count: 17 → 19
- Summary table: Total 40 → 42, P1 26 → 28
- Tool count table: proptest 17 → 19
- Complete VP Catalog: VP-041 and VP-042 rows added (after VP-040)
- P1 Properties list: VP-041 and VP-042 inserted (newest first ordering)
- Consistency Invariants: total 40 → 42, proptest 17 → 19, P1 26 → 28, draft 9 → 11

### 6.2 F2P1 Adversarial Remediation — v2.30 → v2.31

- total_vps: 42 → 43 (VP-043 added)
- p1_count: 28 → 29
- proptest_count: 19 → 20
- VP-041 harness renamed: `proptest_vp041_set_difference_correct` →
  `proptest_vp041_oracle_cross_check` (oracle reframe, F-F2P1-008)
- VP-042 key fixed: `(Udp, dst_port)` → `(Udp, min(src_port, dst_port))`; VP-043
  scope note added (F-F2P1-006, F-F2P1-011)
- VP-043 row added: UDP decode-loop accumulation, main.rs, proptest, P1, draft
- Consistency Invariants: total 42 → 43, proptest 19 → 20, P1 28 → 29, draft 11 → 12

## 7. Verification Architecture Document Updates

Updated: `.factory/specs/architecture/verification-architecture.md` (v2.24 → v2.26
initial; v2.26 → v2.27 after F2P1 adversarial remediation)

### 7.1 Initial F2 Design Layer (D-320) — v2.24 → v2.26

- Should Prove table: VP-041 and VP-042 rows added
- Tooling Selection proptest row: VP-041, VP-042 added to list (17 → 19)
- P1 enumeration list: VP-041 and VP-042 bullet points added
- Total counts updated (40 → 42, P1 26 → 28)

### 7.2 F2P1 Adversarial Remediation — v2.26 → v2.27

- VP-041 Should Prove row: oracle reframe (`proptest_vp041_oracle_cross_check`);
  non-vacuous per DF-KANI-NONVACUITY-001 (F-F2P1-008)
- VP-042 Should Prove row: key `(Udp, dst_port)` → `(Udp, min(src_port, dst_port))`;
  VP-043 scope note added (F-F2P1-006, F-F2P1-011)
- VP-043 Should Prove row added: UDP decode-loop accumulation, main.rs, proptest
- Tooling Selection proptest row: VP-043 added (19 → 20 VPs)
- P1 list: VP-043 added; VP-041 entry updated to oracle harness name;
  VP-042 entry updated with key fix + scope note

Updated: `.factory/specs/architecture/verification-coverage-matrix.md` (v1.40 → v1.42
initial; v1.42 → v1.43 after F2P1 adversarial remediation)

### 7.3 Initial F2 Design Layer (D-320) — v1.40 → v1.42

- VP-to-Module Mapping: VP-041 and VP-042 rows added
- Per-Module Coverage Totals: new `protocols.rs` module row added
- Per-Module Coverage Totals: `dispatcher.rs` row proptest 0 → 1, Total 1 → 2
- Totals row: proptest 17 → 19, overall 40 → 42
- Coverage notes: VP-041 and VP-042 notes added

### 7.4 F2P1 Adversarial Remediation — v1.42 → v1.43

- VP-041 row: oracle reframe (`proptest_vp041_oracle_cross_check`) (F-F2P1-008)
- VP-042 row: key `min(src_port, dst_port)`; VP-043 scope note (F-F2P1-006,
  F-F2P1-011)
- VP-043 row added after VP-042: main.rs, proptest
- Per-Module Coverage Totals: `main.rs` row added (0 Kani, 1 proptest, 0 fuzz,
  0 integration, total 1)
- Totals row: proptest 19 → 20, overall 42 → 43

## 8. Product-Owner Anchoring Guidance

The product-owner needs the following to write BC-2.18.001..004 / BC-2.05.010..011 /
BC-2.12.022..024:

### SS-18 BCs (BC-2.18.NNN)

Use `subsystem: SS-18` in frontmatter. The four expected BCs:

| BC ID | Suggested Topic |
|-------|----------------|
| BC-2.18.001 | `protocols` subcommand static catalog output: terminal table lists all KNOWN_PROTOCOLS with name, category, transport, port(s), supported status, EtherType |
| BC-2.18.002 | `protocols` subcommand JSON mode: output honors global --json flag with structured JSON |
| BC-2.18.003 | `supported_protocols()` returns exactly the entries whose canonical_ports intersect SUPPORTED_PORTS (plus ARP); `unsupported_protocols()` returns the complement |
| BC-2.18.004 | Partition invariant: supported ∪ unsupported == KNOWN_PROTOCOLS; disjoint invariant: supported ∩ unsupported == ∅ |

**Must include in BC-2.18.NNN:**
- Port 102 four-way collision caveat in display output (S7comm / S7comm-plus / MMS / ICCP-TASE.2)
- L2/multicast port_detectable: false caveat in display output

### SS-05 BCs (BC-2.05.010..011)

Use `subsystem: SS-05` in frontmatter.

| BC ID | Suggested Topic |
|-------|----------------|
| BC-2.05.010 | `unclassified_port_counts: HashMap<(TransportProto, u16), u64>` updated at `on_flow_close` for `DispatchTarget::None` TCP flows; key is `(Tcp, lower_port)` where `lower_port = min(src_port, dst_port)`; UDP unclassified packets tracked separately in decode loop with key `(Udp, min(src_port, dst_port))` — symmetric normalization prevents ephemeral-port noise (F-F2P1-006) |
| BC-2.05.011 | Per-(transport, port) counts are exact and monotonic; classified-flow `on_flow_close` does NOT update the TCP map; all TCP-map entries carry `TransportProto::Tcp`; TCP dispatcher-path exactness verified by VP-042; UDP decode-loop exactness verified by VP-043 |

### SS-12 BCs (BC-2.12.022..024)

Use `subsystem: SS-12` in frontmatter.

| BC ID | Suggested Topic |
|-------|----------------|
| BC-2.12.022 | `protocols` subcommand: `wirerust protocols` prints terminal table; `wirerust protocols --json` prints JSON |
| BC-2.12.023 | `--coverage-gaps` flag: when passed with `analyze`, appends CoverageGapsSummary section to output; NOT auto-enabled under `--all` |
| BC-2.12.024 | CoverageGapsSummary includes mandatory caveats: TCP+UDP scope; L2/multicast (GOOSE/SV/PROFINET-RT/EtherCAT/Ethernet POWERLINK) structurally absent from gap report (no TCP/UDP port); port-102 four-way TCP collision ambiguity |

**CoverageGapsSummary mandatory caveat text (fixed string — updated for TCP+UDP scope):**
> "Dynamic gap detection covers TCP and UDP flows. Layer-2 protocols (e.g., GOOSE,
> Sampled Values, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK) have no TCP/UDP
> port and are not represented in the gap report. Consult
> `wirerust protocols --unsupported` for L2 protocol coverage."

## 9. Files Modified in This Delta

### 9.1 Initial F2 Design Layer (D-320)

| File | Change | Version |
|------|--------|---------|
| `.factory/specs/architecture/ARCH-INDEX.md` | SS-18 row, ADR-012 row, Document Map, Bounded-Resource note | v2.5 → v2.6 |
| `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` | NEW | v1.0 |
| `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md` | NEW | accepted |
| `.factory/specs/verification-properties/VP-INDEX.md` | VP-041, VP-042 added | v2.28 → v2.30 |
| `.factory/specs/architecture/verification-architecture.md` | VP-041, VP-042 in Should Prove + P1 list + Tooling | v2.24 → v2.26 |
| `.factory/specs/architecture/verification-coverage-matrix.md` | VP-041, VP-042 rows + module rows + Totals | v1.40 → v1.42 |

### 9.2 F2-SCOPE-DRIFT-UDP-001 Resolution (TCP+UDP reconciliation)

| File | Change | Version |
|------|--------|---------|
| `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md` | Decision 6 corrected TCP-only → TCP+UDP; Decision 3a updated TCP-only caveat → L2-only structural caveat; Context corrected; Consequences updated HashMap key type to (TransportProto, u16) | accepted (modified) |
| `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` | Dynamic Detection Scope section updated TCP-only → TCP+UDP; BACnet/IP gap text updated (now detectable); Bounded-Resource Note updated with dual-counter design and (TransportProto, u16) key; Subsystem Purpose updated | v1.0 (modified) |
| `.factory/specs/architecture/ARCH-INDEX.md` | Bounded-Resource SS-18 note updated (TransportProto, u16) dual-counter; ADR-012 row updated TCP-only → TCP+UDP; SS-18 registry comment updated; modified log entry added | v2.6 → v2.7 |
| `.factory/specs/architecture/module-decomposition.md` | C-21 row updated: unclassified_port_counts field + TransportProto type note | v1.8 (modified) |
| `.factory/phase-f2-spec-evolution/feature-protocol-coverage-architecture-delta.md` | OQ-5 overview, SS-05 section, ADR decisions summary, BC-2.05.010/011 anchoring, mandatory caveat text — all updated for TCP+UDP | draft (modified) |

### 9.3 F2P1 Adversarial Remediation (F-F2P1-003, 005, 006, 007, 008, 010, 011, 012)

| File | Finding(s) | Change | Version |
|------|-----------|--------|---------|
| `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md` | F-F2P1-003, F-F2P1-006, F-F2P1-010 | Decision 7: L2 variant removed from ProtocolCategory, ICS/IT only, `--ics-only` softened to display-only; Decision 6: UDP key `dst_port` → `min(src_port, dst_port)` | accepted (modified) |
| `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` | F-F2P1-003, F-F2P1-005, F-F2P1-006, F-F2P1-007, F-F2P1-012 | ProtocolCategory ICS/IT only; HART-IP transport TCP+UDP → UDP (single-canonical); §Transport Model section added; `known` → `known-supported` in output spec; UDP key → `min(src_port, dst_port)`; bound `2 × 65,535` → `2 × 65,536` | v1.0 → v1.1 |
| `.factory/specs/architecture/ARCH-INDEX.md` | F-F2P1-003, F-F2P1-006, F-F2P1-008, F-F2P1-011, F-F2P1-012 | SS-18 registry comment: ProtocolCategory ICS/IT only, oracle VP-041, VP-043; Bounded-Resource note: key fix + `2 × 65,536` | v2.7 → v2.8 |
| `.factory/specs/verification-properties/VP-INDEX.md` | F-F2P1-006, F-F2P1-008, F-F2P1-011 | VP-041 oracle reframe; VP-042 key fix + scope note; VP-043 added | v2.30 → v2.31 |
| `.factory/specs/architecture/verification-architecture.md` | F-F2P1-006, F-F2P1-008, F-F2P1-011 | VP-041/042/043 Should Prove rows updated; P1 list updated; Tooling proptest VP-043 added | v2.26 → v2.27 |
| `.factory/specs/architecture/verification-coverage-matrix.md` | F-F2P1-006, F-F2P1-008, F-F2P1-011 | VP-041/042/043 rows updated; main.rs row added; Totals updated | v1.42 → v1.43 |
| `.factory/phase-f2-spec-evolution/feature-protocol-coverage-architecture-delta.md` | ALL F2P1 | ProtocolCategory ICS/IT only; VP-041 oracle reframe; VP-042 key fix + scope note; VP-043 section added; ADR Decision 7 note; UDP key fix in BC-2.05.010 anchoring | draft (modified) |

**Product-owner BC files requiring alignment (PO scope — NOT modified by architect):**
| File | Finding | Change needed |
|------|---------|---------------|
| `.factory/specs/bcs/BC-2.05.010.md` | F-F2P1-012 | Line 81: `2 × 65,535` → `2 × 65,536`; also adopt `min(src_port, dst_port)` UDP key per F-F2P1-006 |
| `.factory/specs/prd.md` | F-F2P1-006 | Line 2162: `(Udp, dst_port)` → `(Udp, min(src_port, dst_port))` |

## 10. Design Decisions for Product-Owner BC Alignment

The following design decisions were made during F2P1 adversarial remediation. The
product-owner must reflect these decisions when updating BC files.

### D-F2P1-001: ProtocolCategory has no L2 variant (F-F2P1-003)

`ProtocolCategory` is `{ ICS, IT }` only. GOOSE.category = ICS (not a third enum
variant). L2-protocol detection is expressed structurally: `transport:
Transport::LinkLayer AND port_detectable: false`. There is no `L2` category value
in the enum at any layer. BC-2.18.001/003/004 must not reference a `L2` category
variant in their postconditions or invariants.

### D-F2P1-002: Single-canonical-transport per KnownProtocol entry (F-F2P1-005)

The `Transport` field on `KnownProtocol` admits exactly one value (`Tcp`, `Udp`, or
`LinkLayer`). Protocols that run on both TCP and UDP have a single canonical transport
designated for the catalog. HART-IP canonical transport = UDP (port 5094); the TCP
variant is noted in `description` but not modeled in the type. BCs must not assert
multi-value transport fields.

### D-F2P1-003: UDP gap counter uses min(src_port, dst_port) (F-F2P1-006)

Both TCP (`dispatcher.rs::on_flow_close`) and UDP (`main.rs` decode loop) unclassified
counters use `min(src_port, dst_port)` as the port component of the HashMap key. This
prevents ephemeral-port noise: SNMP response from 161→52000 is keyed as `(Udp, 161)`,
not `(Udp, 52000)`. BC-2.05.010 and BC-2.05.011 must state this normalization.

### D-F2P1-004: CoverageGapsSummary tri-state uses "known-supported" (F-F2P1-007)

The authoritative tri-state terms (Suricata-derived, ADR-012 Decision 2) are:
`known-supported` / `known-unsupported` / `unknown`. The third term is
`known-supported`, NOT `known`. BC-2.12.023/024 and any BC referencing the
CoverageGapsSummary output schema must use `known-supported`.

### D-F2P1-005: VP-041 is a non-vacuous oracle cross-check (F-F2P1-008; updated F-F2P5-002)

The original partition/disjoint formulation of VP-041 was vacuously true by
construction. The harness is reframed to `proptest_vp041_oracle_cross_check`: an
independent oracle computes `oracle_supported` without calling
`supported_protocols()` or `unsupported_protocols()`, then asserts agreement.

VP-041 is falsifiable if `supported_protocols()` diverges from the
SUPPORTED_PORTS-intersection rule; it does NOT detect `classify()`-vs-`SUPPORTED_PORTS`
drift (that is a documented convention per ADR-012 Decision 5, intentionally
non-enforced at compile time — port 53/DNS lives in SUPPORTED_PORTS but has no
`classify()` rule and no `DispatchTarget` variant, by design).

A second harness, `proptest_vp041_partition_invariant`, guards partition completeness
(supported∪unsupported = KNOWN_PROTOCOLS) and disjointness (supported∩unsupported = ∅).
VP-041 therefore uses 2 proptest harnesses total (Pass-2 fix F-F2P2-001 propagated
here at Pass-5). BC-2.18.003/004 postconditions remain valid but the VP proof strategy
uses 2 harnesses.

### D-F2P1-006: VP-042 covers only dispatcher.rs on_flow_close; VP-043 covers main.rs UDP (F-F2P1-011)

VP-042 is scoped to `dispatcher.rs::on_flow_close` (TCP path). The UDP decode loop
in `main.rs` is a separate execution path not routed through `on_flow_close`. VP-043
(new) covers the `main.rs` UDP decode-loop unclassified-packet counter. OQ-5 UDP
exactness/monotonicity is jointly covered by VP-042 (dispatcher path) + VP-043
(decode-loop path). BC-2.05.010/011 may cite both VPs.

### D-F2P1-007: Port space is 0..=65535 = 65,536 values (F-F2P1-012)

The bounded-resource bound is `2 × 65,536` unique HashMap keys (one per transport ×
one per port value in 0..=65535). The prior `2 × 65,535` was off-by-one. BC-2.05.010
postconditions stating HashMap bounds must use 65,536, not 65,535.

### D-F2P1-008: --ics-only filter NOT shipping this cycle (F-F2P1-010)

`ProtocolCategory` is retained for display in the `protocols` subcommand output and
for future filtering. The `--ics-only` CLI flag does NOT ship this cycle. No BC may
require or describe a `--ics-only` filter. BC-2.12.022/023 must not reference it.
