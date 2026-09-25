# STORY-187 AC-187-004/005/012: Some(0x32) DT dispatch, sticky first-classification-
# wins, and classic-dissection gating on sticky classified_protocol (BC-2.21.002)
#
# Demonstrates:
#   - test_BC_2_21_002_classic_s7comm_dispatch_asserts_classified_protocol_classic:
#     a Some(0x32) DT frame dispatches to classic dissection (AC-187-004)
#   - test_BC_2_21_002_malformed_0x32_frame_on_classic_flow_emits_exactly_one_t0814:
#     malformed error-path counterpart -- exactly one T0814, not zero, not more
#     (AC-187-004)
#   - test_BC_2_21_002_sticky_first_classification: first DT frame's protocol_id
#     classifies the flow exactly once; later frames never overwrite it (AC-187-005)
#   - test_BC_2_21_002_none_protocol_id_dt_first_then_0x32_dt_classifies_classic:
#     protocol_id: None never classifies; classification stays deferred (AC-187-005)
#   - test_BC_2_21_002_two_frames_one_delivery_first_write_wins /
#     test_BC_2_21_002_empty_dt_followed_by_frame_same_delivery_stays_unclassified /
#     test_BC_2_21_002_unparseable_cotp_does_not_classify (AC-187-005)
#   - test_BC_2_21_002_0x32_dt_frame_not_dissected_when_sticky_classified_plus_or_unclassified /
#     test_BC_2_21_002_non_0x32_dt_frame_on_classic_flow_not_dissected: the gate is a
#     conjunction of BOTH the frame's own protocol_id byte AND the flow's sticky
#     classified_protocol -- neither alone is sufficient (AC-187-012)
#   - proptest_vp053_protocol_id_dispatch_totality: VP-053 property -- for any
#     protocol_id (None or Some(byte)), the first DT frame classifies exactly per
#     the four-way table and always creates flow state
#
# Note: this filter also picks up test_BC_2_21_002_setup_comm_fixture_pcap_well_formed_no_findings
# (AC-187-013's fixture-pcap regression test, also recorded in its own dedicated
# context in AC-013-canonical-frames) since it shares this BC's dispatch path.
#
# Traces to: BC-2.21.002 postconditions 1, 3, 6, invariant 4, edge cases EC-003..EC-005
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_002
# Test: cargo test --test s7comm_analyzer_tests vp053

Output AC-004-005-012-dispatch-classification.gif
Output AC-004-005-012-dispatch-classification.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 620
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-187-004/005/012: Some(0x32) DT dispatch + sticky first-classification-wins + gating (BC-2.21.002)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_002 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 1s

Type "cargo test --test s7comm_analyzer_tests vp053 2>&1 | grep -E 'story_187::vp053|test result:'"
Enter
Wait
Sleep 2s
