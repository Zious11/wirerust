---
document_type: red-gate-log
level: ops
version: "1.0"
status: complete
producer: test-writer
timestamp: 2026-06-11T00:00:00Z
phase: feature-f4
story: STORY-107
wave: 36
traces_to: .factory/stories/STORY-107.md
stub_commit: 7c0dfdb
failing_tests_commit: 2a250a6
red_gate_verified: true
---

# Red Gate Log: Wave 36 — STORY-107 (DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds)

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|---------------|-----------------|------|
| STORY-107 | 12 | Yes | PASSED |

## Stubs Created

### STORY-107: DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds

Scaffolding stubs committed at `7c0dfdb`:

- `struct Dnp3FlowState` — per-flow state struct with carry buffer, pending-request map, malformed/correlation window fields
- `fn Dnp3FlowState::new() -> Self` — constructor returning zeroed state
- `fn Dnp3Analyzer::on_data(...)` — restructured from STORY-106 skeleton; frame-walk stub returning without processing
- `fn Dnp3Analyzer::get_or_create_flow(flow_key) -> &mut Dnp3FlowState` — stub returning mutable flow reference

Failing tests committed at `2a250a6` (12 red via assertion/`todo!()` panics):

- `test_carry_buffer_accumulates_partial_frame`
- `test_carry_buffer_flushes_on_complete_frame`
- `test_carry_buffer_evicts_on_overflow`
- `test_frame_walk_gate_before_count`
- `test_frame_walk_multi_frame_single_chunk`
- `test_pending_request_inserted_on_operate`
- `test_pending_request_evicted_on_response`
- `test_pending_request_bounds_at_max`
- `test_malformed_frame_counted`
- `test_wire_valid_length_field`
- `test_parse_errors_below_min_payload`
- `test_per_flow_state_isolation`

## Red Gate Verification

### STORY-107

| Test | AC | BC | Result |
|------|----|----|--------|
| test_carry_buffer_accumulates_partial_frame | AC-001 | BC-2.15.008 | FAIL (expected — todo!/assertion) |
| test_carry_buffer_flushes_on_complete_frame | AC-002 | BC-2.15.008 | FAIL (expected) |
| test_carry_buffer_evicts_on_overflow | AC-003 | BC-2.15.009 | FAIL (expected) |
| test_frame_walk_gate_before_count | AC-004 | BC-2.15.004 | FAIL (expected) |
| test_frame_walk_multi_frame_single_chunk | AC-005 | BC-2.15.008 | FAIL (expected) |
| test_pending_request_inserted_on_operate | AC-006 | BC-2.15.016 | FAIL (expected) |
| test_pending_request_evicted_on_response | AC-007 | BC-2.15.016 | FAIL (expected) |
| test_pending_request_bounds_at_max | AC-008 | BC-2.15.016 | FAIL (expected) |
| test_malformed_frame_counted | AC-009 | BC-2.15.023 | FAIL (expected) |
| test_wire_valid_length_field | AC-010 | BC-2.15.004 | FAIL (expected) |
| test_parse_errors_below_min_payload | AC-011 | BC-2.15.008 | FAIL (expected) |
| test_per_flow_state_isolation | AC-012 | BC-2.15.016 | FAIL (expected) |

All 12 STORY-107 tests: RED (assertion failure or `todo!()` panic). Red Gate PASSED.

## Regression Check

| Existing Tests | Status |
|----------------|--------|
| Pre-existing STORY-106 and earlier tests | All pass at 2a250a6 (only STORY-107 tests red) |

Note: 3 STORY-106 `on_data` test frame payloads were corrected at `9acb31e` to be wire-valid
(LENGTH field 0x0E→0x06, frame_len=13) as part of the STORY-107 implementation burst. These
corrections did not break red-gate integrity — the 12 STORY-107 tests were written against the
corrected frames and were already red at 2a250a6.

## Hand-Off to Implementer

- Stories ready for implementation: STORY-107
- Implementation guidance:
  - Restructure `on_data` from skeleton into real carry-buffer frame-walk (gate-before-count, lifetime parse_errors)
  - `Dnp3FlowState`: carry buffer, pending-request HashMap (bounded by MAX_PENDING_REQUESTS), malformed counter + window fields
  - Consolidate `MAX_DNP3_FRAME_LEN` (deprecate `MAX_DNP3_CARRY_BYTES` as alias per PR reviewer nit)
  - Resolve DOC-106-001: add CONFIRM (0x00) to Management variant doc
  - Impl commits: 9acb31e (carry-walk impl) → f8bb076 (doc deferrals) → 8fbbbff (test strengthen) → 9fe884b (demo evidence)
  - Final merge: PR #226 → ebb4751 (2026-06-11T17:19Z)
