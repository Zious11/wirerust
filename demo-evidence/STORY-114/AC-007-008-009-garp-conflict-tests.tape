Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-007-008-009-garp-conflict-tests.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-007-008-009-garp-conflict-tests.webm"

Set Theme "Dracula"
Set FontFamily "Menlo"
Set Width 1200
Set Height 750
Set Padding 20
Set FontSize 14

# AC-007 (GARP upgrades to MEDIUM on conflict), AC-008 (D1 also emitted),
# AC-009 (D1 HIGH at threshold with GARP MEDIUM)

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114"
Enter
Sleep 500ms
Show

Type "# AC-007/008/009: GARP-that-Conflicts escalation + co-emitted D1 finding"
Enter
Sleep 300ms

Type "cargo test test_garp_conflicts 2>&1 | grep -E 'test analyzer|test result.*passed'"
Enter
Sleep 8s

Sleep 2s
