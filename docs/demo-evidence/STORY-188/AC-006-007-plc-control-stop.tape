# STORY-188 AC-188-006/007: PLC Control service decode (incl. truncated/case/trailing) and PLC Stop
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_015 story_188::test_BC_2_21_016

Output AC-006-007-plc-control-stop.gif
Output AC-006-007-plc-control-stop.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 640
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 AC-188-006/007: PLC Control service decode (incl. truncated/case/trailing) and PLC Stop"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_015 story_188::test_BC_2_21_016 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
