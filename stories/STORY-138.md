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
**I want** the EtherNet/IP analyzer to track session state (RegisterSession/UnRegisterSession),
accumulate per-flow statistics (frames, commands, CIP services), enforce the MAX_FINDINGS
DoS guard, emit a T0814 DoS finding when the session establishes without a proper
RegisterSession sequence, and produce a structured summary of all ENIP analysis results,
**so that** the final report contains actionable statistics and the analyzer is protected
against resource exhaustion.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.17.025 | `EnipSummary` produced at end of analysis with per-flow stats | Core summary implementation |
| BC-2.17.017 | RegisterSession/UnRegisterSession lifecycle tracking in EnipFlowState | Session state tracking |
| BC-2.17.021 | Per-flow frame and command statistics accumulation | Statistics accumulation |
| BC-2.17.022 | `all_findings` capped at MAX_FINDINGS (10,000) DoS guard | DoS guard enforcement |
| BC-2.17.024 | Session anomaly: data sent without RegisterSession emits T0814 | Session-anomaly detection |

## Acceptance Criteria

### AC-138-001: RegisterSession and UnRegisterSession update flow session state
**Traces to:** BC-2.17.017 postconditions 1–3
- When an ENIP RegisterSession command (`EnipCommand::RegisterSession`, code 0x0065) is processed:
  - `flow.session_registered = true`
  - `flow.session_handle = Some(header.session_handle)`
- When an ENIP UnRegisterSession command (`EnipCommand::UnRegisterSession`, code 0x0066) is processed:
  - `flow.session_registered = false`
  - `flow.session_handle = None`
- Session state persists across PDUs within the same `EnipFlowState`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_register_session_sets_flag`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_unregister_session_clears_flag`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_session_handle_stored`

### AC-138-002: Data sent without RegisterSession emits T0814 session-anomaly finding
**Traces to:** BC-2.17.024 postconditions 1–2
- Given `flow.session_registered == false`
- When a SendRRData (`EnipCommand::SendRRData`, 0x0072) or SendUnitData (`EnipCommand::SendUnitData`, 0x006F) command arrives
- AND `flow.session_anomaly_emitted == false`
- AND `flow.is_non_enip == false`
- AND `all_findings.len() < MAX_FINDINGS`
- Then ONE `Finding`:
  - `category: ThreatCategory::DenialOfService` (or equivalent)
  - `verdict: Verdict::Possible`
  - `confidence: Confidence::Medium`
  - `summary: "ENIP data sent without RegisterSession: possible session hijack or scanner (T0814)"`
  - `mitre_techniques: vec!["T0814"]`
  - `flow.session_anomaly_emitted = true` (one-shot guard per flow)
- This is a DIFFERENT T0814 detection from STORY-137's malformed-frame T0814 — both exist independently; both set separate one-shot guards
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_t0814_data_without_session`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_t0814_data_after_register_no_finding`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_t0814_session_anomaly_one_shot`

### AC-138-003: Per-flow statistics are accumulated in EnipFlowState
**Traces to:** BC-2.17.021 postconditions 1–4
- `flow.total_frames: u32` increments on each successfully parsed ENIP frame (valid frames only)
- `flow.command_counts: HashMap<EnipCommand, u32>` increments per command code seen
- `flow.cip_service_counts: HashMap<CipServiceClass, u32>` increments per CIP service class seen (0x00B2 items only)
- Statistics are per-flow (each `EnipFlowState` tracks its own)
- Statistics accumulate regardless of `is_non_enip` flag (but invalid frames do not contribute to `total_frames`)
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_frame_count_increments`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_command_count_accumulates`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_cip_service_count_accumulates`

### AC-138-004: `MAX_FINDINGS = 10,000` hard cap on `all_findings`
**Traces to:** BC-2.17.022 postconditions 1–2
- `const MAX_FINDINGS: usize = 10_000` is defined in `src/analyzer/enip.rs`
- Every finding emit is gated by `self.all_findings.len() < MAX_FINDINGS`
- When `all_findings.len() == MAX_FINDINGS`, all subsequent finding pushes are silently skipped
- ONE log warning is emitted when MAX_FINDINGS is first reached: `"ENIP: MAX_FINDINGS (10000) reached; suppressing further findings"`
- Statistics and session state continue to accumulate even when findings are suppressed
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_max_findings_cap`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_stats_accumulate_past_max_findings`

### AC-138-005: `EnipSummary` produced at end of analysis via `finish()`
**Traces to:** BC-2.17.025 postconditions 1–5
- `EnipAnalyzer::finish() -> EnipSummary` (or `finalize()`) produces a summary struct:
  ```rust
  pub struct EnipSummary {
      pub total_findings: usize,
      pub total_flows: usize,
      pub flows_quarantined: usize,         // is_non_enip == true
      pub total_frames_parsed: u64,         // sum across all flows
      pub command_counts: HashMap<String, u32>, // command name → count
      pub cip_service_counts: HashMap<String, u32>, // service name → count
      pub open_connection_count: u32,       // ForwardOpen total
      pub close_connection_count: u32,      // ForwardClose total
      pub finding_summary: Vec<FindingSummaryEntry>, // top findings by category
  }
  ```
- `finish()` is called after all PDUs are processed (called by `take_enip_analyzer()` chain in `main.rs`)
- `total_findings = self.all_findings.len()`
- `flows_quarantined = flows where is_non_enip == true`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_finish_produces_summary`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_summary_flows_quarantined_count`
- **Test:** `tests/enip_analyzer_tests.rs::session_lifecycle::test_summary_connection_counts`

## Architecture Mapping

| Component | Location | Role |
|-----------|----------|------|
| `EnipFlowState.session_registered` | `src/analyzer/enip.rs` | `bool` — RegisterSession state |
| `EnipFlowState.session_handle` | `src/analyzer/enip.rs` | `Option<u32>` — active session handle |
| `EnipFlowState.session_anomaly_emitted` | `src/analyzer/enip.rs` | `bool` — T0814 session-anomaly one-shot guard |
| `EnipFlowState.total_frames` | `src/analyzer/enip.rs` | `u32` — valid frames parsed |
| `EnipFlowState.command_counts` | `src/analyzer/enip.rs` | `HashMap<EnipCommand, u32>` |
| `EnipFlowState.cip_service_counts` | `src/analyzer/enip.rs` | `HashMap<CipServiceClass, u32>` |
| `MAX_FINDINGS` | `src/analyzer/enip.rs` | `const usize = 10_000` |
| `EnipSummary` struct | `src/analyzer/enip.rs` | Output type from `EnipAnalyzer::finish()` |
| `EnipAnalyzer::finish()` | `src/analyzer/enip.rs` | Aggregates per-flow stats → `EnipSummary` |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod session_lifecycle { ... }` |

**Two distinct T0814 detections:**
- STORY-137 T0814: triggered by `malformed_count >= MALFORMED_ANOMALY_THRESHOLD` (malformed frames, frame-walk level)
- STORY-138 T0814: triggered by `SendRRData/SendUnitData without prior RegisterSession` (session-protocol level)

Both use `mitre_techniques: vec!["T0814"]` and both are valid T0814 detections. They have separate one-shot guards (`dos_emitted` for STORY-137, `session_anomaly_emitted` for this story).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RegisterSession followed by SendRRData | No T0814 session anomaly (session_registered=true) |
| EC-002 | SendRRData without prior RegisterSession | T0814 session anomaly; guard set |
| EC-003 | Second SendRRData without session | Guard prevents second finding |
| EC-004 | UnRegisterSession then SendRRData | session_registered=false again → T0814 if guard not set |
| EC-005 | `all_findings` at 9,999; new finding attempt | 9,999th finding added; cap not yet hit |
| EC-006 | `all_findings` at 10,000 | All subsequent findings silently suppressed; log warning |
| EC-007 | 2 flows processed; 1 quarantined | `summary.flows_quarantined = 1` |
| EC-008 | `finish()` called with 0 flows | Empty summary with zero counts |
| EC-009 | `finish()` returns total_frames summed across all flows | Sum is correct |
| EC-010 | Command counts for ListIdentity×3 | `command_counts["ListIdentity"] = 3` |

## Tasks

- [ ] Add to `EnipFlowState`: `session_registered: bool`, `session_handle: Option<u32>`, `session_anomaly_emitted: bool`, `total_frames: u32`, `command_counts: HashMap<EnipCommand, u32>`, `cip_service_counts: HashMap<CipServiceClass, u32>`
- [ ] Define `const MAX_FINDINGS: usize = 10_000` in `src/analyzer/enip.rs`
- [ ] In `process_pdu`, after command classification:
  - Increment `total_frames` for each valid frame
  - Increment `command_counts[command]`
  - On RegisterSession: set `session_registered=true`, `session_handle=Some(header.session_handle)`
  - On UnRegisterSession: set `session_registered=false`, `session_handle=None`
  - On SendRRData or SendUnitData with `!session_registered && !session_anomaly_emitted && !is_non_enip && all_findings.len() < MAX_FINDINGS`: emit T0814 session anomaly; set guard
- [ ] In CIP-layer processing (for 0x00B2 items): increment `cip_service_counts[service_class]`
- [ ] Define `EnipSummary` struct with all required fields
- [ ] Implement `EnipAnalyzer::finish(&self) -> EnipSummary` — aggregate across `self.flow_states` (or equivalent map of flow states)
- [ ] Add log warning when MAX_FINDINGS is first reached (use `warn!` macro, one-time via a `max_findings_warned: bool` field on `EnipAnalyzer`)
- [ ] Add `mod session_lifecycle { ... }` test wrapper to `tests/enip_analyzer_tests.rs` with all AC-138 tests
- [ ] Run `cargo test enip` — all session_lifecycle tests pass
- [ ] Run `cargo test --all-targets` — full test suite green
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod session_lifecycle { ... }`

```
session_lifecycle::test_register_session_sets_flag
session_lifecycle::test_unregister_session_clears_flag
session_lifecycle::test_session_handle_stored
session_lifecycle::test_t0814_data_without_session
session_lifecycle::test_t0814_data_after_register_no_finding
session_lifecycle::test_t0814_session_anomaly_one_shot
session_lifecycle::test_frame_count_increments
session_lifecycle::test_command_count_accumulates
session_lifecycle::test_cip_service_count_accumulates
session_lifecycle::test_max_findings_cap
session_lifecycle::test_stats_accumulate_past_max_findings
session_lifecycle::test_finish_produces_summary
session_lifecycle::test_summary_flows_quarantined_count
session_lifecycle::test_summary_connection_counts
```

## Previous Story Intelligence

- STORY-134 adds `error_counts_in_window`, `error_rate_emitted`, `is_non_enip`, `error_window_start` to `EnipFlowState`
- STORY-135 adds `write_count_in_window`, `write_burst_emitted`, `write_window_start` to `EnipFlowState`
- STORY-136 adds `open_connection_count`, `close_connection_count` to `EnipFlowState`
- STORY-137 adds `carry`, `malformed_count`, `dos_emitted` to `EnipFlowState`
- STORY-138 adds `session_registered`, `session_handle`, `session_anomaly_emitted`, `total_frames`, `command_counts`, `cip_service_counts` to `EnipFlowState`

After all Wave 60 stories are merged, `EnipFlowState` will have all fields. STORY-138's implementer must not re-declare fields already defined by earlier stories.

**`finish()` aggregation pattern:** The `EnipAnalyzer` maintains a `flow_states: HashMap<FlowKey, EnipFlowState>`. `finish()` iterates over all entries, summing `total_frames`, merging `command_counts`, summing `open_connection_count` etc., and counting `is_non_enip` flows. The pattern mirrors `DnpAnalyzer::summarize()` or `ModbusAnalyzer::finish()` — check those for the exact return type shape used in existing reporters.

## Architecture Compliance Rules

From ADR-010 Decision 5 (DoS bounds) and BC-2.17.022:

1. **`MAX_FINDINGS = 10,000` is non-negotiable (BC-2.17.022):** This is the global cap for `all_findings`. Never remove or bypass this guard. Every finding emit path (all 9 detection BCs) must check `self.all_findings.len() < MAX_FINDINGS`.
2. **Two T0814 detections are independent and coexist:** STORY-137's `dos_emitted` guard is for malformed-frame T0814. STORY-138's `session_anomaly_emitted` guard is for session-protocol T0814. They have different triggers and different guards. Both can fire for the same flow (if the flow has both malformed frames AND sends data without registration).
3. **Statistics accumulate past MAX_FINDINGS cap (BC-2.17.022 Invariant 2):** `total_frames`, `command_counts`, `cip_service_counts` continue to increment even when `all_findings` is at capacity. The cap only silences finding emission, not statistics.
4. **`finish()` is called exactly once (BC-2.17.025):** After `take_enip_analyzer()` transfers ownership to the reporter. The reporter calls `analyzer.finish()` to get `EnipSummary`. `finish()` should be `&self` (or `self` by consumption if the design requires it) — check the existing analyzer pattern.
5. **Session anomaly T0814 is per-flow, not per-run:** The `session_anomaly_emitted` guard is on `EnipFlowState`, not on `EnipAnalyzer`. Each flow can fire once. Multiple flows can each fire one T0814 session anomaly.

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
