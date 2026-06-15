Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-001-002-016-d1-spoof-unit-tests.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-001-002-016-d1-spoof-unit-tests.webm"

Set Theme "Dracula"
Set FontFamily "Menlo"
Set Width 1200
Set Height 750
Set Padding 20
Set FontSize 14

# AC-001 (first rebind MEDIUM), AC-002 (escalate to HIGH at threshold),
# AC-016 (evidence has old+new MAC)
# No fixture triggers D1 in real pcaps — demonstrated via unit tests
# (synthetic bindings in arp.rs tests exercise the exact logic)

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114"
Enter
Sleep 500ms
Show

Type "# AC-001/002/016: D1 spoof detection + escalation + evidence MAC fields"
Enter
Sleep 300ms

Type "cargo test test_d1 2>&1 | grep -E 'test analyzer|test result.*passed'"
Enter
Sleep 8s

Sleep 2s
