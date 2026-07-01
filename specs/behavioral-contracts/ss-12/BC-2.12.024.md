---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-01T18:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-12
capability: CAP-12
lifecycle_status: active
introduced: feature-protocol-coverage-F2
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.12.024: `CoverageGapsSummary` Includes Mandatory Caveat Text — L2/Multicast Structural Limitation, Port-102 Collision Ambiguity

## Description

Whenever `CoverageGapsSummary` is rendered (terminal or JSON), it MUST include fixed
mandatory caveat text. Two categories of caveats are required: (1) the structural L2/multicast
limitation (GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT are not present in the gap
report because they have no TCP/UDP port), and (2) the port-102 four-way TCP collision
ambiguity (a gap on TCP/102 cannot be attributed to a single protocol). These caveats are
NOT optional warnings; they are required disclosures that prevent operators from making
incorrect inferences about protocol coverage from the gap report alone. The caveat text
is fixed (not configurable by CLI flags).

## Related BCs

- BC-2.12.023 — depends on (`--coverage-gaps` flag must be set for this output to appear; BC-2.12.023 covers the flag mechanics)
- BC-2.05.010 — depends on (the counters that populate the gap report; L2 protocols are structurally absent because the TCP dispatcher and UDP decode loop only see TCP/UDP traffic)
- BC-2.18.001 — composes with (the static `protocols --unsupported` command lets operators discover L2 protocols that are absent from the dynamic gap report)

## Preconditions

1. `--coverage-gaps` is set on the `analyze` subcommand.
2. `CoverageGapsSummary` is being rendered (terminal or JSON).

## Postconditions

1. **L2/multicast structural limitation caveat MUST appear.** The exact canonical text is:
   > "Dynamic gap detection covers TCP and UDP flows. Layer-2 protocols (GOOSE, Sampled Values, PROFINET-RT/DCP, EtherCAT) have no TCP/UDP port and are not represented in the gap report. Consult `wirerust protocols --unsupported` for L2 protocol coverage."
   
   This text (or a semantically identical localization of it) MUST be present in terminal output and in JSON output as a `"caveat_l2"` (or equivalent) field.

2. **Port-102 collision caveat MUST appear when `(Tcp, 102)` has a non-zero count.** The exact canonical text is:
   > "TCP/102 gap: S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 all share this port (ISO-on-TCP/TPKT). This gap cannot be attributed to a single protocol."
   
   This inline annotation MUST appear adjacent to the TCP/102 entry in the gap report. It MUST NOT suppress the TCP/102 entry; the count is shown alongside the ambiguity note.

3. **Port-102 caveat when TCP/102 count is zero or absent:** The port-102 collision footnote does NOT need to appear when there are no TCP/102 unclassified flows (the caveat is row-specific, not a global header). If TCP/102 is not in the gap report (count == 0 or key absent), the footnote is omitted.

4. **Tri-state classification in gap report:** Each entry in the gap report MUST be classified using the Suricata-derived vocabulary:
   - `known-unsupported` — the port matches a catalog entry with `supported: false` (e.g., BACnet/IP 47808)
   - `unknown` — the port matches no catalog entry (completely unrecognized port)
   - `known-supported` — the port matches a catalog entry with `supported: true` (should never appear in the gap report; present as a sanity-check signal if it does)
   
   The classification is determined by looking up the `(TransportProto, port)` key against `KNOWN_PROTOCOLS`.

5. **JSON representation:** In JSON mode, caveats appear as structured fields in the `"coverage_gaps"` object, not as free-text. Example:
   ```json
   "coverage_gaps": {
     "caveat_l2": "Dynamic gap detection covers TCP and UDP flows...",
     "entries": [
       { "transport": "UDP", "port": 47808, "count": 12, "state": "known-unsupported", "name": "BACnet/IP" },
       { "transport": "TCP", "port": 102, "count": 5, "state": "known-unsupported",
         "collision_note": "TCP/102 gap: S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 all share this port." },
       { "transport": "TCP", "port": 9600, "count": 3, "state": "unknown" }
     ]
   }
   ```

6. **Exit code:** 0.

## Invariants

1. The L2/multicast caveat is ALWAYS present in `CoverageGapsSummary` when rendered, regardless of whether any L2 protocols are in the catalog. It is a fixed structural disclaimer, not a conditional warning.
2. The port-102 collision footnote is row-specific: it appears IF AND ONLY IF TCP/102 has a non-zero count in the gap report.
3. Caveat text is NOT configurable. No CLI flag suppresses or alters these caveats.
4. The tri-state classification is deterministic: same port + same catalog → same state label, always.
5. The `known-supported` state in the gap report indicates a BUG (a supported protocol appeared as an unclassified flow). Its presence does NOT suppress the entry; instead, it signals to the operator that a dissector may have failed to classify traffic it should have handled.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty pcap — no unclassified flows | `CoverageGapsSummary` still rendered; L2 caveat present; no port-102 footnote (no TCP/102 traffic); entries array empty |
| EC-002 | UDP/47808 (BACnet/IP) only | Entry: `(Udp, 47808)`, state=known-unsupported, name=BACnet/IP; L2 caveat present; no port-102 footnote |
| EC-003 | TCP/102 with non-zero count | Entry: `(Tcp, 102)`, state=known-unsupported, port-102 collision footnote present; four protocols named |
| EC-004 | TCP/9600 (no catalog match) | Entry: `(Tcp, 9600)`, state=unknown; no name |
| EC-005 | TCP/502 in gap report (unusual — Modbus is normally classified; this means Modbus dissector failed) | Entry: `(Tcp, 502)`, state=known-supported (BUG signal); entry NOT suppressed; count shown |
| EC-006 | Multiple unclassified ports including TCP/102 and UDP/47808 | Both entries present; TCP/102 has collision footnote; UDP/47808 is classified as known-unsupported; L2 caveat present |
| EC-007 | GOOSE (EtherType 0x88B8) traffic in the pcap | GOOSE does NOT appear in the gap report (no TCP/UDP port); L2 caveat explains this absence |
| EC-008 | `--json --coverage-gaps` | JSON output has structured `caveat_l2` field and `entries` array; port-102 collision note in relevant entry as `collision_note` field |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `--coverage-gaps` on pcap with UDP/47808 | CoverageGapsSummary has entry `{transport:UDP, port:47808, state:known-unsupported, name:BACnet/IP}`; L2 caveat present | known-unsupported |
| `--coverage-gaps` on pcap with TCP/102 | CoverageGapsSummary has entry `{transport:TCP, port:102}` + port-102 collision footnote | port-102-collision |
| `--coverage-gaps` on pcap with TCP/9600 | Entry `{transport:TCP, port:9600, state:unknown}`; L2 caveat present | unknown-port |
| `--coverage-gaps` on empty pcap | CoverageGapsSummary rendered; L2 caveat present; entries empty | empty-pcap |
| `--json --coverage-gaps` on any pcap | JSON `"coverage_gaps"` has `"caveat_l2"` field (non-null string) | json-caveat |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | L2 caveat always present in CoverageGapsSummary (including empty pcap) | integration: `test_BC_2_12_024_l2_caveat_always_present` |
| — | Port-102 footnote present when TCP/102 count > 0 | integration: `test_BC_2_12_024_port102_footnote_on_tcp102_traffic` |
| — | Port-102 footnote absent when TCP/102 count == 0 | integration: `test_BC_2_12_024_port102_footnote_absent_without_tcp102` |
| — | BACnet/IP UDP/47808 classified as known-unsupported | integration: `test_BC_2_12_024_bacnet_known_unsupported` |
| — | Unknown port classified as unknown | integration: `test_BC_2_12_024_unknown_port_state` |
| — | JSON output has `caveat_l2` field | integration: `test_BC_2_12_024_json_has_caveat_field` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md |
| Capability Anchor Justification | CAP-12 ("CLI Orchestration / Entry Point") per domain/capabilities/cap-12-cli-orchestration.md — `CoverageGapsSummary` caveat text is part of the analysis output rendering layer, which is orchestrated by the CLI entry point; the mandatory caveat obligation is an output-contract of the `analyze` subcommand when `--coverage-gaps` is set |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-12 (src/main.rs — `CoverageGapsSummary` rendering function; caveat text constants); SS-18 (src/protocols.rs — catalog lookup for tri-state classification) |
| ADR | ADR-012 Decision 2 (Suricata tri-state vocabulary), Decision 3 (mandatory caveats — 3a L2/multicast, 3b port-102 collision, 3d heuristic disclaimer), Decision 3a (exact canonical L2 caveat text), Decision 9 (CoverageGapsSummary as named section) |
| Stories | TBD (F3 story decomposition) |

## Architecture Anchors

- `src/main.rs` — `render_coverage_gaps_summary(counts: &HashMap<(TransportProto, u16), u64>, json: bool)` (or equivalent function); contains:
  - `L2_CAVEAT_TEXT: &str` constant (canonical L2/multicast caveat text)
  - `PORT_102_NOTE: &str` constant (canonical port-102 collision footnote)
  - Tri-state classification logic: `lookup_protocol_state(transport, port) -> ProtocolGapState { KnownUnsupported, Unknown, KnownSupported }`; calls `KNOWN_PROTOCOLS` lookup
- `src/protocols.rs` — `KNOWN_PROTOCOLS` used by tri-state classification
- `tests/integration_tests.rs` (or equivalent) — integration test suite for all Canonical Test Vectors above

## Story Anchor

TBD (F3 story decomposition for feature-protocol-coverage)

## VP Anchors

(None assigned yet — integration tests serve as verification; caveat text is deterministic and does not require proptest)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | stdout write (rendering is effectful) |
| **Global state access** | read-only (`KNOWN_PROTOCOLS`, caveat text constants) |
| **Deterministic** | yes (same counters + same catalog → same output) |
| **Thread safety** | yes |
| **Overall classification** | effectful (rendering/stdout); pure (tri-state classification and caveat text lookup) |
