# Evidence Report — STORY-170

**Story:** STORY-170: IEC-104 Control Command Detection: TypeIDs 45–51, C_RP, Interrogation, Reserved TypeIDs  
**Wave:** 79  
**Date:** 2026-07-14  
**Branch:** feature/STORY-170-iec104-control-command-detection  
**Product type:** Library (effectful detection function — no CLI/web surface; dispatch wiring is STORY-173)

---

## Full Test Suite: 136/136 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-70fabb01cb3ba763)

running 136 tests
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_two_bytes ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_one_byte ... ok
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
test story_168::test_BC_2_19_007_returns_iformat_for_cf1_0x00_canonical_vector ... ok
test story_168::test_BC_2_19_007_invariant_all_128_even_cf1_values_return_iformat ... ok
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
test story_168::test_BC_2_19_010_startdt_con_sets_session_started_true_on_fresh_flow ... ok
test story_168::test_BC_2_19_012_invariant_stopdt_confidence_escalation_likely_vs_possible ... ok
test story_168::test_BC_2_19_011_stopdt_act_after_startdt_emits_t0881_possible ... ok
test story_168::test_BC_2_19_012_stopdt_con_sets_session_false_no_finding_act_only_mvp ... ok
test story_168::test_BC_2_19_013_invariant_testfr_does_not_modify_session_started ... ok
test story_168::test_BC_2_19_012_stopdt_act_without_startdt_emits_t0881_likely ... ok
test story_168::test_BC_2_19_013_testfr_act_emits_no_finding_canonical_vector ... ok
test story_168::test_BC_2_19_013_testfr_con_emits_no_finding_canonical_vector ... ok
test story_168::test_BC_2_19_010_startdt_act_sets_session_started_true_on_fresh_flow ... ok
test story_168::test_BC_2_19_010_startdt_act_idempotent_when_already_started ... ok
test story_168::test_BC_2_19_011_stopdt_act_followed_by_startdt_act_restarts_session ... ok
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
test story_169::test_BC_2_19_016_vsq_0x81_sq_true_count_1 ... ok
test story_169::test_BC_2_19_017_cot_all_bits_byte2_0xC6_byte3_0x01_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_cause_6_activation_canonical_vector ... ok
test story_169::test_BC_2_19_016_vsq_0x7F_sq_false_count_127_max ... ok
test story_169::test_BC_2_19_017_cot_cause_max_63_byte2_0x3F_byte3_0xFF_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_originator_verbatim_from_byte_3 ... ok
test story_169::test_BC_2_19_017_cot_pn_true_byte2_0x46_canonical_vector ... ok
test story_169::test_BC_2_19_017_cot_test_true_byte2_0x86_canonical_vector ... ok
test story_169::test_BC_2_19_018_casdu_0_undefined_extracted_without_rejection ... ok
test story_169::test_BC_2_19_018_casdu_little_endian_1_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_le_byte_order_verified ... ok
test story_169::test_BC_2_19_018_casdu_max_65535_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_max_0xFFFFFF_canonical_vector ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_7_or_8_bytes_count_gt_0 ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_exactly_6_bytes_count_gt_0 ... ok
test story_169::test_BC_2_19_018_first_ioa_none_when_count_0_regardless_of_length ... ok
test story_169::test_BC_2_19_018_first_ioa_some_count_1_len_9_canonical_vector ... ok
test story_170::test_BC_2_19_017_cot_test_false_control_command_summary_has_no_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_control_command_summary_has_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_setpoint_both_findings_have_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_t0814_reserved_type_summary_has_test_tag ... ok
test story_170::test_BC_2_19_017_cot_test_true_t0827_summary_has_test_tag ... ok
test story_170::test_BC_2_19_017_invariant_cot_test_false_never_adds_test_tag ... ok
test story_170::test_BC_2_19_017_start_idx_guard_preexisting_finding_not_tagged ... ok
test story_170::test_BC_2_19_019_invariant_all_findings_have_possible_verdict ... ok
test story_170::test_BC_2_19_019_invariant_setpoint_types_48_to_51_each_emit_two_findings ... ok
test story_170::test_BC_2_19_019_invariant_switching_types_45_to_47_each_emit_one_finding ... ok
test story_170::test_BC_2_19_019_invariant_t1692001_present_for_all_types_45_to_51 ... ok
test story_170::test_BC_2_19_019_type45_c_sc_na1_emits_exactly_one_finding ... ok
test story_170::test_BC_2_19_019_type45_does_not_emit_t0836 ... ok
test story_170::test_BC_2_19_019_type45_emits_t1692001_possible_impact ... ok
test story_170::test_BC_2_19_019_type46_c_dc_na1_emits_t1692001_only ... ok
test story_170::test_BC_2_19_019_type47_c_rc_na1_emits_t1692001_only ... ok
test story_170::test_BC_2_19_019_type48_c_se_na1_emits_exactly_two_findings ... ok
test story_170::test_BC_2_19_019_type48_emits_t0836_possible ... ok
test story_170::test_BC_2_19_019_type48_emits_t1692001_possible ... ok
test story_170::test_BC_2_19_019_type49_c_se_nb1_emits_t1692001_and_t0836 ... ok
test story_170::test_BC_2_19_019_type50_c_se_nc1_emits_t1692001_and_t0836 ... ok
test story_170::test_BC_2_19_020_type105_c_rp_na1_emits_exactly_one_finding ... ok
test story_170::test_BC_2_19_019_type51_c_bo_na1_emits_exactly_two_findings_canonical_vector ... ok
test story_170::test_BC_2_19_020_type105_category_is_impact ... ok
test story_170::test_BC_2_19_020_type105_does_not_emit_t1692001 ... ok
test story_170::test_BC_2_19_020_type105_emits_t0827_likely_canonical_vector ... ok
test story_170::test_BC_2_19_020_type105_verdict_is_likely_not_possible ... ok
test story_170::test_BC_2_19_021_invariant_all_interrogation_types_emit_no_finding ... ok
test story_170::test_BC_2_19_021_type100_c_ic_emits_no_finding_canonical_vector ... ok
test story_170::test_BC_2_19_021_type101_c_ci_emits_no_finding_canonical_vector ... ok
test story_170::test_BC_2_19_021_type103_c_cs_emits_no_finding_canonical_vector ... ok
test story_170::test_BC_2_19_022_invariant_silent_range_sample_emits_no_findings ... ok
test story_170::test_BC_2_19_022_type0_undefined_emits_t0814_anomaly_canonical_vector ... ok
test story_170::test_BC_2_19_022_type102_c_rd_not_in_detection_set_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type104_defined_unhandled_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type127_max_defined_unhandled_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type128_emits_t0814_possible_canonical_vector ... ok
test story_170::test_BC_2_19_022_type1_minimum_defined_unhandled_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type200_private_use_emits_t0814_possible ... ok
test story_170::test_BC_2_19_022_type255_emits_t0814_possible_canonical_vector ... ok
test story_170::test_BC_2_19_022_type44_max_monitoring_emits_no_finding_canonical_vector ... ok
test story_170::test_BC_2_19_022_type52_reserved_above_control_range_emits_no_finding ... ok
test story_170::test_BC_2_19_022_type99_defined_unhandled_emits_no_finding ... ok
test story_170::test_F_170_001_casdu_appears_in_finding_evidence_for_control_type ... ok
test story_170::test_F_170_001_first_ioa_appears_in_finding_evidence_when_some ... ok
test story_168::proptest_vp046_frame_format_totality ... ok

test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**STORY-167 contribution:** 30 tests (story_167 module)  
**STORY-168 contribution:** 34 tests (story_168 module, includes proptest)  
**STORY-169 contribution:** 27 tests (story_169 module)  
**STORY-170 contribution:** 45 tests (story_170 module)  
**Total:** 136/136 PASS

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-170-001 | Control TypeIDs 45–51: T1692.001 for all; T0836 also for 48–51; CASDU/IOA evidence | BC-2.19.019 PC1-PC3; inv2 | 17 (15+2) | `AC-001-control-typeid-detection.md` | PASS |
| AC-170-002 | C_RP TypeID 105 emits T0827 Likely (not Possible) | BC-2.19.020 PC1-PC2; inv1 | 5 | `AC-002-c_rp-reset-detection.md` | PASS |
| AC-170-003 | Interrogation TypeIDs 100/101/103 produce no finding (false-positive prevention) | BC-2.19.021 PC1-PC3; inv1 | 4 | `AC-003-interrogation-no-finding.md` | PASS |
| AC-170-004 | TypeID=0 or [128, 255] emits T0814 Possible | BC-2.19.022 PC2 | 4 (within 12) | `AC-004-005-006-typeid-dispatch.md` | PASS |
| AC-170-005 | Defined-but-unhandled TypeIDs in [1, 127] produce no finding | BC-2.19.022 inv1 | 8 (within 12) | `AC-004-005-006-typeid-dispatch.md` | PASS |
| AC-170-006 | TypeID dispatch exhaustive: every TypeID (0–255) produces exactly one outcome | BC-2.19.022 inv1 | covered by 12 | `AC-004-005-006-typeid-dispatch.md` | PASS |
| AC-170-007 | cot_test=true tags finding summary with " [TEST]"; cot_test=false does not | BC-2.19.017 inv1 | 7 | `AC-007-cot-test-tagging.md` | PASS |

**Total STORY-170 test-based coverage: 45/45 (all AC-170-001..007)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Module | Key Test Names |
|----|-----|-----------|--------|----------------|
| AC-170-001 | BC-2.19.019 PC1-PC3 | 15 + 2 F_170 = 17 | story_170 | type45/46/47_emits_t1692001_only; type48/49/50/51_emits_t1692001_and_t0836; invariant_switching_types_one_finding; invariant_setpoint_types_two_findings; F_170_casdu; F_170_first_ioa |
| AC-170-002 | BC-2.19.020 PC1-PC2 | 5 | story_170 | type105_emits_t0827_likely_canonical_vector; type105_verdict_is_likely_not_possible; type105_does_not_emit_t1692001; type105_category_is_impact; type105_c_rp_na1_emits_exactly_one_finding |
| AC-170-003 | BC-2.19.021 PC1 | 4 | story_170 | type100_c_ic_emits_no_finding; type101_c_ci_emits_no_finding; type103_c_cs_emits_no_finding; invariant_all_interrogation_types_emit_no_finding |
| AC-170-004/005/006 | BC-2.19.022 PC2; inv1 | 12 | story_170 | type0_emits_t0814; type128/200/255_emits_t0814; type1/44/52/99/102/104/127_no_finding; invariant_silent_range_sample |
| AC-170-007 | BC-2.19.017 inv1 | 7 | story_170 | cot_test_true_control_command_summary_has_test_tag; cot_test_false_no_test_tag; t0827_summary_has_test_tag; setpoint_both_findings_have_test_tag; t0814_summary_has_test_tag; invariant_false_never_adds_test_tag; start_idx_guard |

---

## Dispatch Table Summary (AC-170-006)

| TypeID Range | Finding(s) Emitted | MITRE | Verdict |
|-------------|-------------------|-------|---------|
| 0 | T0814 "Denial of Service" | T0814 | Possible |
| 1–44 | None (monitoring direction) | — | — |
| 45–47 | T1692.001 "Command Message" | T1692.001 | Possible |
| 48–51 | T1692.001 "Command Message" + T0836 "Modify Parameter" | T1692.001, T0836 | Possible |
| 52–99 | None (defined-but-unhandled) | — | — |
| 100, 101, 103 | None (interrogation/clock-sync benign) | — | — |
| 102, 104, 106–127 | None (defined-but-unhandled) | — | — |
| 105 | T0827 "Loss of Control" | T0827 | **Likely** |
| 128–255 | T0814 "Denial of Service" | T0814 | Possible |

---

## Source-Level Evidence

**`detect_iec104_threats` function:** `src/analyzer/iec104.rs`

Presence confirmed via:
```
grep -n "pub fn detect_iec104_threats\|fn detect_iec104_threats" src/analyzer/iec104.rs
```

---

## Edge Case Coverage Summary

| Edge Case | BC | Test Covering | Verdict |
|-----------|-----|--------------|---------|
| EC-001 (BC-019): TypeID=45 min switching | BC-2.19.019 | `type45_emits_t1692001_possible_impact` | PASS |
| EC-002 (STORY-170): TypeID=44 max monitoring, no finding | BC-2.19.022 | `type44_max_monitoring_emits_no_finding_canonical_vector` | PASS |
| EC-003 (STORY-170): TypeID=45 min switching → T1692.001 only | BC-2.19.019 | `type45_c_sc_na1_emits_exactly_one_finding`, `type45_does_not_emit_t0836` | PASS |
| EC-004 (STORY-170): TypeID=51 max control → T1692.001+T0836 | BC-2.19.019 | `type51_c_bo_na1_emits_exactly_two_findings_canonical_vector` | PASS |
| EC-005 (STORY-170): TypeID=52 above control range → no finding | BC-2.19.022 | `type52_reserved_above_control_range_emits_no_finding` | PASS |
| EC-006 (STORY-170): TypeID=105 → T0827 Likely (not Possible) | BC-2.19.020 | `type105_verdict_is_likely_not_possible` | PASS |
| EC-007 (STORY-170): TypeID=102 (C_RD) defined-unhandled → no finding | BC-2.19.022 | `type102_c_rd_not_in_detection_set_emits_no_finding` | PASS |
| EC-008 (STORY-170): TypeID=255 max reserved → T0814 | BC-2.19.022 | `type255_emits_t0814_possible_canonical_vector` | PASS |
| EC-009 (STORY-170): cot_test=true TypeID=45 → [TEST] tag | BC-2.19.017 | `cot_test_true_control_command_summary_has_test_tag` | PASS |

---

## Recording Method

This is an effectful library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Inline dispatch table and edge-case coverage tables
- Source-level verification of `detect_iec104_threats` function presence

VHS/Playwright recordings are not applicable (no interactive surface at this story scope;
dispatch wiring to the CLI analyzer is STORY-173).

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-control-typeid-detection.md` | AC-170-001 (BC-2.19.019 PC1-PC3) |
| `AC-002-c_rp-reset-detection.md` | AC-170-002 (BC-2.19.020) |
| `AC-003-interrogation-no-finding.md` | AC-170-003 (BC-2.19.021) |
| `AC-004-005-006-typeid-dispatch.md` | AC-170-004 (BC-2.19.022 PC2), AC-170-005 (BC-2.19.022 inv1), AC-170-006 |
| `AC-007-cot-test-tagging.md` | AC-170-007 (BC-2.19.017 inv1) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute host-local paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-14).
