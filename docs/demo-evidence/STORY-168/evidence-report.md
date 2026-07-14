# Evidence Report — STORY-168

**Story:** STORY-168: IEC-104 Frame Format Discrimination + U-Format Session State Machine  
**Wave:** 77  
**Date:** 2026-07-14  
**Branch:** feature/STORY-168-iec104-frame-discrimination-session-sm  
**Product type:** Library (pure-core free functions + effectful session state machine — no CLI/web surface; dispatch wiring is STORY-173)

---

## Full Test Suite: 64/64 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 64 tests
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_two_bytes ... ok
test story_167::test_BC_2_19_001_returns_none_for_one_byte ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x69_off_by_one ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0xFF_canonical_vector ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_1_and_len_2 ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_3_off_by_one_canonical_vector ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_zero_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_254_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_255_canonical_vector ... ok
test story_167::test_BC_2_19_005_apci_header_equality_and_field_layout ... ok
test story_167::test_BC_2_19_005_cf_fields_verbatim_from_data_indices_2_through_5 ... ok
test story_167::test_BC_2_19_005_i_frame_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_005_invariant_len_plus_two_in_range_for_boundaries ... ok
test story_167::test_BC_2_19_005_returns_some_for_len_253_maximum_canonical_vector ... ok
test story_167::test_BC_2_19_005_s_frame_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_005_u_frame_startdt_act_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_006_invariant_consistency_with_parse_apci_header ... ok
test story_167::test_BC_2_19_006_invariant_false_gate_implies_none_from_parse ... ok
test story_167::test_BC_2_19_006_returns_false_for_empty_slice ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_254_above_maximum ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_3_below_minimum ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_ff_out_of_range_canonical_vector ... ok
test story_167::test_BC_2_19_006_returns_false_for_one_byte_slice ... ok
test story_167::test_BC_2_19_006_returns_false_for_wrong_start_byte_canonical_vector ... ok
test story_167::test_BC_2_19_006_returns_true_for_valid_start_and_len_253 ... ok
test story_167::test_BC_2_19_006_returns_true_for_valid_start_and_len_4_canonical_vector ... ok
test story_168::test_BC_2_19_007_invariant_all_128_even_cf1_values_return_iformat ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x00_canonical_vector ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x02_canonical_vector ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x7E_canonical_vector ... ok
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0xFE_all_even_bits_set ... ok
test story_168::test_BC_2_19_008_does_not_return_sformat_for_cf1_0x03_uformat ... ok
test story_168::test_BC_2_19_008_invariant_all_64_cf1_values_bits1_0_0b01_return_sformat ... ok
test story_168::test_BC_2_19_008_returns_sformat_for_cf1_0x01_canonical_vector ... ok
test story_168::test_BC_2_19_008_returns_sformat_for_cf1_0x05_canonical_vector ... ok
test story_168::test_BC_2_19_009_invariant_all_64_cf1_values_bits1_0_0b11_return_uformat ... ok
test story_168::test_BC_2_19_009_invariant_vp046_totality_exhaustive_all_256_values ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x03_non_canonical_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x07_startdt_act_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x0B_startdt_con_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0x13_stopdt_act_canonical_vector ... ok
test story_168::test_BC_2_19_009_returns_uformat_for_cf1_0xFF_canonical_vector ... ok
test story_168::test_BC_2_19_010_startdt_act_idempotent_when_already_started ... ok
test story_168::test_BC_2_19_010_startdt_act_sets_session_started_true_on_fresh_flow ... ok
test story_168::test_BC_2_19_010_startdt_con_sets_session_started_true_on_fresh_flow ... ok
test story_168::test_BC_2_19_011_stopdt_act_after_startdt_emits_t0881_possible ... ok
test story_168::test_BC_2_19_011_stopdt_act_followed_by_startdt_act_restarts_session ... ok
test story_168::test_BC_2_19_012_invariant_stopdt_confidence_escalation_likely_vs_possible ... ok
test story_168::test_BC_2_19_012_stopdt_act_without_startdt_emits_t0881_likely ... ok
test story_168::test_BC_2_19_012_stopdt_con_sets_session_false_no_finding_act_only_mvp ... ok
test story_168::test_BC_2_19_013_invariant_testfr_does_not_modify_session_started ... ok
test story_168::test_BC_2_19_013_testfr_act_emits_no_finding_canonical_vector ... ok
test story_168::test_BC_2_19_013_testfr_con_emits_no_finding_canonical_vector ... ok
test story_168::test_BC_2_19_014_invariant_non_canonical_u_frame_does_not_advance_session_state ... ok
test story_168::test_BC_2_19_014_negative_canonical_cf1_values_do_not_emit_t0814 ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0x03_emits_t0814_possible ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0x0F_emits_t0814_possible_canonical_vector ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0x1B_emits_t0814_possible ... ok
test story_168::test_BC_2_19_014_non_canonical_cf1_0xFF_emits_t0814_possible_canonical_vector ... ok
test story_168::proptest_vp046_frame_format_totality ... ok

test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**STORY-167 contribution:** 30 tests (story_167 module)  
**STORY-168 contribution:** 34 tests (story_168 module, includes proptest)  
**Total:** 64/64 PASS

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-168-001 | `classify_frame_format` correctly classifies I-format frames | BC-2.19.007 | 5 | `AC-001-002-003-frame-discrimination.md` | PASS |
| AC-168-002 | `classify_frame_format` correctly classifies S-format frames | BC-2.19.008 | 4 | `AC-001-002-003-frame-discrimination.md` | PASS |
| AC-168-003 | `classify_frame_format` classifies U-format; total over all 256 u8 values (VP-046) | BC-2.19.009 | 7 | `AC-001-002-003-frame-discrimination.md` | PASS |
| AC-168-004 | STARTDT-act/con sets session_started=true; no finding; idempotent | BC-2.19.010 | 3 | `AC-004-startdt-session-state.md` | PASS |
| AC-168-005 | STOPDT-act while session active emits T0881 Possible | BC-2.19.011 | 2 | `AC-005-006-stopdt-t0881.md` | PASS |
| AC-168-006 | STOPDT-act without prior STARTDT emits T0881 Likely; STOPDT-con no finding | BC-2.19.012 | 3 | `AC-005-006-stopdt-t0881.md` | PASS |
| AC-168-007 | TESTFR-act/con produce no finding; session state unchanged | BC-2.19.013 | 3 | `AC-007-testfr-no-finding.md` | PASS |
| AC-168-008 | Non-canonical U-frame CF1 emits T0814 Possible (CVE-2026-1773) | BC-2.19.014 | 6 | `AC-008-non-canonical-u-t0814.md` | PASS |
| AC-168-009 | VP-046 proptest skeleton compiles; classify_frame_format totality | VP-046 | 1 | `AC-009-vp046-proptest-totality.md` | PASS |

**Total STORY-168 test-based coverage: 34/34 (all AC-168-001..009)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Test Names |
|----|-----|-----------|------------|
| AC-168-001 | BC-2.19.007 | 5 | returns_iformat_for_cf1_0x00_canonical_vector, returns_iformat_for_cf1_0x02_canonical_vector, returns_iformat_for_cf1_0x7E_canonical_vector, returns_iformat_for_cf1_0xFE_all_even_bits_set, invariant_all_128_even_cf1_values_return_iformat |
| AC-168-002 | BC-2.19.008 | 4 | returns_sformat_for_cf1_0x01_canonical_vector, returns_sformat_for_cf1_0x05_canonical_vector, does_not_return_sformat_for_cf1_0x03_uformat, invariant_all_64_cf1_values_bits1_0_0b01_return_sformat |
| AC-168-003 | BC-2.19.009 | 7 | returns_uformat_for_cf1_0x07_startdt_act_canonical_vector, returns_uformat_for_cf1_0x0B_startdt_con_canonical_vector, returns_uformat_for_cf1_0x13_stopdt_act_canonical_vector, returns_uformat_for_cf1_0x03_non_canonical_canonical_vector, returns_uformat_for_cf1_0xFF_canonical_vector, invariant_all_64_cf1_values_bits1_0_0b11_return_uformat, invariant_vp046_totality_exhaustive_all_256_values |
| AC-168-004 | BC-2.19.010 | 3 | startdt_act_sets_session_started_true_on_fresh_flow, startdt_act_idempotent_when_already_started, startdt_con_sets_session_started_true_on_fresh_flow |
| AC-168-005 | BC-2.19.011 | 2 | stopdt_act_after_startdt_emits_t0881_possible, stopdt_act_followed_by_startdt_act_restarts_session |
| AC-168-006 | BC-2.19.012 | 3 | stopdt_act_without_startdt_emits_t0881_likely, invariant_stopdt_confidence_escalation_likely_vs_possible, stopdt_con_sets_session_false_no_finding_act_only_mvp |
| AC-168-007 | BC-2.19.013 | 3 | testfr_act_emits_no_finding_canonical_vector, testfr_con_emits_no_finding_canonical_vector, invariant_testfr_does_not_modify_session_started |
| AC-168-008 | BC-2.19.014 | 6 | non_canonical_cf1_0x03_emits_t0814_possible, non_canonical_cf1_0x0F_emits_t0814_possible_canonical_vector, non_canonical_cf1_0x1B_emits_t0814_possible, non_canonical_cf1_0xFF_emits_t0814_possible_canonical_vector, negative_canonical_cf1_values_do_not_emit_t0814, invariant_non_canonical_u_frame_does_not_advance_session_state |
| AC-168-009 | VP-046 | 1 | proptest_vp046_frame_format_totality |

---

## VP-046 Totality Evidence Summary

**Method:** Two complementary approaches:
1. **Exhaustive-256 unit test** (`test_BC_2_19_009_invariant_vp046_totality_exhaustive_all_256_values`): deterministic loop over all 256 u8 values; partition assertion by `cf1 & 0x03`
2. **Proptest** (`proptest_vp046_frame_format_totality`): property-based random strategy `0u8..=255u8` with shrinking; same partition assertion

Both pass. Full Kani formal verification and extended proptest run: STORY-174.

**Proptest harness location:** `tests/iec104_analyzer_tests.rs`, line 1508  
**Source function:** `src/analyzer/iec104.rs`, line 206 (`classify_frame_format`)

---

## Source-Level Evidence

**`FrameFormat` enum:** `src/analyzer/iec104.rs`, line 106  
**`classify_frame_format` function:** `src/analyzer/iec104.rs`, line 206 (pure-core, VP-046 target)  
**`process_u_frame` function:** `src/analyzer/iec104.rs`, line 256 (effectful, emits T0881/T0814)  
**`Iec104FlowState::session_started` field:** `src/analyzer/iec104.rs`, line 174  

Presence confirmed via:
```
grep -n "pub fn classify_frame_format\|pub fn process_u_frame\|pub session_started\|pub enum FrameFormat" src/analyzer/iec104.rs
```
Output:
```
106:pub enum FrameFormat {
174:    pub session_started: bool,
206:pub fn classify_frame_format(cf1: u8) -> FrameFormat {
256:pub fn process_u_frame(state: &mut Iec104FlowState, cf1: u8) -> Option<Finding> {
```

---

## Finding Content Reference

### T0881 "Service Stop" (STOPDT-act)

| Field | Value |
|-------|-------|
| `category` | `ThreatCategory::Impact` |
| `verdict` (session active) | `Verdict::Possible` |
| `verdict` (no prior STARTDT) | `Verdict::Likely` |
| `confidence` | `Confidence::Medium` |
| `mitre_techniques` | `["T0881"]` |
| `summary` | `"IEC-104 STOPDT-act received: CF1=0x13 — ICS data-transfer service stop request observed (T0881 inhibit-response-function; BC-2.19.011/012)"` |
| evidence (Likely path only) | adds `"STOPDT received without prior STARTDT on this flow"` |

### T0814 "Denial of Service" (non-canonical U-frame, CVE-2026-1773)

| Field | Value |
|-------|-------|
| `category` | `ThreatCategory::Anomaly` |
| `verdict` | `Verdict::Possible` |
| `confidence` | `Confidence::Medium` |
| `mitre_techniques` | `["T0814"]` |
| `summary` | `"IEC-104 non-canonical U-frame CF1=0xNN: CF1 bits1:0=0b11 but not in canonical set {0x07,0x0B,0x13,0x23,0x43,0x83} — potential CVE-2026-1773 denial-of-service attack (T0814; BC-2.19.014)"` |

---

## Recording Method

This is a pure-core library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Inline canonical vector tables from the BC specifications
- Source-level grep verification for public function and field presence

VHS/Playwright recordings are not applicable (no interactive surface at this story scope;
dispatch wiring to the CLI analyzer is STORY-173).

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-002-003-frame-discrimination.md` | AC-168-001 (BC-2.19.007), AC-168-002 (BC-2.19.008), AC-168-003 (BC-2.19.009) |
| `AC-004-startdt-session-state.md` | AC-168-004 (BC-2.19.010) |
| `AC-005-006-stopdt-t0881.md` | AC-168-005 (BC-2.19.011), AC-168-006 (BC-2.19.012) |
| `AC-007-testfr-no-finding.md` | AC-168-007 (BC-2.19.013) |
| `AC-008-non-canonical-u-t0814.md` | AC-168-008 (BC-2.19.014) |
| `AC-009-vp046-proptest-totality.md` | AC-168-009 (VP-046) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute host-local paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-14).
