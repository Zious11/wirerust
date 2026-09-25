# STORY-187 AC-187-010/011: ROSCTR=Ack/Ack_Data 12-byte header + error_class/
# error_code extraction, truncated-Ack(_Data) rejection, and declared
# param_length/data_length bounds-checking before slice access (BC-2.21.008/009)
#
# Demonstrates:
#   - test_BC_2_21_008_ack_rosctr_12_byte_minimum_and_error_fields /
#     test_BC_2_21_008_ack_data_12_byte_header_and_error_fields /
#     test_BC_2_21_008_canonical_ack_vector_verbatim /
#     test_BC_2_21_008_error_fields_none_for_job_and_userdata_rosctr /
#     test_BC_2_21_008_ack_data_nonzero_error_fields_with_parameter_block: Ack AND
#     Ack_Data both require 12 bytes, error_class/error_code extracted, Some only
#     for Ack/AckData (AC-187-010 -- SUCCESS PATH)
#   - test_BC_2_21_008_truncated_ack_data_returns_none /
#     test_BC_2_21_008_truncated_ack_on_data_emits_t0814_once /
#     test_BC_2_21_008_truncated_ack_data_on_data_emits_t0814_once: 10/11-byte
#     truncated Ack and Ack_Data both return None with reason-specific T0814
#     evidence, distinguishable from each other and from AC-187-006/009's reasons
#     (AC-187-010 -- ERROR PATH)
#   - test_BC_2_21_009_bounds_check_before_parameter_data_slice /
#     test_BC_2_21_009_bounds_check_dedup_s2c /
#     test_BC_2_21_009_ack_header_len_12_bounds_check /
#     test_BC_2_21_009_data_length_overrun_on_data_emits_t0814 /
#     test_BC_2_21_009_dissection_bounded_to_own_tpkt_frame /
#     test_BC_2_21_009_bounds_failure_evidence_reports_declared_and_available /
#     test_BC_2_21_009_bounds_failure_evidence_ack_data_header_len_12: declared
#     param_length/data_length bounds-checked before any slice access; never
#     borrows bytes from a trailing frame; per-direction dedup verified for BOTH
#     c2s and s2c (AC-187-011 -- ERROR PATH)
#   - test_BC_2_21_009_bounds_check_passes_exact_match /
#     test_BC_2_21_009_empty_parameter_and_data_blocks_trivial_pass /
#     test_BC_2_21_009_overflow_free_arithmetic_max_values /
#     test_BC_2_21_009_s7comm_bounds_ok_data_length_only_overrun /
#     test_BC_2_21_009_s7comm_bounds_ok_helper_matches_bounds_decision: bounds
#     check passes on exact-fit data and never overflows (AC-187-011 -- SUCCESS
#     PATH)
#   - proptest_bc_2_21_006_008_some_iff_rosctr_and_length_conditional: joint
#     length-conditional totality across BC-2.21.006/007/008, exhaustive over all
#     256 ROSCTR byte values and lengths [9,10,11,12,13]
#
# Traces to: BC-2.21.008 postconditions 1-4, edge case EC-007; BC-2.21.009
# postconditions 1-3
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_008
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_009
# Test: cargo test --test s7comm_analyzer_tests bc_2_21_006_008

Output AC-010-011-ack-bounds-check.gif
Output AC-010-011-ack-bounds-check.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 900
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# AC-187-010/011: Ack/Ack_Data 12-byte header extraction + bounds-check before slice access"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_008 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_009 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests bc_2_21_006_008 2>&1 | grep -E 'story_187::proptest|test result:'"
Enter
Wait
Sleep 2s
