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
subsystem SS-18, new component C-26, new ADR-012, and two new verification properties
VP-041 and VP-042.

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
- OQ-5: TCP-only dynamic detection this cycle; BACnet UDP/47808 structural gap documented
  with mandatory caveat; UDP gap detection is immediate follow-on

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
- L2/multicast `port_detectable: false` entries (GOOSE, SV, PROFINET-RT/DCP, EtherCAT)
- Dynamic detection scope caveat (TCP-only, BACnet UDP mandatory caveat)
- Bounded-resource note for `unclassified_port_counts` HashMap (SS-05)

### 2.2 Data Model (src/protocols.rs — PURE CORE)

```rust
pub enum ProtocolCategory { ICS, IT, L2 }
pub enum Transport { Tcp, Udp, LinkLayer }

pub struct KnownProtocol {
    pub name:            &'static str,
    pub category:        ProtocolCategory,
    pub transport:       Transport,
    pub canonical_ports: &'static [u16],   // empty for L2 protocols
    pub ethertype:       Option<u16>,       // Some only for L2 protocols
    pub port_detectable: bool,              // false for L2/multicast entries
    pub description:     &'static str,
}

// Compile-time constant mirrors dispatcher.rs::classify() port rules
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
- SS-05 (dispatcher.rs) — adds `unclassified_port_counts: HashMap<(u16, u16), u64>`
  for CoverageGapsSummary; SUPPORTED_PORTS relationship documented but SS-05 does NOT
  import from protocols.rs at runtime (drift is a compile-time documentation invariant)

## 3. New ADR: ADR-012

**ADR ID:** ADR-012
**File:** `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`
**Status:** accepted
**Subsystems affected:** SS-18, SS-05, SS-12

Key decisions recorded:
1. Hand-curated static compile-time array (not external file) — zero I/O, verifiable
2. Suricata tri-state vocabulary (known-supported / known-unsupported / unknown)
3. Port-based detection caveats: TCP-only this cycle; port-102 four-way collision;
   L2/multicast NOT port-detectable; heuristic disclaimer
4. ICS + core-IT scope (not ICS-only, not all-IANA)
5. SUPPORTED_PORTS compile-time mirror of classify() — drift guarded by VP-041
6. TCP-only dynamic detection; BACnet UDP/47808 deferred with high-priority follow-on
7. Category tagging (ICS / IT / L2)
8. `--coverage-gaps` explicit flag (NOT auto under `--all`)
9. `CoverageGapsSummary` as report section (NOT Finding entries)

## 4. New Verification Properties

### VP-041 (proptest, P1, draft — src/protocols.rs)

**Property:** Catalog set-difference correctness — partition and disjoint invariants.
- `supported_protocols() ∪ unsupported_protocols() == KNOWN_PROTOCOLS`
- `supported_protocols() ∩ unsupported_protocols() == ∅`
- Every KNOWN_PROTOCOLS entry appears in exactly one output set

**Harnesses:** 2 proptest harnesses
- `proptest_vp041_set_difference_correct` (partition + disjoint via SUPPORTED_PORTS mask)
- `proptest_vp041_partition_invariant` (each entry in exactly one set)

**Traces:** BC-2.18.003, BC-2.18.004

### VP-042 (proptest, P1, draft — dispatcher.rs)

**Property:** Per-port unclassified-flow count accumulation exactness.
- `unclassified_port_counts.values().sum() == N` after N None-target on_flow_close calls
- Per-port count equals input frequency
- Classified-flow on_flow_close does NOT increment any counter

**Harnesses:** 3 proptest harnesses
- `proptest_vp042_total_count_equals_n`
- `proptest_vp042_per_port_count_equals_frequency`
- `proptest_vp042_no_count_spurious_on_classified_flows`

**VP-004 re-validation:** VP-004 (Kani, dispatcher classify(), P0, verified) MUST be
re-run at F6 as regression confirmation. The `classify()` function is unchanged, but
the new `unclassified_port_counts` HashMap field changes the `StreamDispatcher` struct;
Kani proof bounds must be re-checked.

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

Updated: `.factory/specs/verification-properties/VP-INDEX.md` (v2.28 → v2.30)

Changes applied:
- total_vps: 40 → 42
- p1_count: 26 → 28
- proptest_count: 17 → 19
- Summary table: Total 40 → 42, P1 26 → 28
- Tool count table: proptest 17 → 19
- Complete VP Catalog: VP-041 and VP-042 rows added (after VP-040)
- P1 Properties list: VP-041 and VP-042 inserted (newest first ordering)
- Consistency Invariants: total 40 → 42, proptest 17 → 19, P1 26 → 28, draft 9 → 11

## 7. Verification Architecture Document Updates

Updated: `.factory/specs/architecture/verification-architecture.md` (v2.24 → v2.26)

Changes applied:
- Should Prove table: VP-041 and VP-042 rows added
- Tooling Selection proptest row: VP-041, VP-042 added to list (17 → 19)
- P1 enumeration list: VP-041 and VP-042 bullet points added
- Total counts updated (40 → 42, P1 26 → 28)

Updated: `.factory/specs/architecture/verification-coverage-matrix.md` (v1.40 → v1.42)

Changes applied:
- VP-to-Module Mapping: VP-041 and VP-042 rows added
- Per-Module Coverage Totals: new `protocols.rs` module row added
- Per-Module Coverage Totals: `dispatcher.rs` row proptest 0 → 1, Total 1 → 2
- Totals row: proptest 17 → 19, overall 40 → 42
- Coverage notes: VP-041 and VP-042 notes added

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
| BC-2.05.010 | `unclassified_port_counts: HashMap<(u16, u16), u64>` updated at on_flow_close for DispatchTarget::None flows; key is direction-normalized port pair (src_port ≤ dst_port) |
| BC-2.05.011 | Per-port-pair counts are exact and monotonic; classified-flow on_flow_close does NOT update the map |

### SS-12 BCs (BC-2.12.022..024)

Use `subsystem: SS-12` in frontmatter.

| BC ID | Suggested Topic |
|-------|----------------|
| BC-2.12.022 | `protocols` subcommand: `wirerust protocols` prints terminal table; `wirerust protocols --json` prints JSON |
| BC-2.12.023 | `--coverage-gaps` flag: when passed with `analyze`, appends CoverageGapsSummary section to output; NOT auto-enabled under `--all` |
| BC-2.12.024 | CoverageGapsSummary includes mandatory caveats: TCP-only scope, BACnet UDP/47808 structural gap, L2/multicast not port-detectable |

**CoverageGapsSummary mandatory caveat text (fixed string):**
> "Dynamic gap detection covers TCP flows only. UDP-based protocols (e.g. BACnet/IP on
> 47808, SNMP, NTP, PROFINET RPC) and Layer-2 protocols (GOOSE, Sampled Values,
> PROFINET-RT/DCP, EtherCAT) are not represented in the gap report. Consult the static
> `protocols` catalog for the full known-protocol set."

## 9. Files Modified in This Delta

| File | Change | Version |
|------|--------|---------|
| `.factory/specs/architecture/ARCH-INDEX.md` | SS-18 row, ADR-012 row, Document Map, Bounded-Resource note | v2.5 → v2.6 |
| `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` | NEW | v1.0 |
| `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md` | NEW | accepted |
| `.factory/specs/verification-properties/VP-INDEX.md` | VP-041, VP-042 added | v2.28 → v2.30 |
| `.factory/specs/architecture/verification-architecture.md` | VP-041, VP-042 in Should Prove + P1 list + Tooling | v2.24 → v2.26 |
| `.factory/specs/architecture/verification-coverage-matrix.md` | VP-041, VP-042 rows + module rows + Totals | v1.40 → v1.42 |
