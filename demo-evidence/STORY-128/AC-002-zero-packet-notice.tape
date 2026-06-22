# STORY-128 AC-002 / BC-2.01.009 PC6: Zero-packet notice variants
# Case 1: SHB-only -> bare notice
# Case 2: OPB-bearing -> notice + obsolete-block clause + mergecap hint
# Case 3: 2 unknown-skip blocks -> notice + "(2 block(s) skipped as unsupported)"

Output AC-002-zero-packet-notice.gif
Output AC-002-zero-packet-notice.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 15
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type `alias wirerust="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-128/target/debug/wirerust"`
Enter
Wait
Type `python3 /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-128/make-fixtures-notice.py`
Enter
Wait
Show

Type `alias wirerust="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-128/target/debug/wirerust"`
Enter
Wait

Type "# Case 1: SHB-only -- bare notice (Decision 19, no parenthetical)"
Enter
Sleep 200ms
Type "wirerust analyze /tmp/demo-128-notice/shb-only.pcapng --no-color 2>&1"
Enter
Wait
Sleep 500ms

Type "# Case 2: OPB-bearing -- notice + obsolete-block clause + mergecap hint"
Enter
Sleep 200ms
Type "wirerust analyze /tmp/demo-128-notice/opb-bearing.pcapng --no-color 2>&1"
Enter
Wait
Sleep 500ms

Type "# Case 3: 2 unknown skip-blocks -- '(2 block(s) skipped as unsupported)'"
Enter
Sleep 200ms
Type "wirerust analyze /tmp/demo-128-notice/unknown-skip.pcapng --no-color 2>&1"
Enter
Wait
Sleep 2s
