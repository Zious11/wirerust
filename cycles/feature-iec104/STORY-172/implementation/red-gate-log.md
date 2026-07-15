---
document_type: red-gate-log
level: ops
version: "1.0"
status: verified
producer: test-writer
timestamp: 2026-07-15T00:00:00Z
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: "STORY-172"
stub_architect_agent: "tw-172-stubs"
stub_compile_verified: true
test_writer_agent: "tw-172-tests"
red_gate_verified: true
---

# Red Gate Log: feature-iec104 STORY-172

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| STORY-172 | 19 new failing tests in mod story_172 | Yes — 19 fail, 166 pre-existing pass | VERIFIED RED — PASSED |

## Stubs Created

### STORY-172: IEC-104 carry-buffer frame-walk + flow lifecycle

- `pub const MAX_IEC104_CARRY_BYTES: usize = 255` — ADR-013 Decision 2 overflow bound (BC-2.19.025); declared at `src/analyzer/iec104.rs:166`
- `Iec104FlowState::malformed_len_reported_c2s: bool` — one-shot dedup flag for first malformed-LEN finding in C→S direction (BC-2.19.026 invariant 5); Default false
- `Iec104FlowState::malformed_len_reported_s2c: bool` — same dedup flag for S→C direction; Default false
- `pub struct Iec104Analyzer { flows: HashMap<FlowKey, Iec104FlowState>, pub all_findings: Vec<Finding> }` — top-level analyzer (mirrors Dnp3Analyzer/EnipAnalyzer pattern); declared at `src/analyzer/iec104.rs:1026`
- `pub fn Iec104Analyzer::new() -> Self` — returns empty-map instance; stub body returns struct literal
- `pub fn Iec104Analyzer::on_data(flow_key, data, timestamp, direction)` — frame-walk loop over carry buffers; `todo!("STORY-172: implement frame-walk loop with carry buffers (BC-2.19.025/026)")` at `src/analyzer/iec104.rs:1066`
- `pub fn Iec104Analyzer::on_flow_close(flow_key)` — flow teardown; `todo!("STORY-172: implement flow teardown (BC-2.19.027)")` at `src/analyzer/iec104.rs:1080`
- `fuzz/fuzz_targets/fuzz_iec104_parser.rs` — VP-047 fuzz harness skeleton registered in `fuzz/Cargo.toml` (AC-172-007)
- VP-045 proptest skeletons in `tests/iec104_analyzer_tests.rs` (two harnesses: `proptest_vp045_direction_isolation`, `proptest_vp045_independent_run_equivalence`; compile-only, assertions deferred to STORY-174)

Stub commit: `c8d46b7` — `cargo check --all-targets` clean, zero warnings; all pre-existing tests pass.

Stub evolution during Red Gate: `pub all_findings: Vec<Finding>` added to `Iec104Analyzer` at test-write time so tests can assert T0814 emission counts and attributes after `on_data` calls.

## Red Gate Verification

### AC-172-001 (BC-2.19.025 — directional carry stash)

- `test_AC_172_001_carry_stash_c2s_partial_frame` — FAIL (expected, todo!() panic)
- `test_AC_172_001_carry_stash_s2c_partial_frame` — FAIL (expected, todo!() panic)
- `test_AC_172_001_carry_directional_isolation_interleaved` — FAIL (expected, todo!() panic)

### AC-172-002 (BC-2.19.025 — carry overflow canonical vectors)

- `test_BC_2_19_025_carry_overflow_c2s_1_plus_255_emits_t0814` — FAIL (expected, todo!() panic)
  Canonical overflow vector EC-002: 1-byte carry + 255-byte delivery → overflow → T0814
- `test_BC_2_19_025_carry_overflow_s2c_200_plus_100_emits_t0814` — FAIL (expected, todo!() panic)
  Canonical overflow vector EC-003: 200-byte carry + 100-byte delivery → overflow → T0814
- `test_BC_2_19_025_ec_001_exact_255_boundary_no_t0814` — FAIL (expected, todo!() panic)
  EC-001 boundary: carry + delivery = exactly 255 → no overflow, no T0814

### AC-172-003 (BC-2.19.026 — multiple complete frames per delivery)

- `test_BC_2_19_026_multiple_complete_frames_processed_sequentially` — FAIL (expected, todo!() panic)
- `test_BC_2_19_026_ec_009_back_to_back_three_frames` — FAIL (expected, todo!() panic)
  EC-009: 3 back-to-back complete APCI frames in a single delivery

### AC-172-004 (BC-2.19.026 — bad-start-byte silent advance)

- `test_BC_2_19_026_bad_start_byte_advance_one_no_finding` — FAIL (expected, todo!() panic)

### AC-172-005 (EC-004 — empty data slice)

- `test_AC_172_005_empty_data_slice_no_panic_no_finding` — FAIL (expected, todo!() panic)

### AC-172-006 (BC-2.19.027 — flow teardown lifecycle)

- `test_BC_2_19_027_on_flow_close_removes_state` — FAIL (expected, todo!() panic)
- `test_AC_172_006_reopen_flow_yields_fresh_state` — FAIL (expected, todo!() panic)
- `test_BC_2_19_027_ec_010_close_with_carry_no_finding` — FAIL (expected, todo!() panic)
  EC-010: close with non-empty carry buffer → no finding emitted
- `test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic` — FAIL (expected, todo!() panic)

### AC-172-008 (BC-2.19.026 inv5 — malformed-LEN EMIT-WITH-DEDUP)

- `test_BC_2_19_026_malformed_len_first_c2s` — FAIL (expected, todo!() panic)
  BC-2.19.026 EC-006: first malformed-LEN in C→S emits finding, sets dedup flag
- `test_BC_2_19_026_malformed_len_second_c2s` — FAIL (expected, todo!() panic)
  BC-2.19.026 EC-007: second malformed-LEN in C→S suppressed by dedup flag
- `test_BC_2_19_026_malformed_len_first_s2c_after_c2s` — FAIL (expected, todo!() panic)
  BC-2.19.026 EC-008: first malformed-LEN in S→C emits independently (separate dedup flag)

### AC-172-007 (VP-045 proptest skeletons — compile-verify)

- `proptest_vp045_direction_isolation` — FAIL (expected, todo!() panic via on_data call)
- `proptest_vp045_independent_run_equivalence` — FAIL (expected, todo!() panic via on_data call)

All 19 failures are `todo!()` panics. The panic messages directly reference the behavior under test:
- `on_data` panics: `"STORY-172: implement frame-walk loop with carry buffers (BC-2.19.025/026)"`
- `on_flow_close` panics: `"STORY-172: implement flow teardown (BC-2.19.027)"`

Command: `cargo test --test iec104_analyzer_tests story_172`
Result: `0 passed; 19 failed`

## Regression Check

| Existing Tests | Status |
|---------------|--------|
| 166 pre-existing IEC-104 tests (all prior waves, mod story_162 through story_171) | all pass |
| All other test targets | all pass |

`cargo test --all-targets` compiles clean; zero regressions; only `mod story_172` is red.

Test commit: `87b924f`

## Hand-Off to Implementer

- Stories ready for implementation: STORY-172
- Implementation guidance:
  1. Implement `Iec104Analyzer::on_data` at `src/analyzer/iec104.rs:1059` — frame-walk loop
     consuming `carry_c2s`/`carry_s2c` per direction, bounded by `MAX_IEC104_CARRY_BYTES`
     (BC-2.19.025), emitting T0814 on overflow, advancing past bad-start-bytes silently,
     dedup-gating T0814 malformed-LEN findings via `malformed_len_reported_*` flags (BC-2.19.026).
  2. Implement `Iec104Analyzer::on_flow_close` at `src/analyzer/iec104.rs:1079` — remove flow
     state from `self.flows`, discard any carry bytes, emit no finding (BC-2.19.027).
  3. Verify: `cargo test --test iec104_analyzer_tests story_172` → 19 passed, 0 failed.
  4. Verify: `cargo test --all-targets` → all pass, 0 regressions.

Verifier: orchestrator ran suite independently 2026-07-15.
