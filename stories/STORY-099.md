---
document_type: story
story_id: STORY-099
epic_id: E-12
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 3
inputs:
  - .factory/specs/behavioral-contracts/ss-09/BC-2.09.007.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.055.md
  - .factory/specs/verification-properties/vp-021-timestamp-provenance-threading.md
  - .factory/feature-delta/issue-100-pcap-timestamps/delta-analysis.md
input-hash: 36c2b0d
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-098]
blocks: []
behavioral_contracts:
  - BC-2.09.007
  - BC-2.04.055
verification_properties:
  - VP-021
priority: P1
cycle: v0.2.0-feature-100
wave: 30
target_module: tests
subsystems: [SS-09, SS-04, SS-06, SS-07]
estimated_days: 2
tdd_mode: strict
feature_id: issue-100-pcap-timestamps
github_issue: 100
assumption_validations: []
---

# STORY-099: Verify Timestamp Provenance End-to-End (VP-021)

## Narrative

- **As a** spec steward validating the issue-100 feature delivery
- **I want** end-to-end integration tests and proptests that verify `Finding.timestamp` is correctly populated from pcap `ts_sec` across both the hot-path flush and close-flush cases
- **So that** VP-021 (Timestamp Provenance Threading) is demonstrably satisfied, cross-flow isolation holds, and the serde serialization behavior is confirmed correct before the feature is locked

## Behavioral Contracts

| BC | Title |
|----|-------|
| BC-2.09.007 | Finding.timestamp Carries Capture-Relative Pcap Timestamp from on_data Call Site |
| BC-2.04.055 | StreamHandler::on_data Carries Capture-Relative Timestamp Parameter |

## Acceptance Criteria

### AC-001 (traces to BC-2.09.007 postcondition 1; verifies VP-021 hot-path case)
An integration test constructs a synthetic TCP flow with HTTP payload packets at `ts_sec=1_000_000`, runs the full pipeline through `TcpReassembler::process_packet` → `StreamDispatcher::on_data` → `HttpAnalyzer::on_data`, and asserts that at least one emitted `Finding` has `timestamp == Some(DateTime::from_timestamp(1_000_000, 0).unwrap())`.
- **Test:** `test_finding_timestamp_hot_path()` in `tests/timestamp_threading_tests.rs` (or `tests/reassembly_engine_tests.rs`).

### AC-002 (traces to BC-2.09.007 postcondition 3; verifies VP-021 close-flush case)
An integration test constructs a synthetic TCP flow where the FIN packet has a specific `ts_sec=2_000_000`, triggers flow close via FIN sequence, and asserts that the resulting Finding (if any) has `timestamp == Some(DateTime::from_timestamp(2_000_000, 0).unwrap())`. If the close-flush path produces no Finding for this specific scenario, the test verifies instead that `close_flow` passes `flow.last_seen` correctly using a recording handler.
- **Test:** `test_finding_timestamp_close_flush()`.

### AC-003 (traces to BC-2.09.007 postcondition 6 and invariant 1; verifies VP-021 segment-limit exception)
An integration test drives the reassembler past `MAX_SEGMENTS_PER_DIRECTION` to produce the segment-limit summary finding, then asserts `finding.timestamp == None` for that specific finding. Additionally, asserts that JSON serialization of that finding via `serde_json::to_string` does NOT contain a `"timestamp"` key (confirming `skip_serializing_if = "Option::is_none"` fires).
- **Test:** `test_segment_limit_summary_timestamp_is_none_and_absent_from_json()`.

### AC-004 (traces to BC-2.09.007 postcondition 5; serde JSON correctness)
An integration test takes a `Finding` with `timestamp = Some(DateTime::from_timestamp(1_000_000, 0).unwrap())` and asserts that `serde_json::to_string(&finding)` produces JSON containing `"timestamp": "2001-09-08T21:46:40Z"` (ISO-8601 UTC format via chrono's serde integration).
- **Test:** `test_finding_timestamp_json_serialization()`.

### AC-005 (traces to BC-2.04.055 postcondition 1; verifies VP-021 two-case proptest)
A proptest exercises the hot-path flush case with arbitrary `ts_sec in 0u32..u32::MAX`: for each `ts_sec`, process at least one HTTP-payload packet at that timestamp and assert every non-summary Finding has `timestamp == Some(DateTime::from_timestamp(ts_sec as i64, 0).unwrap_or_default())`.
- **Test:** `prop_finding_timestamp_matches_on_data_timestamp()` using `proptest!` macro.

### AC-006 (traces to BC-2.09.007 invariant 4 and BC-2.04.055 invariant 3; verifies VP-021 cross-flow isolation)
A proptest (or integration test) creates two distinct TCP flows A (`src_ip=10.0.0.1`) and B (`src_ip=10.0.0.2`) with distinct `ts_sec` values: `ts_a in 1u32..500_000u32` and `ts_b in 500_001u32..1_000_000u32`. Both flows emit at least one Finding each. The test asserts that Findings attributed to flow A have `timestamp` derived from `ts_a` only, and Findings attributed to flow B have `timestamp` derived from `ts_b` only. No cross-contamination.
- **Test:** `prop_cross_flow_timestamp_isolation()` following the VP-021 proof harness skeleton.

### AC-007 (traces to BC-2.09.007 invariant 2 — lossless conversion boundary)
A unit test asserts the boundary conversions from BC-2.09.007 EC-003 and EC-004: `ts_sec=0` produces `Some(1970-01-01T00:00:00Z)` and `ts_sec=u32::MAX` produces a `Some(...)` value (not `None`, not a panic), confirming the conversion is lossless for the entire `u32` range.
- **Test:** `test_timestamp_conversion_boundary_values()`.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Integration test harness | `tests/timestamp_threading_tests.rs` (new file) | Effectful (constructs reassembler, runs packets) |
| proptest harness (VP-021) | `tests/timestamp_threading_tests.rs` | Effectful (stateful pipeline; proptest manages state per run) |
| `serde_json` serialization assertion | `tests/timestamp_threading_tests.rs` | Pure (serialization is deterministic) |
| `DateTime::from_timestamp` boundary test | `tests/timestamp_threading_tests.rs` | Pure |
| `src/findings.rs` | No change | `Finding.timestamp` field already exists; serde attribute already correct |

**Subsystem anchor justification:**
- SS-09 (Findings) is the primary verification target: this story verifies that `Finding.timestamp` carries the correct value end-to-end.
- SS-04 (Reassembly), SS-06 (HTTP), SS-07 (TLS) are all exercised by the integration tests as they form the pipeline under test.
- The test infrastructure lives in `tests/` which does not belong to a single subsystem; this story's scope is end-to-end verification across all four subsystems.

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `ts_sec=0` (epoch) in proptest | `Finding.timestamp = Some(1970-01-01T00:00:00Z)`; proptest must not filter out 0 from the strategy range |
| EC-002 | `ts_sec=u32::MAX` in boundary test | Lossless conversion; chrono handles ~2106 CE; no panic or None |
| EC-003 | Segment-limit summary Finding in JSON output | `"timestamp"` key is absent from JSON (not `"timestamp": null`); serde's `skip_serializing_if` behavior confirmed |
| EC-004 | Two flows with identical timestamps but different FlowKeys | Each flow's Findings carry that timestamp; isolation test still passes because the test checks per-FlowKey attribution, not timestamp uniqueness |
| EC-005 | proptest finds a ts_sec value that triggers an edge in DateTime conversion | Test must pass for all valid u32 values; if chrono's `from_timestamp` returns None for any valid u32, the test exposes it as a bug |
| EC-006 | Integration test with TLS analyzer instead of HTTP | TLS-path integration test follows same structure; at least one test variant should cover TLS (AC-001 may use HTTP; a secondary assertion or separate test covers TLS) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `tests/timestamp_threading_tests.rs` (integration tests) | Effectful | Constructs mutable reassembler state; drives packet processing |
| `tests/timestamp_threading_tests.rs` (serde assertions) | Pure | `serde_json::to_string` is deterministic; no I/O |
| `tests/timestamp_threading_tests.rs` (proptest harnesses) | Effectful | proptest manages state across runs; reassembler is mutable |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| `tests/reassembly_engine_tests.rs` (patterns reference for test harness construction) | ~6,000 |
| `src/reassembly/mod.rs` (reassembler API reference) | ~4,000 |
| `src/analyzer/http.rs` (HttpAnalyzer API reference) | ~5,000 |
| `src/findings.rs` (Finding struct and serde) | ~2,000 |
| BC files (2 BCs: BC-2.09.007, BC-2.04.055) | ~6,000 |
| VP-021 proof harness skeleton | ~2,500 |
| `tests/timestamp_threading_tests.rs` (new file to be written) | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~34,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~17%** |

## Tasks (MANDATORY)

1. [ ] Create `tests/timestamp_threading_tests.rs` — this is the new test file for VP-021. Add it to `Cargo.toml` as a test target if needed (check whether test discovery is automatic for files in `tests/`).
2. [ ] Write failing test scaffolds for AC-001 through AC-007 using `todo!()` stubs or empty assertion bodies. Confirm `cargo test --all-targets` fails (Red Gate) because VP-021 scenarios are not yet verified (the implementation from STORY-097 + STORY-098 is complete, but the E2E assertions do not exist yet).
3. [ ] Implement `test_finding_timestamp_hot_path()` (AC-001): craft SYN + HTTP GET packet bytes at `ts_sec=1_000_000`, drive through `TcpReassembler` + `HttpAnalyzer`, collect findings, assert `timestamp == Some(2001-09-08T21:46:40Z)`. Reference existing test patterns in `tests/reassembly_engine_tests.rs` for packet construction (e.g., how existing tests build `RawPacket` with `timestamp_secs`).
4. [ ] Implement `test_finding_timestamp_close_flush()` (AC-002): set up flow, send data packets at `ts_sec=1_500_000`, then FIN at `ts_sec=2_000_000`, drive `finalize` or FIN-path; assert that the finding carries the expected timestamp from `flow.last_seen`.
5. [ ] Implement `test_segment_limit_summary_timestamp_is_none_and_absent_from_json()` (AC-003): follow existing segment-limit test pattern from STORY-021; assert `timestamp == None` and `!json_str.contains("timestamp")`.
6. [ ] Implement `test_finding_timestamp_json_serialization()` (AC-004): construct a `Finding` directly with `timestamp = Some(DateTime::from_timestamp(1_000_000, 0).unwrap())`, serialize via `serde_json::to_string`, assert `json_str.contains("\"2001-09-08T21:46:40Z\"")`.
7. [ ] Implement `prop_finding_timestamp_matches_on_data_timestamp()` (AC-005): follow VP-021 proptest harness skeleton in `vp-021-timestamp-provenance-threading.md`. Use `proptest! { ... }` macro. Strategy: `ts_sec in 0u32..=u32::MAX`.
8. [ ] Implement `prop_cross_flow_timestamp_isolation()` (AC-006): two distinct flows with non-overlapping timestamp ranges; assert per-flow attribution. Reference VP-021 harness skeleton for the strategy definition (`ts_a in 1u32..500_000u32`, `ts_b in 500_001u32..1_000_000u32`).
9. [ ] Implement `test_timestamp_conversion_boundary_values()` (AC-007): assert `DateTime::from_timestamp(0, 0)` is `Some(...)` equal to Unix epoch and `DateTime::from_timestamp(u32::MAX as i64, 0)` is `Some(...)` (not None, no panic).
10. [ ] **Green Gate:** `cargo test --all-targets` — all 7 new tests and both proptests pass. No regressions.
11. [ ] `cargo clippy --all-targets -- -D warnings` clean.
12. [ ] `cargo fmt --check` clean.
13. [ ] Update VP-021 `proof_completed_date` and `verified_at_commit` fields in `vp-021-timestamp-provenance-threading.md` after the Green Gate is confirmed (state-manager does this at commit time).

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-097 | `on_data` gains `timestamp: u32` param; hot-path passes current packet ts; close-flush passes `flow.last_seen`; all 5 implementors updated | Per-flow `last_ts: u32` storage pattern established in `HttpAnalyzer` and `TlsAnalyzer` | Trait break is a compile-time guarantee — this story depends on STORY-097 being merged first |
| STORY-098 | All 21 flow-data-path emission sites set `timestamp: Some(...)`; segment-limit summary retains `None`; `DateTime::from_timestamp(ts as i64, 0)` conversion pattern confirmed | Test pattern for `test_http_findings_have_timestamp()` in STORY-098 is a direct predecessor to the E2E test here | Check the exact `chrono` API for `from_timestamp` — it may return `Option` or be infallible depending on the workspace version. STORY-098 will have established the working pattern. |

**Key reference:** VP-021 `vp-021-timestamp-provenance-threading.md` contains a complete Rust proof harness skeleton with exact function names (`test_finding_timestamp_hot_path`, `test_finding_timestamp_segment_limit_summary_is_none`, `prop_finding_timestamp_matches_on_data_timestamp`, `prop_cross_flow_timestamp_isolation`). The implementer should use these exact names to match the VP's test-anchor expectations.

**Test construction pattern:** Examine `tests/reassembly_engine_tests.rs` for the existing patterns used to construct `RawPacket` values with `timestamp_secs` set (this field is already in `RawPacket` per F1 §2). The E2E test needs a SYN packet at time T, followed by a data packet (HTTP GET) also at T, then optionally a FIN — all constructed to produce a valid TCP stream that the reassembler will flush to the HTTP analyzer.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| VP-021 proof method is integration + proptest (NOT Kani) | VP-021; F1 §5.3 | Do not attempt to write a Kani harness for this VP; DateTime + HashMap exceed tractable Kani bounds |
| Test file is `tests/timestamp_threading_tests.rs` (new file) | VP-021 proof harness location | Keeps VP-021 tests isolated from the 16,090-line `reassembly_engine_tests.rs`; easier to navigate |
| proptest strategies must include `ts_sec=0` (epoch) | BC-2.09.007 EC-003 | Use `0u32..=u32::MAX` not `1u32..u32::MAX` |
| JSON serialization test must assert ABSENCE of `"timestamp"` key (not just `null`) | BC-2.09.007 postcondition 6 | `serde(skip_serializing_if = "Option::is_none")` omits the key entirely; test must assert `!json_str.contains("\"timestamp\"")` for None case |
| VP-021 `verification_lock` must remain `false` until this story's Green Gate | VP-021 lifecycle fields | Do not set `verification_lock: true` prematurely; lock is set at F6 formal hardening gate by spec-steward |
| Cross-flow isolation test must use distinct `src_ip` values to produce distinct `FlowKey` values | BC-2.09.007 invariant 4; VP-021 cross-flow isolation property | `FlowKey` is canonicalized from (src_ip, dst_ip, src_port, dst_port); distinct src_ip guarantees distinct keys |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| `proptest` | workspace version (current: `1.x`) | `prop_compose!` and `proptest!` macros for AC-005 and AC-006; already used by VP-006 and VP-014 |
| `chrono` | workspace version (current: `0.4`) | `DateTime::from_timestamp` and serde integration for ISO-8601 format in AC-004 |
| `serde_json` | workspace version | `serde_json::to_string` serialization assertion in AC-004 |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `tests/timestamp_threading_tests.rs` | **create** | New test file for VP-021; all 7 AC tests live here |
| `src/findings.rs` | **no change** | Confirmed: `Finding.timestamp` field, serde attribute, and `DateTime<Utc>` type are already correct |
| `.factory/specs/verification-properties/vp-021-timestamp-provenance-threading.md` | **update (state-manager)** | After Green Gate: update `proof_completed_date`, `verified_at_commit` fields |
