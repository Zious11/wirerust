# STORY-187 AC-187-006/007/008/009: parse_s7comm_header 10-byte minimum, defensive
# 0x32 reject, Job/Userdata common-header extraction, and unrecognized-ROSCTR
# safe-reject totality (BC-2.21.004/005/006/007)
#
# Demonstrates:
#   - test_BC_2_21_004_parse_returns_none_for_len_lt_10 /
#     test_BC_2_21_004_len_shorter_than_10_returns_none_and_emits_t0814_once /
#     test_BC_2_21_004_len_shorter_than_10_emits_t0814_once_s2c /
#     test_BC_2_21_004_nine_byte_payload_on_data_too_short_evidence: too-short
#     header (<10 bytes) returns None, no panic, one T0814 per direction, exact
#     boundary evidence text (AC-187-006 -- ERROR PATH)
#   - test_BC_2_21_005_defensive_reject_wrong_protocol_id_byte /
#     test_BC_2_21_005_defensive_reject_zero_byte: data[0] != 0x32 returns None,
#     no Finding -- caller-hygiene defense-in-depth (AC-187-007)
#   - test_BC_2_21_006_common_header_field_extraction /
#     test_BC_2_21_006_nonzero_reserved_bytes_do_not_reject /
#     test_BC_2_21_006_byte_asymmetric_big_endian_decode /
#     test_BC_2_21_006_canonical_setup_communication_job_frame_on_data: Job/Userdata
#     common-header field extraction at the correct byte offsets (AC-187-008 --
#     SUCCESS PATH)
#   - test_BC_2_21_007_unrecognized_rosctr_returns_none /
#     test_BC_2_21_007_shares_dedup_flag_with_004_malformed_header /
#     test_BC_2_21_007_unrecognized_rosctr_emits_t0814_once_s2c /
#     proptest_bc_2_21_007_rosctr_byte_totality_over_all_256_values: unrecognized
#     ROSCTR byte returns None for all 252 remaining u8 values, no panic, shared
#     dedup flag with AC-187-006, s2c dedup verified independently (AC-187-009 --
#     ERROR PATH + exhaustive totality)
#
# Traces to: BC-2.21.004 postconditions 1-2/4, BC-2.21.005 postconditions 1/3,
# BC-2.21.006 postconditions 1-5, BC-2.21.007 postconditions 1-3
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_004
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_005
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_006
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_007
# Test: cargo test --test s7comm_analyzer_tests bc_2_21_007

Output AC-006-009-length-rosctr-parser.gif
Output AC-006-009-length-rosctr-parser.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 780
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-187-006/007/008/009: 10-byte minimum, defensive 0x32 reject, Job/Userdata extraction, ROSCTR totality"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_004 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_005 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_006 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_007 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 500ms

Type "cargo test --test s7comm_analyzer_tests bc_2_21_007 2>&1 | grep -E 'story_187::proptest|test result:'"
Enter
Wait
Sleep 2s
