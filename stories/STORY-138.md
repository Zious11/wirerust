---
document_type: story
story_id: STORY-138
title: "ENIP Session Lifecycle, Statistics, DoS Guard, and Analyzer Summary"
epic_id: E-20
wave: 61
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-17]
target_module: analyzer/enip
depends_on: [STORY-134, STORY-135, STORY-136, STORY-137]
behavioral_contracts:
  - BC-2.17.025
  - BC-2.17.017
  - BC-2.17.021
  - BC-2.17.022
  - BC-2.17.024
verification_properties: []
inputs:
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.025.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.017.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.021.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.022.md
  - .factory/specs/behavioral-contracts/ss-17/BC-2.17.024.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "fe79905"
---

# STORY-138: ENIP Session Lifecycle, Statistics, DoS Guard, and Analyzer Summary

## Narrative

**As a** security analyst reviewing wirerust output after analyzing a pcap file,
**I want** the EtherNet/IP analyzer to classify RegisterSession/UnRegisterSession handshakes
(without emitting findings), accumulate per-flow statistics (pdu_count, command_counts),
enforce the MAX_FINDINGS DoS guard, correctly close flows and fold per-flow counters into
aggregate state, and produce a structured enip_summary with canonical field names,
**so that** the final JSON report contains accurate aggregate statistics and the analyzer is
protected against resource exhaustion.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.025 | RegisterSession (0x0065) and UnRegisterSession (0x0066) Classified and PDU-Counted; No Finding Emitted | Session handshake classify + pdu_count; NO finding |
| BC-2.17.017 | on_flow_close Removes Flow State and Updates Aggregate Counters | Flow close removes state; folds pdu_count/parse_errors/command_counts into aggregates |
| BC-2.17.021 | summarize() Emits ENIP Command Distribution and Aggregate Statistics | summarize() produces enip_summary JSON with canonical key names |
| BC-2.17.022 | MAX_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth | Hard cap at 10,000; dropped_findings counter; counters still update past cap |
| BC-2.17.024 | pdu_count Incremented Per Processed Frame and Reflected in summarize() | pdu_count per valid frame; folded to total_pdu_count on flow close |

## Acceptance Criteria

### AC-138-001: RegisterSession and UnRegisterSession are classified, PDU-counted, and emit NO finding
**Traces to:** BC-2.17.025 postconditions 1–5, BC-2.17.024 postconditions 1–2
- When `classify_enip_command(header.command)` returns `EnipCommandClass::RegisterSession` (0x0065) or `EnipCommandClass::UnRegisterSession` (0x0066):
  - `flow.pdu_count += 1` (per BC-2.17.024 Postcondition 1; at start of process_pdu)
  - `flow.command_counts.entry(header.command).or_insert(0) += 1` (per BC-2.17.025 Postcondition 2)
  - NO `Finding` is pushed to `self.all_findings` (BC-2.17.025 Postcondition 3)
  - No one-shot guard is set; no window counter is affected (BC-2.17.025 Postcondition 4)
  - `flow.is_non_enip` is not modified (BC-2.17.025 Postcondition 5)
- **Session-handle anomaly validation is explicitly DEFERRED to v0.12.0 (BC-2.17.025 Invariant 3):** No `session_registered` flag, no `session_handle` field, no session-state tracking is added by this BC or this story
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_register_session_pdu_counted_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_unregister_session_pdu_counted_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_session_command_counts_accumulated`

### AC-138-002: on_flow_close removes flow state and folds per-flow counters into aggregates
**Traces to:** BC-2.17.017 postconditions 1–6
- When `EnipAnalyzer::on_flow_close(flow_key)` is called:
  - `self.flows.remove(&flow_key)` — removes the `EnipFlowState` from the per-flow map (BC-2.17.017 Post 1)
  - `self.total_pdu_count += flow.pdu_count` — PDU count folded into aggregate (BC-2.17.017 Post 2)
  - `self.parse_errors += flow.parse_errors` — lifetime parse error count folded into aggregate (BC-2.17.017 Post 3)
  - For each `(cmd, count)` in `flow.command_counts`: `self.command_distribution.entry(cmd).or_insert(0) += count` (BC-2.17.017 Post 4)
  - If `flow_key` is not found: no-op, no panic (BC-2.17.017 Post 5)
  - Findings in `self.all_findings` are unaffected by flow close (BC-2.17.017 Post 6)
- `EnipFlowState` memory (including `carry: Vec<u8>`) is reclaimed by Rust ownership on remove (BC-2.17.017 Invariant 4)
- Aggregate counters `total_pdu_count`, `parse_errors`, `command_distribution` grow monotonically (BC-2.17.017 Invariant 2)
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_flow_close_removes_state`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_flow_close_folds_pdu_count`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_flow_close_folds_parse_errors`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_flow_close_unknown_key_no_panic`

### AC-138-003: Per-flow pdu_count and command_counts accumulated per BC-2.17.024 and BC-2.17.025
**Traces to:** BC-2.17.024 postconditions 1–5, BC-2.17.025 postcondition 2
- `flow.pdu_count: u64` increments at the start of each `process_pdu` call (valid frames only — frames rejected by `is_valid_enip_frame` do NOT increment pdu_count per BC-2.17.024 Postcondition 3)
- `flow.command_counts: HashMap<u16, u64>` increments per ENIP command code seen (including RegisterSession/UnRegisterSession per BC-2.17.025 Postcondition 2)
- Frames that emit no finding (RegisterSession, IndicateStatus, ListServices, etc.) still increment `pdu_count` (BC-2.17.024 Postcondition 4)
- pdu_count is monotonically increasing within a flow lifetime (BC-2.17.024 Invariant 2)
- At flow close, `flow.pdu_count` is folded into `self.total_pdu_count` per BC-2.17.017
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_pdu_count_increments_on_valid_frame`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_pdu_count_not_incremented_on_invalid_frame`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_command_count_accumulates`

### AC-138-004: `MAX_FINDINGS = 10,000` hard cap with `dropped_findings` counter
**Traces to:** BC-2.17.022 postconditions 1–5, invariants 1–5
- `const MAX_FINDINGS: usize = 10_000` is defined in `src/analyzer/enip.rs` (BC-2.17.022 Invariant 1)
- Every finding emit path is gated by `self.all_findings.len() < MAX_FINDINGS`
- When `all_findings.len() >= MAX_FINDINGS`:
  - No new `Finding` is pushed (BC-2.17.022 Postcondition 1)
  - `self.all_findings.len()` remains at `MAX_FINDINGS` (BC-2.17.022 Postcondition 2)
  - `self.dropped_findings += 1` (BC-2.17.022 Postcondition 3)
  - Per-flow counters (`malformed_in_window`, `command_counts`, `pdu_count`, `parse_errors`) ARE still updated (BC-2.17.022 Postcondition 4)
- One-shot guards (`write_burst_emitted`, `error_rate_emitted`, `malformed_anomaly_emitted`) are NOT set when a finding is dropped due to cap — allows future windows to retry (BC-2.17.022 Postcondition 5)
- `dropped_findings` is reported in `enip_summary.dropped_findings` by `summarize()` (BC-2.17.022 Invariant 4)
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_max_findings_cap`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_dropped_findings_incremented_at_cap`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_stats_accumulate_past_max_findings`

### AC-138-005: `summarize()` produces `enip_summary` JSON with BC-2.17.021 canonical schema
**Traces to:** BC-2.17.021 postconditions 1–4, invariants 1–4
- `EnipAnalyzer::summarize()` (or `finalize()`) produces aggregate statistics in an `enip_summary` object in the JSON output (BC-2.17.021 Postcondition 1):
  ```rust
  // EnipAnalyzer aggregate fields (consumed by summarize()):
  // self.command_distribution: HashMap<u16, u64>  — populated by on_flow_close
  // self.total_pdu_count: u64                      — folded from flow.pdu_count on close
  // self.parse_errors: u64                         — folded from flow.parse_errors on close
  // self.write_count: u64                          — CIP write-class service requests
  // self.error_count: u64                          — CIP error responses
  // self.flows_analyzed: u64                       — distinct TCP flows processed
  // self.dropped_findings: u64                     — findings dropped at MAX_FINDINGS cap

  // enip_summary JSON keys (BC-2.17.021 Postcondition 1 — CANONICAL names):
  // "command_distribution"  — map of ENIP command u16 to count (non-zero only)
  // "total_pdu_count"       — total PDUs processed across all flows
  // "parse_errors"          — CANONICAL KEY (NOT "total_parse_errors") per BC-2.17.021 Invariant 1
  // "write_count"           — CIP write-class service request total
  // "error_count"           — CIP error response total
  // "flows_analyzed"        — distinct flows count
  // "dropped_findings"      — findings suppressed by MAX_FINDINGS cap
  ```
- **CRITICAL key name constraint (BC-2.17.021 Invariant 1):** The parse error key MUST be `"parse_errors"` — NOT `"total_parse_errors"`. This is a breaking-change guard. The v0.10.0 rename lesson (BC-2.15.020 D-220) mandates the canonical name from day one.
- `summarize()` reads from `self.command_distribution`, `self.total_pdu_count`, etc. (populated by `on_flow_close` calls) — it does NOT re-scan per-flow state (BC-2.17.021 Invariant 2)
- Zero-flow case: all counts are 0; `enip_summary` object is still present in JSON (BC-2.17.021 Postcondition 2, Invariant 3)
- `summarize()` does NOT emit new findings (BC-2.17.021 Postcondition 3)
- `dropped_findings` is reported here for operator awareness (BC-2.17.021 Postcondition 1 / BC-2.17.022 Invariant 4)
- **REMOVED fields vs. story draft:** `total_frames` and `total_findings` are NOT in BC-2.17.021 schema. The correct fields are `total_pdu_count` (per BC-2.17.024) and `dropped_findings` (per BC-2.17.022). Do not add unspecified keys.
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_summarize_produces_enip_summary`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_summary_parse_errors_key_canonical`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_summary_zero_flow_case`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_summary_dropped_findings`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipFlowState.pdu_count` | `src/analyzer/enip.rs` | `u64` — valid frames processed (per BC-2.17.024) |
| `EnipFlowState.command_counts` | `src/analyzer/enip.rs` | `HashMap<u16, u64>` — per ENIP command code |
| `EnipAnalyzer.flows` | `src/analyzer/enip.rs` | `HashMap<FlowKey, EnipFlowState>` — per-flow state map |
| `EnipAnalyzer.total_pdu_count` | `src/analyzer/enip.rs` | `u64` — aggregate PDU count across closed flows |
| `EnipAnalyzer.parse_errors` | `src/analyzer/enip.rs` | `u64` — aggregate lifetime parse errors across closed flows |
| `EnipAnalyzer.command_distribution` | `src/analyzer/enip.rs` | `HashMap<u16, u64>` — aggregate command distribution |
| `EnipAnalyzer.write_count` | `src/analyzer/enip.rs` | `u64` — aggregate CIP write-class service requests |
| `EnipAnalyzer.error_count` | `src/analyzer/enip.rs` | `u64` — aggregate CIP error responses |
| `EnipAnalyzer.flows_analyzed` | `src/analyzer/enip.rs` | `u64` — distinct flow count |
| `EnipAnalyzer.dropped_findings` | `src/analyzer/enip.rs` | `u64` — findings suppressed by MAX_FINDINGS cap |
| `MAX_FINDINGS` | `src/analyzer/enip.rs` | `const usize = 10_000` |
| `EnipAnalyzer::on_flow_close()` | `src/analyzer/enip.rs` | Removes flow state; folds counters into aggregates (BC-2.17.017) |
| `EnipAnalyzer::summarize()` | `src/analyzer/enip.rs` | Produces enip_summary JSON from aggregates (BC-2.17.021) |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod session_lifecycle { ... }` |

**NOT added by this story (scope-deferred per BC-2.17.025 Invariant 3):**
- `session_registered: bool` — deferred to v0.12.0
- `session_handle: Option<u32>` — deferred to v0.12.0
- `session_anomaly_emitted: bool` — deferred to v0.12.0 (no session-anomaly T0814 in v0.11.0)

The session-handle anomaly detection (data sent without RegisterSession → T0814) is explicitly OUT OF SCOPE for v0.11.0 and deferred to v0.12.0 per BC-2.17.025 Invariant 3. This story adds NO session tracking state.

**enip_summary canonical JSON keys (BC-2.17.021 Postcondition 1):**
```
command_distribution, total_pdu_count, parse_errors, write_count, error_count, flows_analyzed, dropped_findings
```
The key `parse_errors` is CANONICAL — not `total_parse_errors` (BC-2.17.021 Invariant 1).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RegisterSession (0x0065) on a new flow | `pdu_count += 1`; `command_counts[0x0065] = 1`; no finding (BC-2.17.025) |
| EC-002 | UnRegisterSession (0x0066) after a RegisterSession | `pdu_count += 1`; `command_counts[0x0066] = 1`; no finding (BC-2.17.025) |
| EC-003 | Multiple RegisterSession frames on the same flow | Each increments `pdu_count` and `command_counts[0x0065]`; no findings (BC-2.17.025 EC-003) |
| EC-004 | SendRRData without prior RegisterSession (v0.11.0) | NO T0814 session anomaly in v0.11.0 — session-handle anomaly deferred to v0.12.0 per BC-2.17.025 Inv 3 |
| EC-005 | `all_findings.len() == MAX_FINDINGS - 1`; CIP Stop arrives | Finding pushed; `len == MAX_FINDINGS` (BC-2.17.022 EC-001) |
| EC-006 | `all_findings.len() == MAX_FINDINGS`; next finding attempt | No push; `dropped_findings += 1`; one-shot guard NOT set (BC-2.17.022 EC-002) |
| EC-007 | Flow with `pdu_count=10, parse_errors=2` closed | `total_pdu_count += 10`; `parse_errors += 2` in aggregates (BC-2.17.017) |
| EC-008 | `on_flow_close` called for unknown flow_key | No-op; no panic (BC-2.17.017 Post 5) |
| EC-009 | `summarize()` called with 0 flows | All enip_summary counts are 0; JSON object still present (BC-2.17.021 Post 2, Inv 3) |
| EC-010 | Command distribution: ListIdentity×3 + RegisterSession×2 | `command_distribution = {0x0063: 3, 0x0065: 2}` (BC-2.17.021 EC-002 pattern) |
| EC-011 | parse_errors = 3 (lifetime aggregate) | `enip_summary.parse_errors = 3` (canonical key, not "total_parse_errors") |
| EC-012 | `dropped_findings = 50` | `enip_summary.dropped_findings = 50` (BC-2.17.021 EC-003) |

## Tasks

- [ ] Add to `EnipFlowState`: `pdu_count: u64`, `command_counts: HashMap<u16, u64>` (per BC-2.17.024 and BC-2.17.025)
  - NOTE: `session_registered`, `session_handle`, `session_anomaly_emitted`, `total_frames`, `cip_service_counts` are NOT added — session-handle tracking is deferred to v0.12.0 per BC-2.17.025 Invariant 3
- [ ] Add to `EnipAnalyzer`: `total_pdu_count: u64`, `parse_errors: u64`, `command_distribution: HashMap<u16, u64>`, `write_count: u64`, `error_count: u64`, `flows_analyzed: u64`, `dropped_findings: u64`
- [ ] Define `const MAX_FINDINGS: usize = 10_000` in `src/analyzer/enip.rs`
- [ ] In `process_pdu`, at start of call: `flow.pdu_count += 1` (BC-2.17.024 Post 1)
- [ ] In `process_pdu`, after command classification: `flow.command_counts.entry(header.command).or_insert(0) += 1` (BC-2.17.025 Post 2)
- [ ] For RegisterSession and UnRegisterSession: no finding emitted; pdu_count and command_counts already updated above (BC-2.17.025 Post 3)
- [ ] Gate every finding push: `if self.all_findings.len() < MAX_FINDINGS { push } else { self.dropped_findings += 1 }` (BC-2.17.022) — do NOT set one-shot guards on cap-suppressed findings (BC-2.17.022 Post 5)
- [ ] Implement `EnipAnalyzer::on_flow_close(flow_key)`: remove flow from `self.flows`; fold `pdu_count`, `parse_errors`, `command_counts` into aggregates; no-op on missing key (BC-2.17.017)
- [ ] Implement `EnipAnalyzer::summarize()`: read aggregate fields; produce `enip_summary` JSON with canonical key `parse_errors` (NOT `total_parse_errors`); include all BC-2.17.021 fields; zero-flow case produces valid JSON object (BC-2.17.021)
- [ ] Add `mod session_lifecycle { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-138 tests
- [ ] Run `cargo test enip` — all session_lifecycle tests pass
- [ ] Run `cargo test --all-targets` — full test suite green
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod session_lifecycle { ... }`

```
session_lifecycle::test_register_session_pdu_counted_no_finding
session_lifecycle::test_unregister_session_pdu_counted_no_finding
session_lifecycle::test_session_command_counts_accumulated
session_lifecycle::test_flow_close_removes_state
session_lifecycle::test_flow_close_folds_pdu_count
session_lifecycle::test_flow_close_folds_parse_errors
session_lifecycle::test_flow_close_unknown_key_no_panic
session_lifecycle::test_pdu_count_increments_on_valid_frame
session_lifecycle::test_pdu_count_not_incremented_on_invalid_frame
session_lifecycle::test_command_count_accumulates
session_lifecycle::test_max_findings_cap
session_lifecycle::test_dropped_findings_incremented_at_cap
session_lifecycle::test_stats_accumulate_past_max_findings
session_lifecycle::test_summarize_produces_enip_summary
session_lifecycle::test_summary_parse_errors_key_canonical
session_lifecycle::test_summary_zero_flow_case
session_lifecycle::test_summary_dropped_findings
```

## Previous Story Intelligence

- STORY-134 adds `error_counts_in_window`, `error_rate_emitted`, `is_non_enip`, `error_window_start` to `EnipFlowState`
- STORY-135 adds `write_count_in_window`, `write_burst_emitted`, `write_window_start` to `EnipFlowState`
- STORY-136 adds `open_connection_count`, `close_connection_count` to `EnipFlowState`
- STORY-137 adds `carry: Vec<u8>`, `parse_errors: u64`, `malformed_in_window: u64`, `malformed_anomaly_emitted: bool`, `malformed_window_start` to `EnipFlowState`
- STORY-138 adds `pdu_count: u64`, `command_counts: HashMap<u16, u64>` to `EnipFlowState`; and `total_pdu_count`, `parse_errors` (aggregate), `command_distribution`, `write_count`, `error_count`, `flows_analyzed`, `dropped_findings` to `EnipAnalyzer`

**NOT added by STORY-138 (deferred per BC-2.17.025 Invariant 3):** `session_registered`, `session_handle`, `session_anomaly_emitted` — session-handle anomaly validation is explicitly out-of-scope for v0.11.0.

After all Wave 60/61 stories are merged, `EnipFlowState` will have all fields. STORY-138's implementer must not re-declare fields already defined by earlier stories.

**`summarize()` aggregation pattern (BC-2.17.021 Invariant 2):** `summarize()` reads from `self.command_distribution`, `self.total_pdu_count`, `self.parse_errors`, `self.write_count`, `self.error_count`, `self.flows_analyzed`, and `self.dropped_findings` — all of which are aggregate fields populated incrementally by `on_flow_close()` calls (BC-2.17.017). It does NOT re-scan `self.flows`. The pattern mirrors `DnpAnalyzer::summarize()` or `ModbusAnalyzer::summarize()` — check those for the exact return type shape used in existing reporters.

## Architecture Compliance Rules

From ADR-010 Decision 5 (DoS bounds), BC-2.17.022, BC-2.17.021, BC-2.17.025:

1. **`MAX_FINDINGS = 10,000` is non-negotiable (BC-2.17.022):** This is the global cap for `all_findings`. Never remove or bypass this guard. Every finding emit path (all detection BCs) must check `self.all_findings.len() < MAX_FINDINGS`.
2. **`dropped_findings` counter is mandatory (BC-2.17.022 Post 3 / BC-2.17.021 Post 1):** Every suppressed finding increments `self.dropped_findings`. This is reported in `enip_summary.dropped_findings`. Do not use a log warning as a substitute — the counter is a machine-readable output.
3. **One-shot guards NOT set on cap-suppressed findings (BC-2.17.022 Post 5):** When a finding is dropped because `all_findings.len() >= MAX_FINDINGS`, the corresponding one-shot guard (`write_burst_emitted`, `error_rate_emitted`, `malformed_anomaly_emitted`) is NOT set. The next window can retry.
4. **Statistics accumulate past MAX_FINDINGS cap (BC-2.17.022 Invariant 3):** `pdu_count`, `command_counts`, `parse_errors`, `malformed_in_window` continue to increment even when `all_findings` is at capacity.
5. **`parse_errors` key is canonical in JSON (BC-2.17.021 Invariant 1):** The JSON key MUST be `"parse_errors"`, NOT `"total_parse_errors"`. Failing this is a breaking API change. The key name is enforced from day one per the v0.10.0 lesson (BC-2.15.020 D-220).
6. **`summarize()` reads aggregates, NOT per-flow state (BC-2.17.021 Invariant 2):** `summarize()` reads `self.command_distribution`, `self.total_pdu_count`, etc., which are populated by `on_flow_close` calls. It does NOT re-scan `self.flows`.
7. **No session tracking in v0.11.0 (BC-2.17.025 Invariant 3):** `session_registered`, `session_handle`, `session_anomaly_emitted` are NOT added. The session-anomaly T0814 detector is NOT implemented. Any code implementing these fields or that T0814 detector is out-of-scope and must be rejected.
8. **RegisterSession/UnRegisterSession emit NO finding (BC-2.17.025 Post 3):** These are normal EtherNet/IP protocol steps. Emitting findings for session handshakes would produce extreme false-positive rates.

## Library & Framework Requirements

- `std::collections::HashMap` for `command_counts` and `cip_service_counts` — already in stdlib
- `log::warn!` for MAX_FINDINGS warning — already in project
- No new external crate dependencies

## File Structure Requirements

**Files to modify:**
- `src/analyzer/enip.rs` — add `EnipFlowState` session/stats fields; add `MAX_FINDINGS` constant; implement session lifecycle, statistics, and `finish()` method; define `EnipSummary` struct
- `tests/enip_analyzer_tests.rs` — add `mod session_lifecycle { ... }` block

**Files that may need minor touches:**
- `src/main.rs` — if `finish()` is called in the reporter chain: verify the existing `take_enip_analyzer()` → `finish()` → summary integration is correct after all changes

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/enip.rs` additions (session state, stats, MAX_FINDINGS, EnipSummary, finish()) | ~500 |
| `tests/enip_analyzer_tests.rs` session_lifecycle mod (14 tests) | ~600 |
| **Total** | **~1,100** |

## Dependency Rationale

Wave 61; depends on all Wave 60 stories (STORY-134, 135, 136, 137) to ensure `EnipFlowState` has all detection fields in place before adding statistics fields. `EnipSummary` aggregates data from all fields across all flows, so it must be the last story to touch `EnipFlowState`.
