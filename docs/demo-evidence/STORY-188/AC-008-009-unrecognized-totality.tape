# STORY-188 AC-188-008/009: unrecognized FC, empty and unsliceable block, 256-value totality, VP-052 proptest
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_017 vp052

Output AC-008-009-unrecognized-totality.gif
Output AC-008-009-unrecognized-totality.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 640
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 AC-188-008/009: unrecognized FC, empty and unsliceable block, 256-value totality, VP-052 proptest"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_017 vp052 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
