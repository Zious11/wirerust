# STORY-188 AC-188-010: Ack/Ack_Data error record, count map, cap/dropped, pdu_reference, non-recording paths
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests story_188::test_BC_2_21_008

Output AC-010-ack-error-record.gif
Output AC-010-ack-error-record.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 860
Set FontSize 12
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 AC-188-010: Ack/Ack_Data error record, count map, cap/dropped, pdu_reference, non-recording paths"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests story_188::test_BC_2_21_008 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
