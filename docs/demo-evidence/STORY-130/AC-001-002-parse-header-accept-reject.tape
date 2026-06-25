# STORY-130 AC-001 + AC-002: parse_enip_header accept path and reject path
#
# AC-001: 24-byte canonical SendRRData vector -> Some(EnipHeader) with correct LE fields
#         (session_handle = 0x01020304, command = 0x006F).
# AC-002: empty slice and 23-byte slice -> None, no panic.
# Traces: BC-2.17.001, BC-2.17.002; VP-032 Sub-A targets.

Output AC-001-002-parse-header-accept-reject.gif
Output AC-001-002-parse-header-accept-reject.webm

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

Type "# AC-001/002: parse_enip_header accept path + reject path (BC-2.17.001/002)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests parse_header::test_parse_enip_header 2>&1 | tail -12"
Enter
Wait
Sleep 2s
