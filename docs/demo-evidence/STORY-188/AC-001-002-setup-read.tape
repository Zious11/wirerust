# STORY-188 AC-188-001/002: Setup Communication (0xF0) and Read Var (0x04) classification
# Library story: demonstration vehicle is the story_188 test harness
# (tests/s7comm_analyzer_tests.rs, mod story_188); no CLI surface until STORY-193.
# Test: cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_010_setup story_188::test_BC_2_21_011

Output AC-001-002-setup-read.gif
Output AC-001-002-setup-read.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set Padding 24
Set Shell "bash"
Set TypingSpeed 40ms

Require cargo

Type "# STORY-188 AC-188-001/002: Setup Communication (0xF0) and Read Var (0x04) classification"
Enter
Sleep 300ms

Type "cargo test --test s7comm_analyzer_tests -- story_188::test_BC_2_21_010_setup story_188::test_BC_2_21_011 2>&1 | grep -E 'running [0-9]+ tests|story_188::|test result:'"
Enter
Wait
Sleep 3s
