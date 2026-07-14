# Evidence Report — STORY-169

**Story:** STORY-169: IEC-104 ASDU Header Extraction: parse_asdu / Asdu with Broken-Out DUI Fields  
**Wave:** 78  
**Date:** 2026-07-14  
**Branch:** feature/STORY-169-iec104-asdu-extraction  
**Product type:** Library (pure-core free function + data struct — no CLI/web surface; effectful caller is STORY-170)

---

## Full Test Suite: 91/91 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 91 tests
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_one_byte ... ok
test story_167::test_BC_2_19_001_returns_none_for_two_bytes ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x69_off_by_one ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0xFF_canonical_vector ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_3_off_by_one_canonical_vector ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_zero_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_254_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_255_canonical_vector ... ok
test story_167::test_BC_2_19_005_apci_header_equality_and_field_layout ... ok
test story_167::test_BC_2_19_005_cf_fields_verbatim_from_data_indices_2_through_5 ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_1_and_len_2 ... ok
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
test story_169::test_BC_2_19_015_invariant_no_panic_on_all_short_lengths ... ok
test story_169::test_BC_2_19_015_invariant_parse_asdu_pure_deterministic ... ok
test story_169::test_BC_2_19_015_returns_none_for_empty_body ... ok
test story_169::test_BC_2_19_015_returns_none_for_five_bytes_canonical_vector ... ok
test story_169::test_BC_2_19_015_returns_some_for_exactly_six_bytes_minimum_valid ... ok
test story_169::test_BC_2_19_016_type_id_0_undefined_passthrough_canonical_vector ... ok
test story_169::test_BC_2_19_016_type_id_255_vsq_0x80_sq_true_count_0_canonical_vector ... ok
test story_169::test_BC_2_19_016_type_id_45_c_sc_na_1_canonical_vector ... ok
test story_169::test_BC_2_19_016_type_id_extracted_verbatim_from_byte_0 ... ok
test story_169::test_BC_2_19_016_vsq_0x03_sq_false_count_3 ... ok
test story_169::test_BC_2_19_016_vsq_0x7F_sq_false_count_127_max ... ok
test story_169::test_BC_2_19_016_vsq_0x81_sq_true_count_1 ... ok
test story_169::test_BC_2_19_017_cot_all_bits_byte2_0xC6_byte3_0x01_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_cause_6_activation_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_cause_max_63_byte2_0x3F_byte3_0xFF_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_originator_verbatim_from_byte_3 ... ok
test story_169::test_BC_2_19_017_cot_pn_true_byte2_0x46_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_test_true_byte2_0x86_canonical_vector ... ok
test story_169::test_BC_2_19_018_casdu_0_undefined_extracted_without_rejection ... ok
test story_169::test_BC_2_19_018_casdu_little_endian_1_canonical_vector ... ok
test story_169::test_BC_2_19_018_casdu_max_65535_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_le_byte_order_verified ... ok
test story_169::test_BC_2_19_018_first_ioa_max_0xFFFFFF_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_7_or_8_bytes_count_gt_0 ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_count_0_regardless_of_length ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_exactly_6_bytes_count_gt_0 ... ok
test story_169::test_BC_2_19_018_first_ioa_some_count_1_len_9_canonical_vector ... ok
test story_168::proptest_vp046_frame_format_totality ... ok

test result: ok. 91 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**STORY-167 contribution:** 30 tests (story_167 module)  
**STORY-168 contribution:** 34 tests (story_168 module, includes proptest)  
**STORY-169 contribution:** 27 tests (story_169 module)  
**Total:** 91/91 PASS

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-169-001 | `parse_asdu(<6 bytes)→None`; `==6`→`Some(first_ioa=None)`; no panic; deterministic | BC-2.19.015 | 5 | `AC-001-min6-guard.md` | PASS |
| AC-169-002 | TypeID verbatim from byte 0; `sq` = bit7 of VSQ; `count` = bits6:0 of VSQ | BC-2.19.016 | 7 | `AC-002-typeid-vsq.md` | PASS |
| AC-169-003 | `cot_cause`=bits5:0; `cot_pn`=bit6; `cot_test`=bit7; `cot_originator`=byte3 | BC-2.19.017 | 6 | `AC-003-cot-fields.md` | PASS |
| AC-169-004 | `casdu = u16::from_le_bytes([byte4, byte5])` | BC-2.19.018 PC1 | 3 | `AC-004-005-casdu-first-ioa.md` | PASS |
| AC-169-005 | `first_ioa=Some(24-bit LE)` when count>0 AND len>=9; `None` otherwise | BC-2.19.018 PC2-PC3 | 6 | `AC-004-005-casdu-first-ioa.md` | PASS |
| AC-169-006 | `parse_asdu` is pure: no findings, no state mutation, no I/O | BC-2.19.015 inv 2; ADR-013 §D8 | structural + det. test | `AC-006-purity.md` | PASS |

**Total STORY-169 test-based coverage: 27/27 (all AC-169-001..006)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Test Names |
|----|-----|-----------|------------|
| AC-169-001 | BC-2.19.015 | 5 | returns_none_for_empty_body, returns_none_for_five_bytes_canonical_vector, returns_some_for_exactly_six_bytes_minimum_valid, invariant_no_panic_on_all_short_lengths, invariant_parse_asdu_pure_deterministic |
| AC-169-002 | BC-2.19.016 | 7 | type_id_45_c_sc_na_1_canonical_vector, type_id_0_undefined_passthrough_canonical_vector, type_id_255_vsq_0x80_sq_true_count_0_canonical_vector, type_id_extracted_verbatim_from_byte_0, vsq_0x81_sq_true_count_1, vsq_0x03_sq_false_count_3, vsq_0x7F_sq_false_count_127_max |
| AC-169-003 | BC-2.19.017 | 6 | cot_cause_6_activation_canonical_vector, cot_cause_max_63_byte2_0x3F_byte3_0xFF_canonical_vector, cot_pn_true_byte2_0x46_canonical_vector, cot_test_true_byte2_0x86_canonical_vector, cot_originator_verbatim_from_byte_3, cot_all_bits_byte2_0xC6_byte3_0x01_canonical_vector |
| AC-169-004 | BC-2.19.018 PC1 | 3 | casdu_little_endian_1_canonical_vector, casdu_max_65535_canonical_vector, casdu_0_undefined_extracted_without_rejection |
| AC-169-005 | BC-2.19.018 PC2-PC3 | 6 | first_ioa_some_count_1_len_9_canonical_vector, first_ioa_max_0xFFFFFF_canonical_vector, first_ioa_le_byte_order_verified, first_ioa_none_when_exactly_6_bytes_count_gt_0, first_ioa_none_when_7_or_8_bytes_count_gt_0, first_ioa_none_when_count_0_regardless_of_length |
| AC-169-006 | BC-2.19.015 inv2; ADR-013 | structural | (covered by invariant_parse_asdu_pure_deterministic in AC-169-001 group) |

---

## Source-Level Evidence

**`Asdu` struct:** `src/analyzer/iec104.rs`, line 468  
**`parse_asdu` function:** `src/analyzer/iec104.rs`, line 554 (pure-core, `fn(&[u8]) -> Option<Asdu>`)

Presence confirmed via:
```
grep -n "pub fn parse_asdu\|pub struct Asdu\|pub first_ioa" src/analyzer/iec104.rs
```
Output:
```
468:pub struct Asdu {
506:    pub first_ioa: Option<u32>,
554:pub fn parse_asdu(asdu_body: &[u8]) -> Option<Asdu> {
```

---

## Edge Case Coverage Summary

| Edge Case | BC | Test Covering | Verdict |
|-----------|-----|--------------|---------|
| EC-001: body 5 bytes (1 short of minimum) | BC-2.19.015 | `returns_none_for_five_bytes_canonical_vector` | PASS |
| EC-002: body exactly 6 bytes (min valid) | BC-2.19.015 | `returns_some_for_exactly_six_bytes_minimum_valid` | PASS |
| EC-003: 6-8 bytes, count>0 → first_ioa=None | BC-2.19.018 | `first_ioa_none_when_exactly_6_bytes_count_gt_0`, `first_ioa_none_when_7_or_8_bytes_count_gt_0` | PASS |
| EC-004: 9+ bytes, count>0 → first_ioa=Some | BC-2.19.018 | `first_ioa_some_count_1_len_9_canonical_vector` | PASS |
| EC-005: count=0 regardless of length → None | BC-2.19.018 | `first_ioa_none_when_count_0_regardless_of_length` | PASS |
| EC-006: IOA = 0xFFFFFF max 24-bit | BC-2.19.018 | `first_ioa_max_0xFFFFFF_canonical_vector` | PASS |
| EC-007: TypeID=0 undefined passthrough | BC-2.19.016 | `type_id_0_undefined_passthrough_canonical_vector` | PASS |
| EC-008: T-bit set (cot_test=true) | BC-2.19.017 | `cot_test_true_byte2_0x86_canonical_vector` | PASS |

---

## Recording Method

This is a pure-core library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Inline canonical vector tables from the BC specifications (BC-2.19.015–018)
- Source-level grep verification for public function and struct presence

VHS/Playwright recordings are not applicable (no interactive surface at this story scope;
effectful dispatch to the CLI analyzer is STORY-170).

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-min6-guard.md` | AC-169-001 (BC-2.19.015) |
| `AC-002-typeid-vsq.md` | AC-169-002 (BC-2.19.016) |
| `AC-003-cot-fields.md` | AC-169-003 (BC-2.19.017) |
| `AC-004-005-casdu-first-ioa.md` | AC-169-004 (BC-2.19.018 PC1), AC-169-005 (BC-2.19.018 PC2-PC3) |
| `AC-006-purity.md` | AC-169-006 (BC-2.19.015 inv2; ADR-013 D8) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute host-local paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-14).
