# STORY-130 AC-003 + AC-005: classify_enip_command — all 9 named variants and Unknown arm
#
# AC-003: 0x0004..0x0075 mapped to named variants (ListServices, SendRRData, Cancel, etc.)
# AC-005: 0x0000, 0xFFFF, 0x0067 (gap) map to Unknown (non-vacuous Unknown arm).
# Traces: BC-2.17.004 postconditions 2/4; VP-032 Sub-B biconditional.

Output AC-003-005-classify-command.gif
Output AC-003-005-classify-command.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-130-enip-pure-core-parse"
Enter
Wait
Show

Type "# AC-003/005: classify_enip_command named variants + Unknown arm (BC-2.17.004)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests parse_header::test_classify 2>&1 | tail -12"
Enter
Wait
Sleep 2s
