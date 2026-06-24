---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.021: summarize() Emits ENIP Command Distribution and Aggregate Statistics

## Description

`EnipAnalyzer::summarize()` (or equivalent `finalize()` method) produces aggregate statistics
across all analyzed EtherNet/IP flows: command distribution (`command_distribution: HashMap<u16, u64>`),
total PDU count, total parse errors (lifetime), write count, error count, and flows analyzed.
These statistics are included in the JSON output. The summary is always present even if no
ENIP flows were analyzed. No new findings are emitted by `summarize()`; findings were already
accumulated during `on_data` processing.

## Preconditions

1. `EnipAnalyzer::summarize()` is called after all PCAP frames have been processed.
2. `self.command_distribution` has been populated by `on_flow_close` calls.
3. `self.all_findings` may contain 0 to `MAX_FINDINGS` findings.
4. `self.flows` may still contain open flows (not yet closed) — their state may optionally
   be folded in at summarize time.

## Postconditions

1. The JSON output includes an `enip_summary` object containing:
   - `command_distribution`: map of ENIP command u16 to occurrence count (only non-zero counts).
   - `total_pdu_count`: total PDUs processed across all ENIP flows.
   - `parse_errors`: total structural errors across all flows (lifetime counter aggregate).
   - `write_count`: total CIP write-class service requests across all flows.
   - `error_count`: total CIP error responses across all flows.
   - `flows_analyzed`: count of distinct TCP flows processed.
   - `dropped_findings`: count of findings dropped due to MAX_FINDINGS cap.
2. If no ENIP flows analyzed: all counts are 0; output is still valid JSON (not absent).
3. `summarize()` does not emit new findings.
4. The `parse_errors` key in the JSON uses the canonical name `"parse_errors"` (not
   `"total_parse_errors"`) — consistent with sibling analyzers (modbus, dnp3, http, tls).

## Invariants

1. **parse_errors key naming**: must be `"parse_errors"` (not `"total_parse_errors"`).
   This is a breaking-change guard — the first implementation must use the canonical name
   to avoid a rename fix later (per BC-2.15.020 v1.4 D-220 lesson).
2. **Aggregate only**: `summarize()` reads `self.command_distribution`, `self.total_pdu_count`,
   etc. It does NOT re-scan flow state. Aggregate counters must be up-to-date from
   `on_flow_close` calls before `summarize()` is invoked.
3. **Zero-flow case**: if no ENIP flows, all fields are 0. The `enip_summary` object is
   still present in JSON (not absent, not null).
4. **dropped_findings**: `self.dropped_findings` is incremented inside `on_data` whenever a
   finding would be pushed but `all_findings.len() >= MAX_FINDINGS`. Reported here for
   operator awareness.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No ENIP traffic in PCAP | `enip_summary` present with all-zero counts |
| EC-002 | 5 ListIdentity + 2 SendRRData flows | `command_distribution = {0x0063: 5, 0x006F: 2}` |
| EC-003 | MAX_FINDINGS reached; `dropped_findings = N` | `enip_summary.dropped_findings = N` |
| EC-004 | parse_errors = 3 (lifetime) | `enip_summary.parse_errors = 3` |

## Canonical Test Vectors

| PCAP content | Expected `enip_summary` key fields |
|-------------|----------------------------------|
| 3 ListIdentity frames (1 flow) | `{command_distribution:{0x0063:3}, total_pdu_count:3, parse_errors:0, flows_analyzed:1}` |
| No ENIP traffic | `{command_distribution:{}, total_pdu_count:0, parse_errors:0, flows_analyzed:0}` |
| 1 flow with 5 PDUs, 2 parse errors | `{total_pdu_count:5, parse_errors:2}` |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Aggregation, key naming, zero-flow case: effectful shell; unit + integration test | unit test, integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — ENIP command distribution and aggregate statistics are required in `summarize()` for operator situational awareness and for the JSON output schema; the zero-flow case must be handled gracefully |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (EnipAnalyzer aggregate fields) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — statistics BC; no finding emission) |

## Related BCs

- BC-2.17.017 — depends on (on_flow_close updates aggregate counters consumed here)
- BC-2.17.022 — composes with (dropped_findings counter reported in summary)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipAnalyzer::summarize()` or `finalize()`
- `src/analyzer/enip.rs` — `EnipAnalyzer.command_distribution: HashMap<u16, u64>`
- `src/analyzer/enip.rs` — `EnipAnalyzer.total_pdu_count: u64`, `.write_count: u64`, `.error_count: u64`, `.parse_errors: u64`, `.dropped_findings: u64`

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — statistics aggregation; no formal proof target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (EnipAnalyzer aggregate fields); architecture-delta.md §4.2 (struct fields); BC-2.15.020 (DNP3 precedent: parse_errors key naming lesson) |
| **Confidence** | high — mirrors established DNP3/Modbus summarize() pattern |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads self state (read-only) |
| **Global state access** | reads all aggregate counters |
| **Deterministic** | yes — same state always produces same summary |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (reads shared state; produces output) |
