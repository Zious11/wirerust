# STORY-188 AC-188-003: Write Var area-code extraction, descriptor boundary 11/12/13/14, exhaustive u8
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_012 area_exhaustive

Output AC-003-write-var-area.gif
Output AC-003-write-var-area.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 13
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 AC-188-003: Write Var area-code extraction, descriptor boundary 11/12/13/14, exhaustive u8"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_012 area_exhaustive 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
