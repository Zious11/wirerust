---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.4: fix-pc-013-014-015 PC-014 BREAKING JSON output change (human-approved D-220) — Postcondition 1 key renamed: `total_parse_errors` → `parse_errors` to align with sibling analyzers (`HttpAnalyzer::summarize`, `TlsAnalyzer::summarize`, `ModbusAnalyzer::summarize` all use `parse_errors`). This is a breaking change for callers reading `total_parse_errors` from DNP3 JSON output. A CHANGELOG entry and minor-version bump at release are required. Test vectors and EC table updated to use `parse_errors`. Red Gate test: `test_BC_2_15_020_parse_errors_key_name_is_parse_errors` MUST assert `detail.contains_key(\"parse_errors\") == true` AND `detail.contains_key(\"total_parse_errors\") == false`. — 2026-06-23"
  - "v1.5 (2026-07-06): silent-limit audit — add three observability counters as keys 6, 7, 8 in the detail map: `dropped_findings` (u64, count of findings suppressed by MAX_FINDINGS cap — BC-2.15.022 PC-5), `master_addrs_dropped` (u64, count of new master addresses silently ignored at MAX_MASTER_ADDRS=64 cap — BC-2.15.016 PC-6), `pending_requests_evicted` (u64, count of LRU evictions from pending_requests at MAX_PENDING_REQUESTS=256 — BC-2.15.016 PC-10); all three ALWAYS present even when 0; Invariant 1 updated to enumerate EIGHT authoritative keys; Invariant 5 added (counter semantics — COUNTERS ONLY, no Finding); EC-001 updated; EC-005, EC-006, EC-007 added; canonical test vectors extended; Architecture Anchors for three new fields added; Related BCs updated (BC-2.15.022 + BC-2.15.016 as counter-source dependencies)"
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
input-hash: "08fc164"
---

# BC-2.15.020: summarize() Emits Function-Code Distribution and Control-Operation Counts

## Description

`Dnp3Analyzer::summarize()` (or equivalent finalization method called in `finalize()` /
`on_flow_close()`) produces aggregate statistics across all analyzed DNP3 flows: the
function-code distribution across all flows (`self.fn_code_counts: HashMap<u8, u64>`) and
the per-flow control-operation counts. These statistics are included in the JSON output to
support post-analysis investigation. This implements issue #8's acceptance criterion: "Per
issue #8 AC: function-code distribution + control-operation counts in summarize()."
Three observability counters (`dropped_findings`, `master_addrs_dropped`,
`pending_requests_evicted`) were added in v1.5 (silent-limit audit) to surface cap-pressure
events that were previously silent, achieving parity with the v0.11.4 counter pattern
established by ARP (`bindings_evicted`, `storm_counters_evicted`), Modbus
(`dropped_findings`, `dropped_transactions`), HTTP/TLS (`dropped_map_entries`), and
EtherNet/IP (`dropped_findings`).

## Preconditions

1. `Dnp3Analyzer::finalize()` (or `summarize()`) is called after all PCAP frames have been
   processed.
2. `self.fn_code_counts` has been populated by all `on_data` calls.
3. `self.flows` may be empty (no DNP3 flows found) or non-empty.

## Postconditions

1. The JSON output includes a `dnp3_summary` object (or equivalent structure) containing
   the following EIGHT keys (the complete and authoritative set for v1.5; none may be
   omitted):
   - `function_code_distribution`: a map of FC byte (hex string or integer) to occurrence count,
     drawn from `self.fn_code_counts`. Only FCs with count > 0 are included.
   - `control_operation_counts`: for each flow, the total number of Control-class FC observations
     (`direct_operate_count` field from `Dnp3FlowState`, or an equivalent aggregate count).
   - `total_frames`: sum of `flow.frame_count` across all flows.
   - `parse_errors`: sum of `flow.parse_errors` across all flows.
     **BREAKING CHANGE (D-220, human-approved):** renamed from `total_parse_errors` (old name
     used in `Dnp3Analyzer::summarize` key-insert site and in
     `test_BC_2_15_020_parse_errors_key_name_is_parse_errors`). Aligns with sibling analyzers:
     `HttpAnalyzer::summarize`, `TlsAnalyzer::summarize`, `ModbusAnalyzer::summarize` all use
     `"parse_errors"`. Code sites to update: `Dnp3Analyzer::summarize` (key insert),
     `test_BC_2_15_020_parse_errors_key_name_is_parse_errors` (assertion). Requires CHANGELOG
     entry and minor-version bump at release.
   - `flows_analyzed`: count of distinct TCP flows processed by `Dnp3Analyzer`.
   - `dropped_findings`: count of `Finding` objects suppressed due to the `MAX_FINDINGS =
     10_000` cap (BC-2.15.022 PC-5); sourced from `Dnp3Analyzer.dropped_findings: u64`.
     ALWAYS present, even when 0. No Finding is emitted for drops.
   - `master_addrs_dropped`: count of new master source addresses silently ignored because
     `flow.master_addrs_seen.len() == MAX_MASTER_ADDRS = 64` (BC-2.15.016 PC-6); sourced
     from `Dnp3Analyzer.master_addrs_dropped: u64`. ALWAYS present, even when 0. No Finding
     is emitted for drops.
   - `pending_requests_evicted`: count of LRU evictions from `flow.pending_requests` when
     `flow.pending_requests.len() == MAX_PENDING_REQUESTS = 256` (BC-2.15.016 PC-10); sourced
     from `Dnp3Analyzer.pending_requests_evicted: u64`. ALWAYS present, even when 0. No
     T1691.001 timeout-event is generated for evicted entries.
2. If no DNP3 flows were analyzed, the summary is still present with zero counts (not absent).
3. The summary is produced even if no findings were emitted.

## Invariants

1. **Key name exactness — EIGHT keys (authoritative for all downstream consumers)**:
   `function_code_distribution`, `control_operation_counts`, `total_frames`, `parse_errors`,
   `flows_analyzed`, `dropped_findings`, `master_addrs_dropped`, `pending_requests_evicted`
   are the complete and authoritative set of DNP3 summary keys for v1.5. None may be omitted.
   `dropped_findings`, `master_addrs_dropped`, and `pending_requests_evicted` MUST always be
   present (value 0 when the respective cap was never reached). Any additional future keys
   must be added via a new BC revision.
2. **Consistency**: `fn_code_counts[fc]` equals the total number of times FC `fc` was observed
   as an application function code across ALL flows processed in this analyzer instance.
3. **Aggregate only**: `summarize()` does not emit new findings; it only produces statistics.
   Any T1692.001/T0814/T0836/T1691.001/T0827 findings were already pushed during `on_data`.
4. **Zero-flow case**: if `self.flows.is_empty()`, `flows_analyzed = 0`, `total_frames = 0`,
   all distribution maps are empty, all three counter keys are 0. Output is still valid JSON.
5. **Counter semantics — COUNTERS ONLY, no Finding emitted**: `dropped_findings`,
   `master_addrs_dropped`, and `pending_requests_evicted` are aggregate counters across ALL
   flows handled by this `Dnp3Analyzer` instance. They are monotonically non-decreasing
   across the lifetime of the analyzer. They exist solely to make previously-silent cap
   events observable in `summarize()` output. No anomaly Finding, no warning, and no log
   entry is emitted for the drops/evictions they count.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No DNP3 traffic in PCAP | Summary present with zero counts; `flows_analyzed=0`; all three counter keys present with value 0 |
| EC-002 | Only READ (0x01) traffic | `fn_code_counts = {0x01: N}`; `control_operation_counts = {}`; no T1692.001; all three counters = 0 |
| EC-003 | Multiple flows with overlapping FCs | `fn_code_counts` aggregates ALL flows; `control_operation_counts` is per-flow |
| EC-004 | Flow with is_non_dnp3=true | That flow's frames are NOT counted in fn_code_counts (no app-layer parsing occurred) |
| EC-005 | MAX_FINDINGS cap hit mid-capture (50 findings suppressed) | `dropped_findings = 50`; `total_frames` still reflects actual traffic; `master_addrs_dropped = 0`; `pending_requests_evicted = 0` (unless those caps also triggered) |
| EC-006 | master_addrs_seen cap reached 64 in one flow; 10 additional master addresses arrive | `master_addrs_dropped = 10`; `dropped_findings = 0`; `pending_requests_evicted = 0` (unless those caps also triggered) |
| EC-007 | pending_requests LRU eviction fires N times across all flows | `pending_requests_evicted = N`; `dropped_findings = 0`; `master_addrs_dropped = 0` (unless those caps also triggered) |

## Canonical Test Vectors

| PCAP content | Expected `dnp3_summary` content |
|-------------|--------------------------------|
| 5 DIRECT_OPERATE frames on one flow | `{fn_code_counts:{0x05:5}, control_op_counts:{flow1:5}, total_frames:5, dropped_findings:0, master_addrs_dropped:0, pending_requests_evicted:0}` |
| 3 READ + 2 COLD_RESTART on one flow | `{fn_code_counts:{0x01:3, 0x0D:2}, total_frames:5, dropped_findings:0, master_addrs_dropped:0, pending_requests_evicted:0}` |
| No DNP3 traffic | `{fn_code_counts:{}, total_frames:0, flows_analyzed:0, dropped_findings:0, master_addrs_dropped:0, pending_requests_evicted:0}` |
| 1 flow with 10 frames and 2 parse errors (Red Gate — key name) | `detail` map MUST contain `"parse_errors": 2` and MUST NOT contain `"total_parse_errors"`; test: `assert!(detail.contains_key("parse_errors")); assert!(!detail.contains_key("total_parse_errors"))` |
| MAX_FINDINGS cap hit; 7 findings suppressed | `dropped_findings: 7`; `master_addrs_dropped: 0`; `pending_requests_evicted: 0` |
| 65 distinct master addresses in one flow | `master_addrs_dropped: 1` (the 65th was silently ignored); `dropped_findings: 0`; `pending_requests_evicted: 0` |
| 500 unanswered Control requests in one flow (pending table saturates at 256) | `pending_requests_evicted: 244` (requests 257–500 each trigger one eviction); `dropped_findings: 0`; `master_addrs_dropped: 0` |

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

- BC-2.15.016 — depends on (per-flow frame_count and parse_errors collected during carry-buffer processing; master_addrs_dropped sourced from BC-2.15.016 PC-6; pending_requests_evicted sourced from BC-2.15.016 PC-10)
- BC-2.15.022 — depends on (dropped_findings counter sourced from BC-2.15.022 PC-5 — MAX_FINDINGS cap suppressed-finding counter)
- BC-2.15.010 — composes with (direct_operate_count is one of the control_operation_counts fields)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::finalize()` or `summarize()`
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.fn_code_counts: HashMap<u8, u64>`
- `src/analyzer/dnp3.rs` — `Dnp3FlowState.frame_count: u64`, `.parse_errors: u64`, `.direct_operate_count: u32`
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.dropped_findings: u64` (aggregate counter; incremented by 1 on each MAX_FINDINGS-cap suppression per BC-2.15.022 PC-5; surfaced as key `"dropped_findings"`)
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.master_addrs_dropped: u64` (aggregate counter; incremented by 1 on each MAX_MASTER_ADDRS-cap silent-ignore per BC-2.15.016 PC-6; surfaced as key `"master_addrs_dropped"`)
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.pending_requests_evicted: u64` (aggregate counter; incremented by 1 on each MAX_PENDING_REQUESTS-cap LRU eviction per BC-2.15.016 PC-10; surfaced as key `"pending_requests_evicted"`)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §2.2–2.3` — struct fields
- GitHub issue #8 AC: "function-code distribution + control-operation counts in summarize()"

## Story Anchor

STORY-108

## VP Anchors

(none — statistics aggregation; no formal proof target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | GitHub issue #8 AC (direct requirement); dnp3-architecture-delta.md §2.2 (fn_code_counts field); v1.5 silent-limit audit (PC-003, PC-016, PC-017 confirmed in df-validation-pc019-pc020-2026-07-06.md) |
| **Confidence** | high — explicit acceptance criterion from issue #8; counter gaps confirmed by validation research |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads self.flows (read-only) |
| **Global state access** | reads self.fn_code_counts, self.dropped_findings, self.master_addrs_dropped, self.pending_requests_evicted |
| **Deterministic** | yes — same flows always produce same statistics |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (reads shared state; produces JSON output) |
