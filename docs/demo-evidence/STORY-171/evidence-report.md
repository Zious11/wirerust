# Evidence Report — STORY-171

**Story:** STORY-171: IEC-104 N(S)/N(R) Sequence Tracking: Option<u16> First-Frame Guard + Desync Detection  
**Wave:** 80  
**Date:** 2026-07-15  
**Branch:** feature/STORY-171-iec104-nsnr-tracking-desync  
**Product type:** Library (pure extraction functions + effectful gap detector — no CLI/web surface; dispatch wiring is STORY-173)

---

## Full Test Suite: 166/166 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 166 tests
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_one_byte ... ok
test story_167::test_BC_2_19_001_returns_none_for_two_bytes ... ok
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
test story_170::test_BC_2_19_019_type51_c_bo_na1_emits_exactly_two_findings_canonical_vector ... ok
test story_170::test_BC_2_19_020_type105_c_rp_na1_emits_exactly_one_finding ... ok
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
test story_171::test_BC_2_19_023_extract_nr_cf3_0x00_cf4_0x00_returns_0 ... ok
test story_171::test_BC_2_19_023_extract_nr_cf3_0x02_cf4_0x00_returns_1 ... ok
test story_171::test_BC_2_19_023_extract_nr_cf3_0xFE_cf4_0xFF_returns_32767 ... ok
test story_171::test_BC_2_19_023_extract_nr_is_transient_no_last_nr_field_in_flow_state ... ok
test story_171::test_BC_2_19_023_extract_nr_symmetric_formula_equal_inputs_equal_outputs ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x00_returns_0 ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0x00_cf2_0x80_returns_16384 ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0x02_cf2_0x00_returns_1 ... ok
test story_171::test_BC_2_19_023_extract_ns_cf1_0xFE_cf2_0xFF_returns_32767 ... ok
test story_171::test_BC_2_19_023_invariant_extract_ns_range_and_exact_values_boundary_inputs ... ok
test story_171::test_BC_2_19_023_proptest_extract_ns_always_in_15bit_range ... ok
test story_171::test_BC_2_19_023_proptest_extract_nr_always_in_15bit_range ... ok
test story_171::test_BC_2_19_024_ac171_006_wrap_32767_to_0_gap_1_no_finding ... ok
test story_171::test_BC_2_19_024_ac171_006_wrap_32767_to_1_gap_2_no_finding ... ok
test story_171::test_BC_2_19_024_ac171_007_c2s_call_updates_c2s_not_s2c ... ok
test story_171::test_BC_2_19_024_ac171_007_interleaved_c2s_s2c_independent_baselines_and_gaps ... ok
test story_171::test_BC_2_19_024_ac171_007_s2c_call_updates_s2c_not_c2s ... ok
test story_171::test_BC_2_19_024_ec_006_mid_capture_three_frame_sequence_exercises_all_three_paths ... ok
test story_171::test_BC_2_19_024_path_a_first_frame_c2s_ns_0_no_finding_state_becomes_some_0 ... ok
test story_171::test_BC_2_19_024_path_a_first_frame_s2c_ns_0_no_finding_state_becomes_some_0 ... ok
test story_171::test_BC_2_19_024_path_a_mid_capture_first_frame_c2s_ns_5000_no_finding_state_becomes_some_5000 ... ok
test story_171::test_BC_2_19_024_path_b_gap_0_same_ns_no_finding ... ok
test story_171::test_BC_2_19_024_path_b_gap_1_no_finding_state_updates_to_current_ns ... ok
test story_171::test_BC_2_19_024_path_b_gap_12_exactly_k_boundary_no_finding ... ok
test story_171::test_BC_2_19_024_path_c_canonical_table_row8_prev_100_current_114_gap_14_emits_finding ... ok
test story_171::test_BC_2_19_024_path_c_ec005_gap_32767_massive_jump_emits_t1692_001_possible ... ok
test story_171::test_BC_2_19_024_path_c_gap_13_k_plus_1_emits_t1692_001_possible ... ok
test story_171::test_BC_2_19_024_path_c_gap_19_canonical_vector_prev_5001_current_5020_emits_t1692_001 ... ok
test story_171::test_BC_2_19_024_path_c_state_updates_to_current_ns_after_finding_emitted ... ok
test story_171::test_RETRANSMIT_NS_FALSEPOS_001_backwards_ns_yields_large_gap_emits_t1692_001_finding ... ok
test story_168::proptest_vp046_frame_format_totality ... ok

test result: ok. 166 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

**STORY-167 contribution:** 30 tests (story_167 module)  
**STORY-168 contribution:** 34 tests (story_168 module, includes proptest)  
**STORY-169 contribution:** 27 tests (story_169 module)  
**STORY-170 contribution:** 45 tests (story_170 module)  
**STORY-171 contribution:** 30 tests (story_171 module)  
**Total:** 166/166 PASS

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-171-001 | `extract_ns(cf1, cf2)` 15-bit formula; canonical vectors 0x02/0x00→1, 0xFE/0xFF→32767, 0x00/0x80→16384 | BC-2.19.023 PC1, PC3 | 6 | `AC-001-002-ns-nr-extraction.md` | PASS |
| AC-171-002 | `extract_nr(cf3, cf4)` symmetric formula; N(R) transient (no `last_nr` field) | BC-2.19.023 PC2, PC4 | 6 | `AC-001-002-ns-nr-extraction.md` | PASS |
| AC-171-003 | First I-frame `None → Some(ns)` with no finding (mid-capture guard) | BC-2.19.024 Path A; inv3 | 3 | `AC-003-first-frame-option-guard.md` | PASS |
| AC-171-004 | Gap ≤ k=12: no finding; state updates | BC-2.19.024 Path B | 3 | `AC-004-gap-le-k12-no-finding.md` | PASS |
| AC-171-005 | Gap > k=12: T1692.001 Possible with current/prev/gap evidence | BC-2.19.024 Path C; inv1 | 5 | `AC-005-gap-gt-k12-desync-finding.md` | PASS |
| AC-171-006 | Wrap: Some(32767)→current=1 → gap=2 via `wrapping_sub & 0x7FFF`; no false positive | BC-2.19.024 inv1 (15-bit modular) | 2 | `AC-006-wrap-arithmetic.md` | PASS |
| AC-171-007 | C2S and S2C tracked independently; no cross-direction mixing | BC-2.19.023 PC3 (direction param) | 3 | `AC-007-directional-isolation.md` | PASS |
| RETRANSMIT-NS-FALSEPOS-001 | Backwards N(S) yields large gap → T1692.001 Possible (intentional fail-closed) | BC-2.19.024 inv3 (EC-007) | 1 | `RETRANSMIT-NS-FALSEPOS-001.md` | PASS (intended) |
| EC-006 three-path sequence | Mid-capture sequence exercises all three paths A/B/C in sequence | BC-2.19.024 | 1 | (inline in coverage below) | PASS |

**Total STORY-171 test-based coverage: 30/30 (all AC-171-001..007 + RETRANSMIT + EC-006)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Path | Key Test Names |
|----|-----|-----------|------|----------------|
| AC-171-001 | BC-2.19.023 PC1, PC3 | 6 | extract_ns | cf1_0x02→1; cf1_0xFE_cf2_0xFF→32767; cf1_0x00_cf2_0x00→0; cf1_0x00_cf2_0x80→16384; invariant_range; proptest_15bit |
| AC-171-002 | BC-2.19.023 PC2, PC4 | 6 | extract_nr | cf3_0x02→1; cf3_0xFE→32767; cf3_0x00→0; symmetric; transient_no_field; proptest_15bit |
| AC-171-003 | BC-2.19.024 Path A | 3 | first-frame | path_a_c2s_ns_0; path_a_s2c_ns_0; path_a_mid_capture_ns_5000 |
| AC-171-004 | BC-2.19.024 Path B | 3 | gap ≤ 12 | path_b_gap_0; path_b_gap_1; path_b_gap_12_boundary |
| AC-171-005 | BC-2.19.024 Path C | 5 | gap > 12 | path_c_gap_13_k_plus_1; path_c_gap_19_canonical; path_c_row8_gap_14; path_c_gap_32767; path_c_state_updates |
| AC-171-006 | BC-2.19.024 inv1 | 2 | wrap | ac171_006_wrap_32767_to_1_gap_2; ac171_006_wrap_32767_to_0_gap_1 |
| AC-171-007 | BC-2.19.023 PC3 | 3 | direction | ac171_007_c2s_not_s2c; ac171_007_s2c_not_c2s; ac171_007_interleaved |
| RETRANSMIT-NS-FALSEPOS-001 | BC-2.19.024 inv3 | 1 | fail-closed | RETRANSMIT_NS_FALSEPOS_001_backwards_ns |
| EC-006 three-path | BC-2.19.024 | 1 | multi-path | ec_006_mid_capture_three_frame_sequence |

---

## Gap Detection Logic Summary (AC-171-003, 004, 005)

| State (last_ns_dir) | current_ns | Gap | Path | Finding |
|---------------------|------------|-----|------|---------|
| None | any | N/A | A (first frame) | None — baseline set |
| Some(prev) | prev | 0 | B | None |
| Some(prev) | prev+1 | 1 | B | None |
| Some(prev) | prev+12 | 12 | B | None (boundary) |
| Some(prev) | prev+13 | 13 | C | T1692.001 Possible |
| Some(prev) | prev+19 | 19 | C | T1692.001 Possible (canonical) |
| Some(32767) | 1 | 2 (wrap) | B | None (wrap no false-pos) |
| Some(prev) | prev-delta | ~32767 | C | T1692.001 Possible (fail-closed) |

---

## Source-Level Evidence

Functions confirmed present in `src/analyzer/iec104.rs`:

- `extract_ns(cf1: u8, cf2: u8) -> u16` — pure free function
- `extract_nr(cf3: u8, cf4: u8) -> u16` — pure free function
- `Iec104FlowState` fields: `last_ns_c2s: Option<u16>`, `last_ns_s2c: Option<u16>`
- Gap check logic: `match last_ns_dir { None => ..., Some(prev) => { let gap = current_ns.wrapping_sub(prev) & 0x7FFF; ... } }`

---

## Edge Case Coverage Summary

| Edge Case | BC | Test Covering | Verdict |
|-----------|-----|--------------|---------|
| EC-001: First I-frame, N(S)=0 (fresh flow) | BC-2.19.024 Path A | `path_a_first_frame_c2s_ns_0_no_finding` | PASS |
| EC-002: First I-frame, N(S)=5000 (mid-capture) | BC-2.19.024 Path A; inv3 | `path_a_mid_capture_first_frame_c2s_ns_5000` | PASS |
| EC-003: Gap = 12 exactly (≤ k boundary) | BC-2.19.024 Path B | `path_b_gap_12_exactly_k_boundary_no_finding` | PASS |
| EC-004: Gap = 13 (k+1, just above boundary) | BC-2.19.024 Path C | `path_c_gap_13_k_plus_1_emits_t1692_001_possible` | PASS |
| EC-005: Wrap Some(32767)→1, gap=2 | BC-2.19.024 inv1 | `ac171_006_wrap_32767_to_1_gap_2_no_finding` | PASS |
| EC-006: Massive gap (32767) | BC-2.19.024 Path C | `path_c_ec005_gap_32767_massive_jump_emits_t1692_001_possible` | PASS |
| EC-007: RETRANSMIT-NS-FALSEPOS-001 (backwards N(S)) | BC-2.19.024 inv3 | `RETRANSMIT_NS_FALSEPOS_001_backwards_ns_yields_large_gap` | PASS (intentional) |

---

## Recording Method

This is an effectful library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Inline gap table and edge-case coverage tables
- Source-level verification of `extract_ns`, `extract_nr` function presence and `Iec104FlowState` fields

VHS/Playwright recordings are not applicable (no interactive surface at this story scope;
dispatch wiring to the CLI analyzer is STORY-173).

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-002-ns-nr-extraction.md` | AC-171-001 (BC-2.19.023 PC1,PC3), AC-171-002 (BC-2.19.023 PC2,PC4) |
| `AC-003-first-frame-option-guard.md` | AC-171-003 (BC-2.19.024 Path A) |
| `AC-004-gap-le-k12-no-finding.md` | AC-171-004 (BC-2.19.024 Path B) |
| `AC-005-gap-gt-k12-desync-finding.md` | AC-171-005 (BC-2.19.024 Path C) |
| `AC-006-wrap-arithmetic.md` | AC-171-006 (BC-2.19.024 inv1) |
| `AC-007-directional-isolation.md` | AC-171-007 (BC-2.19.023 PC3) |
| `RETRANSMIT-NS-FALSEPOS-001.md` | EC-007 edge case (BC-2.19.024 inv3) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute host-local paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-15).
