# Evidence Report — STORY-172

**Story:** STORY-172: IEC-104 Carry Buffers + Frame-Walk Loop + Flow Lifecycle (on_data / on_flow_close)
**Wave:** 81
**Date:** 2026-07-15
**Branch:** develop (worktree STORY-172)
**Product type:** Library (effectful on_data / on_flow_close shell — no CLI/web surface; dispatch wiring is STORY-173)

---

## Full Test Suite: 192/192 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output (abbreviated — story_172 module shown; all 192 tests pass):
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 192 tests
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
[... 166 tests from story_167..171 modules, all ok ...]
test story_172::test_AC_172_001_carry_stash_c2s_partial_frame ... ok
test story_172::test_AC_172_001_carry_directional_isolation_interleaved ... ok
test story_172::test_AC_172_001_carry_stash_s2c_partial_frame ... ok
test story_172::test_AC_172_005_empty_data_slice_no_panic_no_finding ... ok
test story_172::test_AC_172_006_reopen_flow_yields_fresh_state ... ok
test story_172::test_BC_2_19_025_v12_ec001_max_conformant_partial_254_no_t0814 ... ok
test story_172::test_BC_2_19_025_v12_vector_i_split_frame_c2s_walk_first_no_t0814 ... ok
test story_172::test_BC_2_19_025_v12_vector_ii_single_delivery_s2c_walk_first_no_t0814 ... ok
test story_172::test_BC_2_19_025_v12_vector_iii_defensive_overflow_dedup_c2s ... ok
test story_172::test_BC_2_19_026_bad_start_byte_advance_one_no_finding ... ok
test story_172::test_BC_2_19_026_ec_009_back_to_back_three_frames ... ok
test story_172::test_BC_2_19_026_malformed_len_first_c2s ... ok
test story_172::test_BC_2_19_026_malformed_len_first_s2c_after_c2s ... ok
test story_172::test_BC_2_19_026_malformed_len_second_c2s ... ok
test story_172::test_BC_2_19_026_multiple_complete_frames_processed_sequentially ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_multi_frame_startdt_plus_type105_joint_effects ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_ns_desync_via_on_data_emits_t1692_001 ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_startdt_act_sets_session_started ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_stopdt_act_after_startdt_emits_t0881 ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_type105_i_frame_emits_t0827 ... ok
test story_172::test_BC_2_19_026_pc2_dispatch_type45_control_command_emits_t1692_001 ... ok
test story_172::test_BC_2_19_027_ec_010_close_with_carry_no_finding ... ok
test story_172::test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic ... ok
test story_172::test_BC_2_19_027_on_flow_close_removes_state ... ok
test story_168::proptest_vp046_frame_format_totality ... ok
test story_171::test_BC_2_19_023_proptest_extract_ns_always_in_15bit_range ... ok
test story_171::test_BC_2_19_023_proptest_extract_nr_always_in_15bit_range ... ok
test story_172::proptest_vp045_independent_run_equivalence ... ok
test story_172::proptest_vp045_direction_isolation ... ok

test result: ok. 192 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

**STORY-167 contribution:** 30 tests (story_167 module)
**STORY-168 contribution:** 34 tests (story_168 module, includes proptest)
**STORY-169 contribution:** 27 tests (story_169 module)
**STORY-170 contribution:** 45 tests (story_170 module)
**STORY-171 contribution:** 30 tests (story_171 module)
**STORY-172 contribution:** 26 tests (story_172 module)
**Total:** 192/192 PASS

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-172-001 | Directional carry buffers independent and bounded at 255 bytes | BC-2.19.025 PC1–4, Inv1–2 | 3 | `AC-001-carry-stash-and-isolation.md` | PASS |
| AC-172-002 | Carry overflow: walk-first residual-bound, T0814 + dedup (v1.2 canonical vectors i/ii/iii + EC-001) | BC-2.19.025 PC1–3, Inv1–5 | 4 | `AC-002-carry-overflow-t0814.md` | PASS |
| AC-172-003 | Frame-walk loop processes all complete APCI frames per on_data; dispatch-effect tests | BC-2.19.026 PC1–3 | 8 | `AC-003-frame-walk-dispatch-effects.md` | PASS |
| AC-172-004 | Frame-walk termination and advance modes: bad-start-byte silent resync + malformed-LEN EMIT-WITH-DEDUP | BC-2.19.026 PC4, Inv1, Inv5 | 1 | `AC-004-advance-modes-bad-start-malformed.md` | PASS |
| AC-172-005 | on_data does not panic for any byte sequence; empty-slice edge case | BC-2.19.026 PC5 (VP-047) | 1 | `AC-005-no-panic-empty-edge.md` | PASS |
| AC-172-006 | on_flow_close removes Iec104FlowState and discards carry; reopen yields fresh state | BC-2.19.027 PC1–4, Inv1–2 | 4 | `AC-006-flow-close-lifecycle.md` | PASS |
| AC-172-007 | VP-045 proptest skeletons compile — carry direction isolation | BC-2.19.025 Inv1 (VP-045) | 2 | `AC-007-vp045-proptest-skeletons.md` | PASS |
| AC-172-008 | Malformed-LEN dedup per direction: first C2S T0814, second C2S silent, first S2C independent | BC-2.19.026 Inv5 | 3 | `AC-008-malformed-len-dedup-per-direction.md` | PASS |

**Total STORY-172 test-based coverage: 26/26 (all AC-172-001..008)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Key Test Names |
|----|----|-----------|----------------|
| AC-172-001 | BC-2.19.025 PC1–4 | 3 | `test_AC_172_001_carry_stash_c2s_partial_frame`; `test_AC_172_001_carry_stash_s2c_partial_frame`; `test_AC_172_001_carry_directional_isolation_interleaved` |
| AC-172-002 | BC-2.19.025 PC1–3, Inv1–5 | 4 | `test_BC_2_19_025_v12_vector_i_split_frame_c2s_walk_first_no_t0814`; `test_BC_2_19_025_v12_vector_ii_single_delivery_s2c_walk_first_no_t0814`; `test_BC_2_19_025_v12_vector_iii_defensive_overflow_dedup_c2s`; `test_BC_2_19_025_v12_ec001_max_conformant_partial_254_no_t0814` |
| AC-172-003 | BC-2.19.026 PC1–3 | 8 | `test_BC_2_19_026_multiple_complete_frames_processed_sequentially`; `test_BC_2_19_026_ec_009_back_to_back_three_frames`; 6 × `test_BC_2_19_026_pc2_dispatch_*` |
| AC-172-004 | BC-2.19.026 PC4, Inv1 | 1 | `test_BC_2_19_026_bad_start_byte_advance_one_no_finding` |
| AC-172-005 | BC-2.19.026 PC5 | 1 | `test_AC_172_005_empty_data_slice_no_panic_no_finding` |
| AC-172-006 | BC-2.19.027 PC1–4 | 4 | `test_BC_2_19_027_on_flow_close_removes_state`; `test_BC_2_19_027_ec_010_close_with_carry_no_finding`; `test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic`; `test_AC_172_006_reopen_flow_yields_fresh_state` |
| AC-172-007 | BC-2.19.025 Inv1 (VP-045) | 2 | `proptest_vp045_direction_isolation`; `proptest_vp045_independent_run_equivalence` |
| AC-172-008 | BC-2.19.026 Inv5 | 3 | `test_BC_2_19_026_malformed_len_first_c2s`; `test_BC_2_19_026_malformed_len_second_c2s`; `test_BC_2_19_026_malformed_len_first_s2c_after_c2s` |

---

## Key Behavior Summary

### Carry Buffer Architecture

`Iec104FlowState` contains 9 fields implementing the full carry + dedup state:

| Field | Type | Purpose |
|-------|------|---------|
| `carry_c2s` | `Vec<u8>` | Per-direction carry (client-to-server) |
| `carry_s2c` | `Vec<u8>` | Per-direction carry (server-to-client) |
| `carry_overflow_reported_c2s` | `bool` | T0814 dedup for carry overflow (C2S) |
| `carry_overflow_reported_s2c` | `bool` | T0814 dedup for carry overflow (S2C) |
| `malformed_len_reported_c2s` | `bool` | T0814 dedup for malformed LEN (C2S) |
| `malformed_len_reported_s2c` | `bool` | T0814 dedup for malformed LEN (S2C) |
| `session_started` | `bool` | IEC-104 STARTDT session flag (from STORY-168) |
| `last_ns_c2s` | `Option<u16>` | N(S) sequence baseline C2S (from STORY-171) |
| `last_ns_s2c` | `Option<u16>` | N(S) sequence baseline S2C (from STORY-171) |

### Frame-Walk Loop Advance Modes (ADR-013 Decision 3)

| Condition | Advance | Finding |
|-----------|---------|---------|
| Bad start byte | +1 (no carry clear) | None |
| Malformed LEN (first per direction) | +2 | T0814 Anomaly/Possible/Medium |
| Malformed LEN (subsequent per direction) | +2 | None (dedup) |
| Valid frame | LEN+2 | From dispatcher (story_167..171) |
| Insufficient data | 0 (return) | None |

### Carry Overflow Reaction (BC-2.19.025 v1.3)

Fired at `on_data` entry against the prior call's walk residual (before current delivery):

| carry.len() at entry | T0814 emitted | Flag behavior |
|----------------------|---------------|---------------|
| 0–255 | No | Unchanged |
| 256+ (first per direction) | Yes, ONE T0814 | Flag set |
| 256+ (subsequent per direction) | No | Flag already set |

---

## Edge Case Coverage Summary

| Edge Case | BC | Test | Verdict |
|-----------|-----|------|---------|
| EC-001: residual=254 bytes (conformant max partial) | BC-2.19.025 | `test_BC_2_19_025_v12_ec001_max_conformant_partial_254_no_t0814` | PASS |
| EC-003: carry=256 bytes (adversarial, first occurrence) | BC-2.19.025 | `test_BC_2_19_025_v12_vector_iii_defensive_overflow_dedup_c2s` | PASS |
| EC-004: empty data slice | BC-2.19.026 | `test_AC_172_005_empty_data_slice_no_panic_no_finding` | PASS |
| EC-005: bad start byte mid-stream | BC-2.19.026 | `test_BC_2_19_026_bad_start_byte_advance_one_no_finding` | PASS |
| EC-006: first malformed-LEN C2S | BC-2.19.026 | `test_BC_2_19_026_malformed_len_first_c2s` | PASS |
| EC-007: second malformed-LEN C2S (dedup) | BC-2.19.026 | `test_BC_2_19_026_malformed_len_second_c2s` | PASS |
| EC-008: first malformed-LEN S2C after C2S flag | BC-2.19.026 | `test_BC_2_19_026_malformed_len_first_s2c_after_c2s` | PASS |
| EC-009: three frames back-to-back | BC-2.19.026 | `test_BC_2_19_026_ec_009_back_to_back_three_frames` | PASS |
| EC-010: on_flow_close with non-empty carry | BC-2.19.027 | `test_BC_2_19_027_ec_010_close_with_carry_no_finding` | PASS |
| EC-011: on_flow_close for unknown flow_key | BC-2.19.027 | `test_BC_2_19_027_ec_011_close_unknown_flow_key_no_panic` | PASS |

---

## Source-Level Evidence

Functions and structs confirmed present in `src/analyzer/iec104.rs`:

- `const MAX_IEC104_CARRY_BYTES: usize = 255`
- `struct Iec104Analyzer { flows: HashMap<FlowKey, Iec104FlowState> }`
- `struct Iec104FlowState` — 9 fields (carry_c2s, carry_s2c, carry_overflow_reported_c2s,
  carry_overflow_reported_s2c, malformed_len_reported_c2s, malformed_len_reported_s2c,
  session_started, last_ns_c2s, last_ns_s2c)
- `impl Iec104Analyzer::on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction)`
- `impl Iec104Analyzer::on_flow_close(&mut self, flow_key: FlowKey)`
- VP-047 fuzz harness skeleton: `fuzz/fuzz_targets/fuzz_iec104_parser.rs`

---

## Recording Method

This is an effectful library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Summary tables for carry overflow, advance modes, and edge case coverage
- Source-level verification of `Iec104Analyzer`, `Iec104FlowState` fields, and `on_data`/`on_flow_close`

VHS/Playwright recordings are not applicable (no interactive surface at this story scope;
dispatch wiring to the CLI analyzer is STORY-173).

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-carry-stash-and-isolation.md` | AC-172-001 (BC-2.19.025 PC1–4, carry stash + directional isolation) |
| `AC-002-carry-overflow-t0814.md` | AC-172-002 (BC-2.19.025 PC1–3, Inv1–5, walk-first residual-bound + T0814 + dedup) |
| `AC-003-frame-walk-dispatch-effects.md` | AC-172-003 (BC-2.19.026 PC1–3, multi-frame dispatch + 6 dispatch-effect tests) |
| `AC-004-advance-modes-bad-start-malformed.md` | AC-172-004 (BC-2.19.026 PC4, Inv1 — bad-start-byte advance mode) |
| `AC-005-no-panic-empty-edge.md` | AC-172-005 (BC-2.19.026 PC5, VP-047 — empty slice no panic) |
| `AC-006-flow-close-lifecycle.md` | AC-172-006 (BC-2.19.027 PC1–4, Inv1–2 — flow close + reopen fresh) |
| `AC-007-vp045-proptest-skeletons.md` | AC-172-007 (BC-2.19.025 Inv1, VP-045 — proptest skeletons compile) |
| `AC-008-malformed-len-dedup-per-direction.md` | AC-172-008 (BC-2.19.026 Inv5, EC-006/007/008 — malformed-LEN dedup) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute host-local paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-15).
