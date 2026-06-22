# STORY-128 AC-003: Test suite -- all 22 BC-2.01.018 tests pass
# cargo test --test bc_2_01_018_story128_tests

Output AC-003-tests-passing.gif
Output AC-003-tests-passing.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-128"
Enter
Wait
Show

Type "# STORY-128: 22-test BC-2.01.018 suite (isolation + notice + PC6)"
Enter
Sleep 200ms
Type "cargo test --test bc_2_01_018_story128_tests 2>&1 | tail -30"
Enter
Wait
Sleep 2s
