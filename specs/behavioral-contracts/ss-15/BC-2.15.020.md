---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.3: F3 story-anchor back-fill. — 2026-06-14"
  - "v1.4: fix-pc-013-014-015 PC-014 BREAKING JSON output change (human-approved D-220) — Postcondition 1 key renamed: `total_parse_errors` → `parse_errors` to align with sibling analyzers (HTTP: http.rs:617, TLS: tls.rs:884, Modbus: modbus.rs:947 all use `parse_errors`). This is a breaking change for callers reading `total_parse_errors` from DNP3 JSON output. A CHANGELOG entry and minor-version bump at release are required. Test vectors and EC table updated to use `parse_errors`. Red Gate test: `test_BC_2_15_020_parse_errors_key_name_is_parse_errors` MUST assert `detail.contains_key(\"parse_errors\") == true` AND `detail.contains_key(\"total_parse_errors\") == false`. — 2026-06-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.020: summarize() Emits Function-Code Distribution and Control-Operation Counts

## Description

`Dnp3Analyzer::summarize()` (or equivalent finalization method called in `finalize()` /
`on_flow_close()`) produces aggregate statistics across all analyzed DNP3 flows: the
function-code distribution across all flows (`self.fn_code_counts: HashMap<u8, u64>`) and
the per-flow control-operation counts. These statistics are included in the JSON output to
support post-analysis investigation. This implements issue #8's acceptance criterion: "Per
issue #8 AC: function-code distribution + control-operation counts in summarize()."

## Preconditions

1. `Dnp3Analyzer::finalize()` (or `summarize()`) is called after all PCAP frames have been
   processed.
2. `self.fn_code_counts` has been populated by all `on_data` calls.
3. `self.flows` may be empty (no DNP3 flows found) or non-empty.

## Postconditions

1. The JSON output includes a `dnp3_summary` object (or equivalent structure) containing:
   - `function_code_distribution`: a map of FC byte (hex string or integer) to occurrence count,
     drawn from `self.fn_code_counts`. Only FCs with count > 0 are included.
   - `control_operation_counts`: for each flow, the total number of Control-class FC observations
     (`direct_operate_count` field from `Dnp3FlowState`, or an equivalent aggregate count).
   - `total_frames`: sum of `flow.frame_count` across all flows.
   - `parse_errors`: sum of `flow.parse_errors` across all flows.
     **BREAKING CHANGE (D-220, human-approved):** renamed from `total_parse_errors` (the
     old name used in `dnp3.rs:1425` and `dnp3_detection_tests.rs:995/1378`). Aligns with
     sibling analyzers: HTTP (`http.rs:617`), TLS (`tls.rs:884`), Modbus (`modbus.rs:947`)
     all use `"parse_errors"`. Code sites to update: `src/analyzer/dnp3.rs:1425` (key
     insert), `tests/dnp3_detection_tests.rs:995` (`.get("total_parse_errors")`),
     `tests/dnp3_detection_tests.rs:1378` (`.contains_key("total_parse_errors")`). Check
     `tests/bc_2_15_110_dnp3_dispatcher_tests.rs:959` — if a local variable assignment only,
     no update required; if a key string lookup, update to `"parse_errors"`. Requires
     CHANGELOG entry and minor-version bump at release.
   - `flows_analyzed`: count of distinct TCP flows processed by `Dnp3Analyzer`.
2. If no DNP3 flows were analyzed, the summary is still present with zero counts (not absent).
3. The summary is produced even if no findings were emitted.

## Invariants

1. **Issue #8 AC**: the presence of `function_code_distribution` and `control_operation_counts`
   in the output is a hard acceptance criterion for issue #8. These fields must be non-absent in
   any run that processes DNP3 flows.
2. **Consistency**: `fn_code_counts[fc]` equals the total number of times FC `fc` was observed
   as an application function code across ALL flows processed in this analyzer instance.
3. **Aggregate only**: `summarize()` does not emit new findings; it only produces statistics.
   Any T1692.001/T0814/T0836/T1691.001/T0827 findings were already pushed during `on_data`.
4. **Zero-flow case**: if `self.flows.is_empty()`, `flows_analyzed = 0`, `total_frames = 0`,
   all distribution maps are empty. Output is still valid JSON.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No DNP3 traffic in PCAP | Summary present with zero counts; `flows_analyzed=0` |
| EC-002 | Only READ (0x01) traffic | `fn_code_counts = {0x01: N}`; `control_operation_counts = {}`; no T1692.001 |
| EC-003 | Multiple flows with overlapping FCs | `fn_code_counts` aggregates ALL flows; `control_operation_counts` is per-flow |
| EC-004 | Flow with is_non_dnp3=true | That flow's frames are NOT counted in fn_code_counts (no app-layer parsing occurred) |

## Canonical Test Vectors

| PCAP content | Expected `dnp3_summary` content |
|-------------|--------------------------------|
| 5 DIRECT_OPERATE frames on one flow | `{fn_code_counts:{0x05:5}, control_op_counts:{flow1:5}, total_frames:5}` |
| 3 READ + 2 COLD_RESTART on one flow | `{fn_code_counts:{0x01:3, 0x0D:2}, total_frames:5}` |
| No DNP3 traffic | `{fn_code_counts:{}, total_frames:0, flows_analyzed:0}` |
| 1 flow with 10 frames and 2 parse errors (Red Gate — key name) | `detail` map MUST contain `"parse_errors": 2` and MUST NOT contain `"total_parse_errors"`; test: `assert!(detail.contains_key("parse_errors")); assert!(!detail.contains_key("total_parse_errors"))` |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Aggregation logic: effectful shell; unit + integration test | unit test, integration test (PCAP acceptance test) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — function-code distribution and control-operation counts are an explicit acceptance criterion for the DNP3/ICS analyzer capability (issue #8 AC), providing operators with situational awareness data beyond individual findings |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — summary stats only cover flows actually routed to the DNP3 analyzer) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24 `finalize()`); ADR-007 Decision 2 |
| Stories | STORY-108 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — statistics/summary BC; no finding emission) |

## Related BCs

- BC-2.15.016 — depends on (per-flow frame_count and parse_errors collected during carry-buffer processing)
- BC-2.15.010 — composes with (direct_operate_count is one of the control_operation_counts fields)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::finalize()` or `summarize()`
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.fn_code_counts: HashMap<u8, u64>`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.frame_count: u64`, `.parse_errors: u64`, `.direct_operate_count: u32`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §2.2–2.3` — struct fields
- GitHub issue #8 AC: "function-code distribution + control-operation counts in summarize()"

## Story Anchor

STORY-108

## VP Anchors

(none — statistics aggregation; no formal proof target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | GitHub issue #8 AC (direct requirement); dnp3-architecture-delta.md §2.2 (fn_code_counts field) |
| **Confidence** | high — explicit acceptance criterion from issue #8 |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads self.flows (read-only) |
| **Global state access** | reads self.fn_code_counts |
| **Deterministic** | yes — same flows always produce same statistics |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (reads shared state; produces JSON output) |
