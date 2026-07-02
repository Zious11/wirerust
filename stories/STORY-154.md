---
document_type: story
story_id: STORY-154
title: "`--coverage-gaps` Opt-In Flag + `CoverageGapsSummary` Tri-State Report + Mandatory Caveats (BC-2.12.023 + BC-2.12.024)"
epic_id: E-21
wave: 69
points: 8
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-protocol-coverage
github_issue: feature-protocol-coverage
subsystems: [SS-12, SS-18]
target_module: main
depends_on: [STORY-151, STORY-152, STORY-153]
blocks: []
behavioral_contracts:
  - BC-2.12.023
  - BC-2.12.024
verification_properties:
  - VP-041
  - VP-042
  - VP-043
assumption_validations: []
risk_mitigations: []
# BC status: all BCs authored and anchored (F2 convergence complete)
# DF-CANONICAL-FRAME-HOLDOUT-001: This story carries canonical-value ACs per obligation #2
# (BACnet/IP UDP/47808 as known-unsupported; TCP/102 port-102 collision).
inputs:
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.023.md
  - .factory/specs/behavioral-contracts/ss-12/BC-2.12.024.md
  - .factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md
input-hash: "56ad679"
---

# STORY-154: `--coverage-gaps` Flag + CoverageGapsSummary Tri-State Report + Mandatory Caveats

> **DF-CANONICAL-FRAME-HOLDOUT-001 STORY**: This story contains mandatory canonical-value ACs
> for protocol framing invariants used in the tri-state classification (TCP/102 collision, BACnet/IP
> UDP/47808 known-unsupported classification).

## Narrative

**As a** network security analyst running `wirerust analyze`,
**I want** an opt-in `--coverage-gaps` flag that causes wirerust to track and report unclassified
TCP flows and UDP packets by (transport, port), presented as a `CoverageGapsSummary` section
with tri-state classification (known-unsupported / unknown / known-supported), mandatory
L2/multicast limitation caveat, and a port-102 collision annotation,
**so that** I can identify which non-dissected protocols are most active on my network without
generating false impressions from silently-absent L2 protocols or ambiguous shared-port entries.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.12.023 | v1.2 | `--coverage-gaps` Flag Is Opt-In; NOT Auto-Enabled Under `analyze --all`; Appends CoverageGapsSummary When Set | Primary: flag wiring, opt-in semantics, analyze --all independence, CoverageGapsSummary as named section |
| BC-2.12.024 | v1.1 | `CoverageGapsSummary` Includes Mandatory Caveat Text — L2/Multicast Structural Limitation, Port-102 Collision Ambiguity | Primary: L2 caveat always present, port-102 footnote conditional, tri-state classification, JSON schema |

> **VP Reference Note (Obs-1):** `verification_properties: [VP-041, VP-042, VP-043]` are
> *regression/relevance* references only. VP-041 harnesses are authored and anchored by STORY-151;
> VP-042 and VP-043 harnesses are authored and anchored by STORY-153. STORY-154 exercises these
> VPs indirectly — it consumes the protocol catalog (VP-041 domain), the dispatcher counters
> (VP-042 domain), and the `udp_gap_key` seam (VP-043 domain) that those stories build.
> Future adversarial passes MUST NOT re-flag these as mis-anchoring; VP anchor stories are
> the stories that build and own the harness files.

## Acceptance Criteria

### AC-154-001: `--coverage-gaps` flag added to `analyze` subcommand — NOT in `--all` expansion
**Traces to:** BC-2.12.023 v1.2 PC-1..2, Invariants 1–2, 5; ADR-012 Decision 8

In `src/cli.rs`, the `Analyze` subcommand gains:
```rust
/// Enable per-port unclassified traffic gap detection (opt-in)
#[arg(long)]
pub coverage_gaps: bool,
```

This flag is NOT in the `--all` expansion group (BC-2.12.023 Invariant 1). `wirerust analyze --all`
does NOT imply `--coverage-gaps`. The two flags are independent.
`wirerust protocols --coverage-gaps` is a clap error (flag not valid on `protocols` subcommand).

(traces to BC-2.12.023 v1.2 PC-1..2, Invariants 1, 2, 5; ADR-012 Decision 8)

**Red-Gate tests (integration):**
- `test_BC_2_12_023_all_without_coverage_gaps` — `wirerust analyze test.pcap --all` produces no `CoverageGapsSummary`
- `test_BC_2_12_023_protocols_coverage_gaps_error` — `wirerust protocols --coverage-gaps` exits non-zero (clap error)

### AC-154-002: `--coverage-gaps` wired via `.with_coverage_gaps(...)` builder to `StreamDispatcher` and decode loop
**Traces to:** BC-2.12.023 v1.2 Postcondition 1; BC-2.05.010 v1.3 PC-1; ADR-012 Decision 8

In `src/main.rs`, `run_analyze()` receives `coverage_gaps: bool` as a scalar parameter
(introduced by STORY-153 as default `false`). STORY-154 changes the call-site in `main()` to
pass `*coverage_gaps` destructured from `Commands::Analyze { ..., coverage_gaps, ... }`.

The `StreamDispatcher` is constructed using the builder method from STORY-153:
`StreamDispatcher::new(/* existing 5 analyzer args */).with_coverage_gaps(coverage_gaps)`
at the existing call site (main.rs:306). This leaves all other `StreamDispatcher::new()`
call sites untouched (no blast radius). The decode loop's `if coverage_gaps { ... }` gate
already uses the same scalar parameter — no additional threading required.

When `--coverage-gaps` is set: `unclassified_port_counts` (TCP) and `udp_unclassified_counts`
(UDP) are populated per BC-2.05.010. When NOT set: both maps remain empty.

(traces to BC-2.12.023 v1.2 PC-1; BC-2.05.010 v1.3 PC-1; ADR-012 Decision 8)

**Red-Gate test:**
- `test_BC_2_12_023_coverage_gaps_counts_unclassified` — `wirerust analyze test.pcap --coverage-gaps`
  on a pcap with known unclassified traffic: `CoverageGapsSummary` present with at least 1 entry

### AC-154-003: `CoverageGapsSummary` appended ONLY when `--coverage-gaps` set; `analyze --all` output unchanged
**Traces to:** BC-2.12.023 v1.2 Postconditions 1–2, Invariants 1, 3–4; ADR-012 Decision 8, Decision 9

When `--coverage-gaps` is set:
- A `CoverageGapsSummary` named section is appended to analysis output (terminal and JSON)
- It appears AFTER all `Finding` entries (BC-2.12.023 Invariant 3; ADR-012 Decision 9)
- The existing `Finding` entries and `AnalysisSummary` are UNCHANGED

When `--coverage-gaps` is NOT set (including `analyze --all`):
- No `CoverageGapsSummary` section appears
- Output is IDENTICAL to pre-feature behavior (zero additive changes)
- `analyze --all` output is byte-identical to pre-story behavior

(traces to BC-2.12.023 v1.2 PC-1..2, Invariants 1, 3–4; ADR-012 Decision 8, Decision 9)

**Red-Gate tests:**
- `test_BC_2_12_023_coverage_gaps_flag_produces_section` — `--coverage-gaps` → CoverageGapsSummary present
- `test_BC_2_12_023_no_coverage_gaps_no_section` — no flag → no CoverageGapsSummary section
- `test_BC_2_12_023_all_without_coverage_gaps` — already listed in AC-154-001

### AC-154-004: Mandatory L2/multicast caveat always present in `CoverageGapsSummary`
**Traces to:** BC-2.12.024 v1.1 Postcondition 1, Invariants 1, 3; ADR-012 Decision 3a

The following canonical caveat text (or a semantically identical version) MUST appear in EVERY
`CoverageGapsSummary` output, regardless of how many entries are in the gap report:

```rust
const L2_CAVEAT_TEXT: &str = "Dynamic gap detection covers TCP and UDP flows. \
    Layer-2 protocols (e.g., GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT, Ethernet POWERLINK) \
    have no TCP/UDP port and are not represented in the gap report. \
    Consult `wirerust protocols --unsupported` for L2 protocol coverage.";
```

This constant lives in `src/main.rs` (or a dedicated rendering module). It is NOT configurable
(BC-2.12.024 Invariant 3). It appears even when the entries array is empty (EC-001 in BC-2.12.024).

In JSON mode, it appears as `"coverage_gaps": { "caveat_l2": "...", "entries": [...] }`.

(traces to BC-2.12.024 v1.1 PC-1, Postcondition 1, Invariants 1, 3; ADR-012 Decision 3a)

**Red-Gate tests:**
- `test_BC_2_12_024_l2_caveat_always_present` — `--coverage-gaps` on empty pcap: CoverageGapsSummary rendered; L2 caveat text present; entries empty
- `test_BC_2_12_023_json_coverage_gaps_key` — `--json --coverage-gaps`: JSON has `"coverage_gaps"` key with `"caveat_l2"` non-null string

### AC-154-005: Port-102 collision footnote conditional on TCP/102 non-zero count
**Traces to:** BC-2.12.024 v1.1 Postconditions 2–3, Invariant 2; ADR-012 Decision 3b

```rust
const PORT_102_NOTE: &str = "TCP/102 gap: S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 \
    all share this port (ISO-on-TCP/TPKT). This gap cannot be attributed to a single protocol.";
```

The port-102 collision footnote:
- MUST appear adjacent to the TCP/102 entry when `(Tcp, 102)` has a non-zero count
- MUST NOT suppress the TCP/102 entry; count shown alongside the note
- MUST NOT appear when TCP/102 count == 0 or key is absent (row-specific, not a global header)

In JSON mode, it appears as `"collision_note": "TCP/102 gap: ..."` in the TCP/102 entry object.

(traces to BC-2.12.024 v1.1 PC-2..3, Postconditions 2–3, Invariant 2; ADR-012 Decision 3b)

**Red-Gate tests:**
- `test_BC_2_12_024_port102_footnote_on_tcp102_traffic` — pcap/state with TCP/102 unclassified: footnote present in CoverageGapsSummary
- `test_BC_2_12_024_port102_footnote_absent_without_tcp102` — pcap/state with no TCP/102: footnote absent

> **DF-CANONICAL-FRAME-HOLDOUT-001 — TCP/102 collision:**
> The four protocols sharing TCP/102 (S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2) all
> use ISO-on-TCP/TPKT framing (RFC 1006). The PORT_102_NOTE constant must name all four.
>
> `test_BC_2_12_024_port102_note_names_all_four` — asserts PORT_102_NOTE (or the rendered output)
> contains "S7comm", "S7comm-plus", "IEC 61850 MMS", and "ICCP" (RFC 1006/ISO-on-TCP/TPKT
> framing; all four protocols documented in IEC standards and Siemens industrial protocol specs).

### AC-154-006: Tri-state classification for each gap entry
**Traces to:** BC-2.12.024 v1.1 Postcondition 4, Invariants 4–5; ADR-012 Decision 2

Each entry in `CoverageGapsSummary` is classified using the Suricata-derived vocabulary:
- `known-unsupported` — `(transport, port)` matches a catalog entry that is NOT in `supported_protocols()` (i.e., `canonical_ports ∩ SUPPORTED_PORTS = ∅` and `name ≠ "ARP"`)
- `unknown` — `(transport, port)` matches no catalog entry (completely unrecognized)
- `known-supported` — `(transport, port)` matches a catalog entry in `supported_protocols()` (i.e., `canonical_ports ∩ SUPPORTED_PORTS ≠ ∅` OR `name == "ARP"`)
  (signals a BUG: a dissector failed to classify traffic it should handle)

> **NOTE:** `KnownProtocol` has NO `supported` field. Supportedness is DERIVED:
> a protocol is supported iff `canonical_ports.iter().any(|cp| SUPPORTED_PORTS.contains(cp)) || name == "ARP"`
> (see STORY-151 AC-151-005 / BC-2.18.003; `supported_protocols()` applies exactly this predicate).

The classification function:
```rust
fn lookup_protocol_state(transport: TransportProto, port: u16) -> ProtocolGapState {
    // Transport mapping: TransportProto::Tcp → Transport::Tcp; TransportProto::Udp → Transport::Udp
    // LinkLayer catalog entries NEVER match a port-keyed lookup
    // A port catalogued under UDP only will NOT match a TCP observation (→ unknown)
    // Supportedness is DERIVED (no `supported` field on KnownProtocol): a protocol is
    // supported iff canonical_ports ∩ SUPPORTED_PORTS ≠ ∅ OR name == "ARP" (BC-2.18.003).
    let catalog_transport = match transport { Tcp => Transport::Tcp, Udp => Transport::Udp };
    match KNOWN_PROTOCOLS.iter().find(|p| {
        p.transport == catalog_transport && p.canonical_ports.contains(&port)
    }) {
        Some(p) if p.canonical_ports.iter().any(|cp| SUPPORTED_PORTS.contains(cp))
            || p.name == "ARP" => ProtocolGapState::KnownSupported,  // BUG signal
        Some(_) => ProtocolGapState::KnownUnsupported,
        None => ProtocolGapState::Unknown,
    }
}
```

Key transport-awareness rule: `(Tcp, 47808)` is `unknown` (not `known-unsupported`) because
BACnet/IP is catalogued as `Udp/47808` — transport mismatch (BC-2.12.024 EC-009).
`(Tcp, 53)` is `unknown` — DNS is UDP-only in the catalog (BC-2.12.024 EC-010).

(traces to BC-2.12.024 v1.1 PC-4, Postcondition 4, Invariants 4–5; ADR-012 Decision 2)

**Red-Gate tests:**
- `test_BC_2_12_024_bacnet_known_unsupported` — `(Udp, 47808)` classifies as `known-unsupported` with name "BACnet/IP"
- `test_BC_2_12_024_unknown_port_state` — `(Tcp, 9600)` (no catalog match) classifies as `unknown`
- `test_BC_2_12_024_tcp_47808_is_unknown` — `(Tcp, 47808)` is `unknown` (transport mismatch; BACnet is UDP-only)
- `test_BC_2_12_024_known_supported_is_bug_signal` — `(Tcp, 502)` is `known-supported` (Modbus normally classified; its presence in gap report signals a dissector failure)

> **DF-CANONICAL-FRAME-HOLDOUT-001 — BACnet/IP UDP/47808:**
> BACnet/IP uses UDP port 47808 (0xBAC0) per ASHRAE 135-2016 Annex J §J.2.1. The catalog
> entry for BACnet/IP has `transport: Transport::Udp` and `canonical_ports: &[47808]`.
>
> `test_BC_2_12_024_bacnet_known_unsupported` (cited above) asserts the classification is
> `known-unsupported` for `(Udp, 47808)` — confirming the catalog entry is correct AND
> that the transport-aware lookup correctly matches `Udp` transport to the BACnet/IP entry
> (ASHRAE 135-2016 Annex J §J.2.1; UDP port 0xBAC0 = 47808).

### AC-154-007: JSON representation — `"coverage_gaps"` object schema
**Traces to:** BC-2.12.023 v1.2 Postcondition 3; BC-2.12.024 v1.1 Postcondition 5; ADR-012 Decision 3

In JSON mode (`--json --coverage-gaps`), the output gains a `"coverage_gaps"` key:
```json
"coverage_gaps": {
  "caveat_l2": "Dynamic gap detection covers TCP and UDP flows...",
  "entries": [
    { "transport": "UDP", "port": 47808, "count": 12, "state": "known-unsupported", "name": "BACnet/IP" },
    { "transport": "TCP", "port": 102, "count": 5, "state": "known-unsupported",
      "collision_note": "TCP/102 gap: S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 ..." },
    { "transport": "TCP", "port": 9600, "count": 3, "state": "unknown" }
  ]
}
```

Schema per BC-2.12.024 PC-5:
- `transport`: `"TCP"` or `"UDP"` string
- `port`: integer
- `count`: integer
- `state`: `"known-unsupported"`, `"unknown"`, or `"known-supported"`
- `name`: optional string (only for `known-unsupported` and `known-supported` entries)
- `collision_note`: optional string (only for TCP/102 when count > 0)

(traces to BC-2.12.023 v1.2 PC-3; BC-2.12.024 v1.1 PC-5, Postcondition 5; ADR-012 Decision 3)

**Red-Gate test:**
- `test_BC_2_12_024_json_has_caveat_field` — `--json --coverage-gaps` output: JSON `"coverage_gaps"."caveat_l2"` is non-null string
- `test_BC_2_12_023_json_coverage_gaps_key` — (shared with AC-154-004)

### AC-154-008: Exit code 0; existing Findings and AnalysisSummary unchanged
**Traces to:** BC-2.12.024 v1.1 Postcondition 6; BC-2.12.023 v1.2 Invariant 4

`wirerust analyze --coverage-gaps` exits with code 0 on successful analysis. The
`CoverageGapsSummary` section is purely additive — existing `Finding` entries and
`AnalysisSummary` output are UNCHANGED when `--coverage-gaps` is set.

(traces to BC-2.12.024 v1.1 PC-6; BC-2.12.023 v1.2 Invariant 4)

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| `--coverage-gaps: bool` flag on `Analyze` subcommand | `src/cli.rs` (modify) | Pure (clap type) |
| `coverage_gaps` wiring to `StreamDispatcher::new()` | `src/main.rs` (modify) | Effectful (construction) |
| `coverage_gaps` wiring to UDP decode loop | `src/main.rs` (modify) | Effectful (control flow) |
| `L2_CAVEAT_TEXT: &str` constant | `src/main.rs` (new const) | Pure |
| `PORT_102_NOTE: &str` constant | `src/main.rs` (new const) | Pure |
| `ProtocolGapState` enum `{ KnownUnsupported, Unknown, KnownSupported }` | `src/main.rs` (new type) | Pure |
| `lookup_protocol_state(transport, port)` | `src/main.rs` (new fn) | Pure |
| `render_coverage_gaps_summary(...)` | `src/main.rs` (new fn) | Effectful (stdout) |
| `render_coverage_gaps_summary_json(...)` | `src/main.rs` or `reporter/json.rs` | Effectful (stdout) |
| Integration tests | `tests/integration_tests.rs` (modify) | Effectful (process spawn) |

SS-12 (CLI/Entry) depends on SS-05 counters (from STORY-153) and SS-18 catalog (from STORY-151).
Layer: L0 Entry → L3 domain (catalog lookup) + L3 dispatcher (counter read).

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-154-1 | BC-2.12.023 EC-001 | `analyze` (no --coverage-gaps) | No CoverageGapsSummary; output identical to pre-feature |
| EC-154-2 | BC-2.12.023 EC-002 | `analyze --all` (no --coverage-gaps) | No CoverageGapsSummary; `--all` includes all analyzers but NOT gap detection |
| EC-154-3 | BC-2.12.023 EC-004 | `analyze --all --coverage-gaps` | Both all-analyzers AND gap detection active; CoverageGapsSummary present |
| EC-154-4 | BC-2.12.023 EC-005 | `analyze --coverage-gaps` on empty pcap | CoverageGapsSummary present; entries empty; L2 caveat present |
| EC-154-5 | BC-2.12.023 EC-006 | `analyze --coverage-gaps --json` | JSON output has `"coverage_gaps"` key |
| EC-154-6 | BC-2.12.023 EC-007 | `protocols --coverage-gaps` | clap error (flag not valid on protocols subcommand) |
| EC-154-7 | BC-2.12.024 EC-001 | Empty pcap — no unclassified flows | CoverageGapsSummary; L2 caveat present; entries empty; no port-102 footnote |
| EC-154-8 | BC-2.12.024 EC-002 | UDP/47808 (BACnet/IP) only | Entry: known-unsupported, name=BACnet/IP; L2 caveat; no port-102 footnote |
| EC-154-9 | BC-2.12.024 EC-003 | TCP/102 with non-zero count | Entry: known-unsupported + port-102 collision footnote naming all four protocols |
| EC-154-10 | BC-2.12.024 EC-004 | TCP/9600 (no catalog match) | Entry: unknown; no name |
| EC-154-11 | BC-2.12.024 EC-005 | TCP/502 in gap report (Modbus dissector failed) | Entry: known-supported (BUG signal); entry NOT suppressed |
| EC-154-12 | BC-2.12.024 EC-007 | GOOSE (EtherType 0x88B8) traffic in pcap | GOOSE does NOT appear in gap report (no TCP/UDP port); L2 caveat explains absence |
| EC-154-13 | BC-2.12.024 EC-009 | TCP/47808 (BACnet is UDP-only in catalog) | State: unknown (NOT known-unsupported — transport mismatch) |
| EC-154-14 | BC-2.12.024 EC-010 | TCP/53 (DNS is UDP-only in catalog) | State: unknown (NOT known-supported — DNS is UDP-only) |

## Estimated Complexity

**Story points: 8** (cli.rs flag addition; main.rs flag wiring to both StreamDispatcher and UDP
decode loop; new constants L2_CAVEAT_TEXT + PORT_102_NOTE; new types ProtocolGapState;
`lookup_protocol_state()` pure function; `render_coverage_gaps_summary()` effectful function;
JSON schema per BC-2.12.024 PC-5; integration test suite including canonical-value assertions)

## Token Budget Estimate

| Context source | Estimated tokens |
|---------------|-----------------|
| This story spec | ~2,800 |
| BC-2.12.023 (v1.2) | ~5,000 |
| BC-2.12.024 (v1.1) | ~5,500 |
| ADR-012 (Decisions 2, 3a/3b, 8, 9) | ~6,000 |
| src/cli.rs (existing + STORY-152 Protocols variant) | ~5,000 |
| src/main.rs (full + STORY-153 UDP counter additions) | ~14,000 |
| src/protocols.rs (STORY-151 output — for catalog lookup) | ~6,000 |
| src/dispatcher.rs (STORY-153 output — for counter accessor) | ~12,000 |
| tests/integration_tests.rs | ~8,000 |
| Tool outputs | ~2,500 |
| **Total estimate** | **~66,800** |

Fits within a 200k context window (~33%). This story reads both dispatcher.rs (STORY-153
output) and protocols.rs (STORY-151 output) — load only the sections needed.

## Tasks

1. **Write Red-Gate tests first (TDD Step 1 — all must FAIL before implementation)**
   Add to `tests/integration_tests.rs` in `mod story_154 { ... }`:
   - `test_BC_2_12_023_all_without_coverage_gaps` — `--all` → no CoverageGapsSummary
   - `test_BC_2_12_023_protocols_coverage_gaps_error` — `protocols --coverage-gaps` → clap error
   - `test_BC_2_12_023_coverage_gaps_counts_unclassified` — `--coverage-gaps` → CoverageGapsSummary + ≥1 entry
   - `test_BC_2_12_023_coverage_gaps_flag_produces_section` — flag present → section present
   - `test_BC_2_12_023_no_coverage_gaps_no_section` — no flag → no section
   - `test_BC_2_12_024_l2_caveat_always_present` — empty pcap + --coverage-gaps → L2 caveat in output
   - `test_BC_2_12_024_port102_footnote_on_tcp102_traffic` — TCP/102 → footnote present
   - `test_BC_2_12_024_port102_footnote_absent_without_tcp102` — no TCP/102 → no footnote
   - `test_BC_2_12_024_port102_note_names_all_four` — footnote names all 4 protocols (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_12_024_bacnet_known_unsupported` — (Udp, 47808) → known-unsupported + name=BACnet/IP (DF-CANONICAL-FRAME-HOLDOUT-001)
   - `test_BC_2_12_024_unknown_port_state` — (Tcp, 9600) → unknown
   - `test_BC_2_12_024_tcp_47808_is_unknown` — (Tcp, 47808) → unknown (transport mismatch)
   - `test_BC_2_12_024_known_supported_is_bug_signal` — (Tcp, 502) in gap report → known-supported
   - `test_BC_2_12_024_json_has_caveat_field` — `--json --coverage-gaps` → `"coverage_gaps"."caveat_l2"` non-null
   - `test_BC_2_12_023_json_coverage_gaps_key` — JSON has `"coverage_gaps"` key
   All tests MUST FAIL (`--coverage-gaps` flag doesn't exist yet).

   Add pure-function unit tests in inline `#[cfg(test)] mod story_154_unit { ... }` in `src/main.rs`
   (NOT in `tests/` — `lookup_protocol_state()` is a binary-private function defined in `src/main.rs`
   and is NOT reachable from `tests/integration_tests.rs` or any other `tests/` file):
   - `test_BC_2_12_024_bacnet_known_unsupported_unit` (unit, direct `lookup_protocol_state()` call; `_unit` suffix disambiguates from identically-named integration test in `mod story_154`)
   - `test_BC_2_12_024_tcp_47808_is_unknown_unit` (unit, direct call; `_unit` suffix)
   - `test_BC_2_12_024_unknown_port_state_unit` (unit, direct call; `_unit` suffix)
   - `test_BC_2_12_024_known_supported_is_bug_signal_unit` (unit, direct call; `_unit` suffix)

2. **Add `--coverage-gaps` flag to `Analyze` subcommand in `src/cli.rs` (AC-154-001)**
   - Add `#[arg(long)] pub coverage_gaps: bool` to the `Analyze` struct
   - NOT in `--all` group
   - Verify: clap help shows flag; `protocols --coverage-gaps` yields clap error; binary compiles

3. **Wire `coverage_gaps` to `StreamDispatcher` builder + decode loop (AC-154-002)**
   - In `main()`, change the `run_analyze()` call-site to pass `*coverage_gaps` (from
     `Commands::Analyze { ..., coverage_gaps, ... }` destructure) instead of the STORY-153
     default `false`
   - At the existing `StreamDispatcher::new()` call site (main.rs:306), apply the STORY-153 builder:
     `StreamDispatcher::new(/* existing 5 analyzer args */).with_coverage_gaps(coverage_gaps)`
     (Do NOT change `StreamDispatcher::new()` signature — builder preserves all existing call sites)
   - The decode loop `if coverage_gaps { ... }` gate already uses the scalar parameter (no change)
   - Verify: `test_BC_2_12_023_coverage_gaps_counts_unclassified` turns GREEN (with a test pcap)

4. **Implement `ProtocolGapState` and `lookup_protocol_state()` (AC-154-006)**
   - Define `enum ProtocolGapState { KnownUnsupported, Unknown, KnownSupported }`
   - Implement `fn lookup_protocol_state(transport: TransportProto, port: u16) -> ProtocolGapState`
     using `KNOWN_PROTOCOLS` catalog with transport-aware lookup (no LinkLayer match)
   - Verify: unit tests for tri-state classification all GREEN

5. **Implement `render_coverage_gaps_summary()` for terminal output (AC-154-003 through AC-154-005)**
   - Render `L2_CAVEAT_TEXT` constant always present
   - Render each `(TransportProto, port)` entry with tri-state label
   - Add `PORT_102_NOTE` constant; append adjacent to TCP/102 entry when count > 0
   - Append CoverageGapsSummary AFTER all Finding entries (ADR-012 Decision 9)
   - Call this function in `run_analyze()` after analysis, gated on `coverage_gaps`
   - Verify: terminal integration tests GREEN

6. **Implement JSON CoverageGapsSummary (AC-154-007)**
   - In JSON output path: add `"coverage_gaps"` key with `caveat_l2` + `entries` array
   - Each entry: `transport`, `port`, `count`, `state`, optional `name`, optional `collision_note`
   - Verify: JSON integration tests GREEN

7. **Full regression sweep**
   - `cargo test --all-targets` — ALL tests GREEN (incl. STORY-151/152/153 tests)
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo fmt --check` — clean
   - Verify `analyze` output without `--coverage-gaps` is identical to pre-feature behavior

8. **Micro-commit and PR** targeting `develop` (wave 69, after STORY-152 — file-sequencing edge 152→154)

## Previous Story Intelligence

**From STORY-151 (direct predecessor):**
Provides `KNOWN_PROTOCOLS`, `Transport` enum, `supported_protocols()` / `unsupported_protocols()`.
The tri-state `lookup_protocol_state()` function uses `KNOWN_PROTOCOLS` directly (not the
filter functions) to find a catalog entry by transport+port. Note that `Transport::LinkLayer`
entries NEVER match a port-keyed lookup — they require EtherType matching. The port-keyed
lookup only checks `Transport::Tcp` and `Transport::Udp` entries.

**From STORY-153 (direct predecessor):**
Provides `TransportProto` enum, `unclassified_port_counts()` accessor on `StreamDispatcher`,
`udp_unclassified_counts` map in the decode loop, and the `with_coverage_gaps(enabled: bool) -> Self`
builder method on `StreamDispatcher` (NOT a new `new()` parameter — builder pattern preserves all
existing call sites). Wire via `StreamDispatcher::new(/* existing 5 analyzer args */).with_coverage_gaps(coverage_gaps)`
(using the `coverage_gaps` scalar parameter — NOT `args.coverage_gaps`; `run_analyze()` takes
flat scalar params, not a struct arg).
The transport mapping in `lookup_protocol_state()` must use `TransportProto::Tcp →
protocols::Transport::Tcp` and `TransportProto::Udp → protocols::Transport::Udp`.

**Key lesson from BC-2.12.023 v1.2 (Pass-8 remediation):**
The `"coverage_gaps"` JSON schema is an object form `{ "caveat_l2": "...", "entries": [...] }`,
NOT a flat dict of string keys. Earlier (pre-BC-v1.2) drafts had the wrong schema. Use the
authoritative object form from BC-2.12.023 PC-3 and BC-2.12.024 PC-5.

**From STORY-088/089 (analysis orchestration):**
The `run_analyze()` function in `src/main.rs` already orchestrates the analysis pipeline.
`CoverageGapsSummary` is appended AFTER the existing output, following the same pattern as
`reassembly_summary` (a named section in `AnalysisSummary`). Study `AnalysisSummary` for
the precedent of additive named sections.

## Architecture Compliance Rules

Source: `architecture/module-decomposition.md` + ADR-012 + BC-2.12.023/024

1. **`--coverage-gaps` is NOT in the `--all` expansion group** — BC-2.12.023 Invariant 1; ADR-012 Decision 8. The two flags are independent; `--all` analyzers ≠ `--coverage-gaps`.
2. **`CoverageGapsSummary` is a NAMED SECTION, not a set of `Finding` entries** — ADR-012 Decision 9; BC-2.12.023 Invariant 3. Adding findings for unclassified ports would pollute the MITRE-severity pipeline with infrastructure data.
3. **`CoverageGapsSummary` is PURELY ADDITIVE** — BC-2.12.023 Invariant 4. The existing `Finding` entries and `AnalysisSummary` content are unchanged. Verify with a before/after comparison test.
4. **Tri-state classification is transport-aware** — BC-2.12.024 PC-4. A port catalogued under UDP only will NOT match a TCP observation (→ `unknown`). LinkLayer entries never match port keys.
5. **`L2_CAVEAT_TEXT` is ALWAYS present** — BC-2.12.024 Invariant 1. Even when entries array is empty (empty pcap). Not configurable.
6. **`PORT_102_NOTE` is row-specific and conditional** — BC-2.12.024 Invariant 2. Present IF AND ONLY IF `(Tcp, 102)` has a non-zero count in the current gap report.
7. **JSON schema: object form** — `{ "caveat_l2": "...", "entries": [...] }` (BC-2.12.023 PC-3 v1.2 corrected form). NOT a flat dict.
8. **Test namespace isolation (DF-TEST-NAMESPACE-001):** ALL integration test functions in `mod story_154 { ... }` wrapper in `tests/integration_tests.rs`. Unit tests for `lookup_protocol_state()` MUST be in inline `#[cfg(test)] mod story_154_unit { ... }` in `src/main.rs` — `lookup_protocol_state()` is binary-private (defined in `src/main.rs`, not re-exported from the library crate) and is NOT callable from `tests/`. Test names in `mod story_154_unit` carry a `_unit` suffix (e.g., `test_BC_2_12_024_bacnet_known_unsupported_unit`) to disambiguate from identically-named integration tests in `mod story_154` (DF-AC-TEST-NAME-SYNC-001 unique-resolution).
9. **BACnet/IP classification must use UDP transport lookup** — `(Udp, 47808)` → `known-unsupported`; `(Tcp, 47808)` → `unknown`. This directly tests the transport-aware lookup correctness (DF-CANONICAL-FRAME-HOLDOUT-001 + BC-2.12.024 EC-009).

## Library & Framework Requirements

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `serde_json` | (existing) | JSON CoverageGapsSummary rendering |
| `clap` | (existing) | `--coverage-gaps` flag on Analyze subcommand |

No new crates.

**Forbidden dependencies:** `render_coverage_gaps_summary()` MUST NOT emit `Finding` structs
(ADR-012 Decision 9). Gap entries are a separate output type, not findings.

## File Structure Requirements

| File | Change Type | Purpose |
|------|------------|---------|
| `src/cli.rs` | Modify | Add `coverage_gaps: bool` to `Analyze` subcommand |
| `src/main.rs` | Modify | Wire `coverage_gaps` to `StreamDispatcher::new()` + decode loop; `L2_CAVEAT_TEXT`, `PORT_102_NOTE` constants; `ProtocolGapState` enum; `lookup_protocol_state()` pure fn; `render_coverage_gaps_summary()` effectful fn |
| `tests/integration_tests.rs` | Modify | Add `mod story_154 { ... }` with integration tests per AC-154-001 through AC-154-008 |

No new source files.

## Revision History

| Version | Date | Change | Finding IDs |
|---------|------|--------|-------------|
| v1.0 | 2026-07-02 | Initial story authored for feature-protocol-coverage F3 decomposition | — |
| v1.1 | 2026-07-02 | LOW: Fixed duplicate "3." in Tasks — renumbered second task-3 (ProtocolGapState) to 4, and all subsequent tasks shifted (4→5, 5→6, 6→7, 7→8). | LOW-STORY-154-TASKS |
| v1.2 | 2026-07-02 | F-F3P2-004 cascade (MEDIUM): Fixed AC-154-002 + Task 3 + Previous Story Intelligence (STORY-153 paragraph) to use STORY-153's `with_coverage_gaps(enabled: bool) -> Self` builder method instead of `StreamDispatcher::new()` parameter. F-F3P2-005 (MEDIUM): `wave: 68` → `wave: 69` and `depends_on` updated to include STORY-152 (file-sequencing edge 152→154 added to dep-graph: both STORY-152 and STORY-154 edit src/cli.rs + src/main.rs + tests/integration_tests.rs; placing 154 in wave 69 eliminates the parallel-edit collision). | F-F3P2-004, F-F3P2-005 |
| v1.3 | 2026-07-02 | F-F3P3-001 (HIGH): Fixed AC-154-006 phantom `p.supported` field — `KnownProtocol` has no `supported` field; supportedness is DERIVED. Replaced `Some(p) if p.supported` guard with `Some(p) if p.canonical_ports.iter().any(|cp| SUPPORTED_PORTS.contains(cp)) || p.name == "ARP"` (mirrors `supported_protocols()` predicate per STORY-151 AC-151-005 / BC-2.18.003). Updated vocabulary bullet descriptions to use `supported_protocols()` set membership language instead of stale `supported: true/false` field notation. Added NOTE block clarifying that supportedness is DERIVED, not a struct field. | F-F3P3-001 |
| v1.4 | 2026-07-02 | F-F3P4-002 (MEDIUM): Fixed AC-154-002 heading from forbidden `StreamDispatcher::new(coverage_gaps_enabled=true)` form to `.with_coverage_gaps(...)` builder, consistent with body/Task 3/PSI which already used the correct builder pattern. F-F3P4-003 (MEDIUM): Narrowed unit test location in Task 1 from "in tests/ or inline" to "inline `#[cfg(test)] mod story_154_unit` in `src/main.rs`" (binary-private `lookup_protocol_state()` is NOT callable from `tests/`); added `_unit` suffix to 4 unit test names to eliminate DF-AC-TEST-NAME-SYNC-001 collision with identically-named integration tests; updated Architecture Compliance Rule 8 accordingly. Obs-1: Added VP Reference Note after Behavioral Contracts table clarifying VP-041/042/043 are regression/relevance references (harnesses authored/anchored by STORY-151/153). | F-F3P4-002, F-F3P4-003, Obs-1 |
| v1.5 | 2026-07-02 | F-F3P6-003 (LOW): Replaced phantom empty-parens `StreamDispatcher::new()` with `StreamDispatcher::new(/* existing 5 analyzer args */)` in AC-154-002 body, Task 3, and Previous Story Intelligence (PSI). F-F3P6-005 (LOW): Replaced `args.coverage_gaps` phantom struct-field ref with `coverage_gaps` (flat scalar param) in AC-154-002, Task 3, and PSI. Clarified that STORY-154's wiring change is flipping the STORY-153 default-false call-site value to `*coverage_gaps` from `Commands::Analyze { ..., coverage_gaps, ... }` destructure. | F-F3P6-003, F-F3P6-005 |
