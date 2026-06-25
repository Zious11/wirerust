# STORY-131 AC-131-003 / AC-131-005 / AC-131-006: CLI flags
#
# AC-003: --enip flag appears in analyze --help, default-off; --all includes it
# AC-005: --enip-write-burst-threshold with default 50
# AC-006: --enip-error-burst-threshold with default 5
# Traces: BC-2.17.020 postconditions 1/3/4; BC-2.17.023 invariant 1; BC-2.17.026 invariant 1

Output AC-003-005-006-cli-flags.gif
Output AC-003-005-006-cli-flags.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-131-enip-dispatcher"
Enter
Wait
Show

Type "# AC-131-003/005/006: ENIP CLI flags in analyze --help (BC-2.17.020/023/026)"
Enter
Sleep 300ms

Type "cargo run -- analyze --help 2>&1 | grep -A3 '\-\-enip'"
Enter
Wait
Sleep 2s
