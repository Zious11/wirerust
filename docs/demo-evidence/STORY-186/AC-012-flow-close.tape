# STORY-186 AC-186-012: on_flow_close teardown + double-close idempotency (BC-2.21.003)
#
# Demonstrates:
#   - test_s7comm_on_flow_close_removes_state_discards_carry: on_flow_close removes
#     S7commFlowState from the per-flow map and discards carry bytes with no finding
#     emitted (AC-186-012)
#   - test_BC_2_21_003_double_close_same_flow_key_is_idempotent_no_op: calling
#     on_flow_close a second time for the same flow_key (or an unknown flow_key) is a
#     no-op -- does not panic, does not re-emit
#
# Traces to: BC-2.21.003 postconditions 1-4

Output AC-012-flow-close.gif
Output AC-012-flow-close.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 460
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-186-012: on_flow_close removes state, discards carry, double-close is a no-op (BC-2.21.003)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests close 2>&1 | grep -E 'story_186::test|test result:'"
Enter
Wait
Sleep 2s
