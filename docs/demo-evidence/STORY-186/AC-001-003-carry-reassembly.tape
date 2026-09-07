# STORY-186 AC-186-001/002/003: carry-buffer reassembly, walk-first extraction (BC-2.20.013)
#
# Demonstrates:
#   - test_BC_2_20_013_walk_first_no_aggregate_precheck: no aggregate carry+incoming
#     pre-check exists anywhere -- the walk always runs first (AC-186-001)
#   - test_BC_2_20_013_adversarial_burst_head_frame_not_dropped: a complete 7-byte CR
#     frame followed by 60,000 bytes of trailing garbage in ONE on_data call is still
#     extracted (anti-evasion property, AC-186-002)
#   - test_BC_2_20_013_split_frame_across_two_calls: a 4-byte TPKT header delivered in
#     call 1 is stashed to carry; call 2 completes the 10-byte frame and empties carry
#     (AC-186-003)
#
# Traces to: BC-2.20.013 postconditions 1-2, invariant 1, edge case EC-002

Output AC-001-003-carry-reassembly.gif
Output AC-001-003-carry-reassembly.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 50ms

Require cargo

Type "# AC-186-001/002/003: walk-first carry-buffer reassembly (BC-2.20.013)"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests BC_2_20_013 2>&1 | grep -E 'story_186::test|test result:'"
Enter
Wait
Sleep 2s
