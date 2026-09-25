# STORY-187 AC-187-001/002/003: S7commFlowState field set, lazy creation, and
# CR/CC session_established matching (BC-2.21.001)
#
# Demonstrates:
#   - test_BC_2_21_001_flow_state_field_set: S7commFlowState carries the full
#     field set this story adds (session_established, cr_observed_dir,
#     classified_protocol, malformed_header_reported_c2s/_s2c) (AC-187-001)
#   - test_BC_2_21_001_lazy_flow_state_creation: state is created on first
#     on_data call, not before (AC-187-002)
#   - test_BC_2_21_001_cr_then_opposite_cc_sets_session_established: CR(A) then
#     CC(B), B opposite A -> session_established becomes true (AC-187-003)
#   - Five negative sequences (CR-only, CC-only, CC-before-CR, same-direction
#     CC, repeated same-direction CR) that all leave session_established false
#     (AC-187-003)
#   - test_BC_2_21_001_most_recent_cr_direction_wins: cr_observed_dir is
#     overwritten by every CR, not only the first (AC-187-003)
#   - test_BC_2_21_001_session_established_is_monotonic: once true, the flag is
#     never reverted by later frames (AC-187-003)
#
# Traces to: BC-2.21.001 postconditions 1-3, edge cases EC-004..EC-009
# Test: cargo test --test s7comm_analyzer_tests BC_2_21_001

Output AC-001-003-flow-state-session.gif
Output AC-001-003-flow-state-session.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-187-001/002/003: S7commFlowState field set, lazy creation, CR/CC session tracking (BC-2.21.001)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_21_001 2>&1 | grep -E 'story_187::test|test result:'"
Enter
Wait
Sleep 2s
