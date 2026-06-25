# STORY-130 AC-001 through AC-006 (Kani harnesses excluded — cargo kani only):
# Full enip_analyzer_tests suite: 21/21 green.
# Covers all unit-test-verifiable ACs in one recording.
# AC-006 (Kani Sub-A/B/C) proofs are not run via cargo test.
# Traces: BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004.

Output AC-001-006-all-tests.gif
Output AC-001-006-all-tests.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 900
Set FontSize 13
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-130-enip-pure-core-parse"
Enter
Wait
Show

Type "# STORY-130 full test suite: 21 tests across AC-001..005"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 2>&1"
Enter
Wait
Sleep 2s
