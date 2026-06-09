---
document_type: story
story_id: STORY-098
epic_id: E-12
version: "1.1"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 3
inputs:
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.007.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.055.md
  - .factory/feature-delta/issue-100-pcap-timestamps/delta-analysis.md
input-hash: 8b39dcb
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-097]
blocks: [STORY-099]
behavioral_contracts:
  - BC-2.09.007
  - BC-2.04.055
verification_properties:
  - VP-021
priority: P1
cycle: v0.2.0-feature-100
wave: 29
target_module: analyzer
subsystems: [SS-06, SS-07, SS-04, SS-09]
estimated_days: 3
tdd_mode: strict
feature_id: issue-100-pcap-timestamps
github_issue: 100
---

# STORY-098: Attach Pcap Timestamp to Emitted Findings

## Narrative

- **As a** forensic analyst reviewing wirerust JSON/CSV output
- **I want** every `Finding` emitted from a flow-data path to carry a `timestamp` field populated from the pcap capture-relative `ts_sec` value
- **So that** I can correlate detections with the original packet capture timeline rather than seeing `null` / omitted timestamps on every finding

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.09.007 | Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site |
| BC-2.04.055 | StreamHandler::on_data Carries Capture-Relative Timestamp Parameter |

## Acceptance Criteria

### AC-001 (traces to BC-2.09.007 postcondition 1 — HTTP emission sites)
All 9 `HttpAnalyzer` finding-emission sites in `src/analyzer/http.rs` set `timestamp: Some(DateTime::from_timestamp(stored_last_ts as i64, 0).unwrap_or_default())` (or equivalent lossless conversion) where `stored_last_ts` is the per-flow `last_ts: u32` field updated in `on_data`. None of the 9 HTTP emission sites retain `timestamp: None`.
- **Test:** `test_http_findings_have_timestamp()` — drive `HttpAnalyzer` with a packet at known `ts_sec`; assert all emitted `Finding` values have `timestamp.is_some()`.

### AC-002 (traces to BC-2.09.007 postcondition 1 — TLS emission sites)
All 7 `TlsAnalyzer` finding-emission sites in `src/analyzer/tls.rs` set `timestamp: Some(...)` derived from the per-flow `last_ts: u32` field. None of the 7 TLS emission sites retain `timestamp: None`.
- **Test:** `test_tls_findings_have_timestamp()` — drive `TlsAnalyzer` with known `ts_sec`; assert all emitted Findings have `timestamp.is_some()`.

### AC-003 (traces to BC-2.09.007 postcondition 1 — reassembly anomaly emission sites)
The 3 anomaly finding-emission sites in `src/reassembly/mod.rs` (overlap at :493, small-segment at :533, out-of-window at :559) and the 2 sites in `src/reassembly/lifecycle.rs` (conflicting-overlap, stream-depth-exceeded) set `timestamp: Some(...)` using the timestamp value in scope at the call site. (These sites are called from paths where the current-packet `timestamp: u32` is accessible — see F1 §4.5.)
- **Test:** `test_reassembly_anomaly_findings_have_timestamp()` — trigger each reassembly anomaly type with a known `ts_sec`; assert the resulting Finding has `timestamp.is_some()`.

### AC-004 (traces to BC-2.09.007 invariant 1 — 21 of 22 sites)
Exactly 21 of 22 production emission sites set `timestamp: Some(...)`. The one exception is the segment-limit summary finding emitted from `finalize` in `src/reassembly/mod.rs`, which retains `timestamp: None`. A code audit (search for `timestamp: None` in all production `src/` files) finds exactly one occurrence in a production finding-construction expression: the segment-limit summary site.
- **Test:** `test_segment_limit_summary_finding_has_no_timestamp()` — drive reassembler past `MAX_SEGMENTS_PER_DIRECTION`; assert the segment-limit summary `Finding` has `timestamp == None`.

### AC-005 (traces to BC-2.09.007 invariant 2 — lossless u32 conversion)
The `u32 → DateTime<Utc>` conversion uses `DateTime::from_timestamp(ts_sec as i64, 0)`. For `ts_sec = 1_000_000`, the result is `Some(1970-01-12T13:46:40Z)`. For `ts_sec = 0`, the result is `Some(1970-01-01T00:00:00Z)`. For `ts_sec = u32::MAX`, the conversion is lossless (within chrono's supported range ~2106 CE).
- **Test:** `test_timestamp_conversion_known_values()` — unit test asserting the three canonical conversion vectors from BC-2.09.007 test vectors table.

### AC-006 (traces to BC-2.09.007 invariant 4 and BC-2.04.055 invariant 3)
Per-flow timestamp state in `HttpAnalyzer` and `TlsAnalyzer` is keyed by `FlowKey`, consistent with all other per-flow state maps. A finding emitted for flow A carries only flow A's last-seen timestamp; flow B's timestamp does not contaminate flow A's findings.
- **Test:** `test_cross_flow_timestamp_isolation()` — run two concurrent HTTP flows with distinct `ts_sec` values; assert each flow's findings carry only that flow's timestamp.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `HttpAnalyzer` per-flow `last_ts` storage + 9 emission updates | `src/analyzer/http.rs` | Effectful (per-flow HashMap mutation) |
| `TlsAnalyzer` per-flow `last_ts` storage + 7 emission updates | `src/analyzer/tls.rs` | Effectful (per-flow HashMap mutation) |
| Reassembly anomaly emission sites (3 in mod.rs + 2 in lifecycle.rs) | `src/reassembly/mod.rs`, `src/reassembly/lifecycle.rs` | Effectful |
| `DateTime::from_timestamp` conversion | `chrono` (pure function) | Pure |
| `Finding` struct (unchanged) | `src/findings.rs` | Pure (data struct; no changes needed) |

**Subsystem anchor justification:**
- SS-06 (HTTP Analyzer) and SS-07 (TLS Analyzer) own the primary scope: all per-flow timestamp storage additions and the 16 HTTP+TLS emission-site updates are within these subsystems.
- SS-04 (Reassembly) is touched because 6 reassembly-engine emission sites also change.
- SS-09 (Findings) is the data model boundary: `Finding.timestamp` already exists with the correct `Option<DateTime<Utc>>` type and serde attribute; no changes to SS-09 itself are required.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `on_data` called with `timestamp=0` (epoch) | `Finding.timestamp = Some(1970-01-01T00:00:00Z)`; not `None`; correct behavior per BC-2.09.007 EC-003 |
| EC-002 | Segment-limit summary finding in `finalize` | `timestamp: None`; JSON omits `"timestamp"` key; correct per BC-2.09.007 EC-002 |
| EC-003 | Flow with multiple `on_data` calls at different timestamps | `stored_last_ts` is overwritten by each call; most-recent timestamp wins; consistent with BC-2.04.055 semantics |
| EC-004 | `TlsAnalyzer` emits a finding before any `on_data` call received (defensive case) | This cannot occur in practice — `on_data` must be called before any emission path is triggered; if a zero-timestamp guard is needed, `last_ts: u32 = 0` produces `Some(1970-01-01T00:00:00Z)` which is valid |
| EC-005 | Two concurrent flows A and B in `HttpAnalyzer` | Flow A's `last_ts` map entry and flow B's `last_ts` map entry are independent; no contamination (VP-014 extension) |
| EC-006 | `ts_sec = u32::MAX` (approx. 2106 CE) | `DateTime::from_timestamp(4_294_967_295, 0)` is within chrono's range; conversion is lossless |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `src/analyzer/http.rs` (emission sites) | Effectful | Mutates per-flow state; emits to findings Vec |
| `src/analyzer/tls.rs` (emission sites) | Effectful | Mutates per-flow state; emits to findings Vec |
| `src/reassembly/mod.rs` (anomaly sites) | Effectful | Called from process_packet with mutable reassembler |
| `DateTime::from_timestamp(...)` | Pure | Deterministic; same ts_sec always produces same DateTime |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| `src/analyzer/http.rs` (full, for 9 emission sites) | ~12,000 |
| `src/analyzer/tls.rs` (full, for 7 emission sites) | ~10,000 |
| `src/reassembly/mod.rs` (anomaly emission sites section) | ~5,000 |
| `src/reassembly/lifecycle.rs` (2 emission sites) | ~2,500 |
| `src/findings.rs` (read for type confirmation) | ~2,000 |
| BC files (2 BCs: BC-2.09.007, BC-2.04.055) | ~6,000 |
| Test files (new + existing for regression) | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~46,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~23%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for AC-001 (`test_http_findings_have_timestamp()`), AC-002 (`test_tls_findings_have_timestamp()`), AC-003 (`test_reassembly_anomaly_findings_have_timestamp()`), AC-004 (`test_segment_limit_summary_finding_has_no_timestamp()`), AC-005 (`test_timestamp_conversion_known_values()`), and AC-006 (`test_cross_flow_timestamp_isolation()`).
2. [ ] **Red Gate:** Confirm tests fail — emission sites still set `timestamp: None`, so `timestamp.is_some()` assertions fail.
3. [ ] Update `HttpAnalyzer` flow state struct to add `last_ts: u32` field (initialized to `0`).
4. [ ] In `HttpAnalyzer::on_data`, update `self.flows.get_mut(flow_key).map(|s| s.last_ts = timestamp)` on every call (per BC-2.04.055 postcondition 3).
5. [ ] Update all 9 HTTP finding-emission sites in `src/analyzer/http.rs` to set `timestamp: Some(DateTime::from_timestamp(stored_state.last_ts as i64, 0).unwrap_or_default())` (or the equivalent infallible conversion pattern used in this codebase).
6. [ ] Update `TlsAnalyzer` flow state struct to add `last_ts: u32` field.
7. [ ] In `TlsAnalyzer::on_data`, update `self.flows.get_mut(flow_key).map(|s| s.last_ts = timestamp)`.
8. [ ] Update all 7 TLS finding-emission sites in `src/analyzer/tls.rs` to set `timestamp: Some(DateTime::from_timestamp(...))`.
9. [ ] Update the 3 anomaly emission sites in `src/reassembly/mod.rs` (mod.rs:493 overlap, :533 small-segment, :559 out-of-window) and the 2 sites in `src/reassembly/lifecycle.rs` to use `timestamp: Some(DateTime::from_timestamp(current_ts as i64, 0).unwrap_or_default())` where `current_ts` is the `u32` in scope at each call site.
10. [ ] Confirm the segment-limit summary finding site in `src/reassembly/mod.rs` (called from `finalize`) retains `timestamp: None` — DO NOT change it.
11. [ ] **Green Gate:** `cargo test --all-targets` passes. All six new tests are green. No regressions.
12. [ ] Perform code audit: `grep -rn 'timestamp: None' src/` — confirm exactly one result in the segment-limit summary site.
13. [ ] `cargo clippy --all-targets -- -D warnings` clean.
14. [ ] `cargo fmt --check` clean.

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-097 | `StreamHandler::on_data` gains `timestamp: u32` as 5th param; `flush_contiguous_data` passes current-packet ts; `close_flow` passes `flow.last_seen`; `StreamDispatcher` forwards unchanged; `TlsAnalyzer` and `HttpAnalyzer` already have `last_ts: u32` storage stub added | Per-flow timestamp storage keyed by `FlowKey` (same pattern as all other per-flow state) | The trait-signature break is a compile-time guarantee — this story REQUIRES STORY-097 to be merged before starting; attempting to write emission sites without the new `on_data` param will not compile. |

**Key pattern from existing analyzers:** Both `HttpAnalyzer` and `TlsAnalyzer` maintain per-flow state via `HashMap<FlowKey, FlowState>` (or equivalent). The `last_ts: u32` field added in STORY-097 follows this exact same pattern. Emission sites access `stored_state.last_ts` via the same map lookup they already use for other per-flow fields.

**Conversion pattern:** Use `DateTime::from_timestamp(ts_sec as i64, 0)` — this returns `Option<DateTime<Utc>>` in some chrono versions and `Result` in others. Use `.unwrap_or_default()` or the infallible variant (`.expect("valid u32 ts")` is safe here since any `u32` as `i64` is in-range for chrono). Check the version in `Cargo.lock` to confirm the exact API.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| All 9 HTTP emission sites set `timestamp: Some(...)` | BC-2.09.007 invariant 1 | Code audit: `grep -c 'timestamp: None' src/analyzer/http.rs` returns 0 after this story |
| All 7 TLS emission sites set `timestamp: Some(...)` | BC-2.09.007 invariant 1 | Code audit: `grep -c 'timestamp: None' src/analyzer/tls.rs` returns 0 after this story |
| Segment-limit summary finding in `finalize` retains `timestamp: None` | BC-2.09.007 invariant 1; postcondition 4 | Code review; `test_segment_limit_summary_finding_has_no_timestamp()` asserts None |
| `DateTime::from_timestamp(ts as i64, 0)` conversion is used (not a custom conversion) | BC-2.09.007 postcondition 1 | Code review; conversion is infallible for valid u32 values |
| Per-flow `last_ts` keyed by `FlowKey` in both analyzers | BC-2.09.007 invariant 4; BC-2.04.055 invariant 3 | Code review; test AC-006 cross-flow isolation |
| `src/findings.rs` must NOT be modified | F1 delta-analysis §4.8 (NOT CHANGED) | `Finding.timestamp` field already exists with correct type and serde attribute; adding a new field would be a separate story |
| Reassembly-module anomaly sites (not called via `on_data`) use the `timestamp` in scope from `process_packet` | F1 delta-analysis §4.5 | `check_anomaly_thresholds` is called from `process_packet` which holds `timestamp: u32` in scope |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `chrono` | workspace version (current: `0.4`) | `DateTime::from_timestamp(i64, u32) -> Option<DateTime<Utc>>` conversion at emission sites |
| `proptest` | workspace version | Used in AC-006 cross-flow isolation test (following VP-014 pattern) |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/http.rs` | **modify** | Add `last_ts: u32` to `HttpFlowState` (or equivalent); update `on_data` to store timestamp; update all 9 emission sites |
| `src/analyzer/tls.rs` | **modify** | Add `last_ts: u32` to `TlsFlowState` (or equivalent); update `on_data` to store timestamp; update all 7 emission sites |
| `src/reassembly/mod.rs` | **modify** | Update 3 anomaly emission sites (overlap :493, small-segment :533, out-of-window :559) to set `timestamp: Some(...)` |
| `src/reassembly/lifecycle.rs` | **modify** | Update 2 emission sites to set `timestamp: Some(...)` |
| `tests/reassembly_engine_tests.rs` | **modify** | Add AC-001, AC-002, AC-003, AC-004, AC-005, AC-006 test functions |
| `src/findings.rs` | **no change** | `Finding.timestamp: Option<DateTime<Utc>>` already correct; serde attribute already in place |

## Revision Notes

- **v1.1 (F5 fixes: HIGH-001 date vector, MED-001 site count):** Corrected `ts_sec=1_000_000` conversion vector from `2001-09-08T21:46:40Z` to `1970-01-12T13:46:40Z` (AC-005, §Background). Corrected `Some`-emitting anomaly site count from 4 to 3 in AC-003, Task 9, Architecture Mapping, and File Structure Requirements — actual sites are mod.rs:493 (overlap), :533 (small-segment), :559 (out-of-window); the previously-listed fourth site does not emit a finding from a `Some`-path. BC-2.09.007 "21 of 22" invariant references are unchanged and remain correct.
