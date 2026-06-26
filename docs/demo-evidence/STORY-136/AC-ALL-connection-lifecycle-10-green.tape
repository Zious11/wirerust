# STORY-136 AC-ALL: Full mod connection_lifecycle suite — 10/10 green
#
# Master green-run covering all STORY-136 acceptance criteria:
#   AC-136-001: ForwardOpen/LargeForwardOpen → Anomaly/Possible/Low, mitre_techniques=[]
#   AC-136-002: ForwardClose → Anomaly/Possible/Low, mitre_techniques=[]
#   AC-136-003: Responses 0xD4/0xCE → no finding
#   AC-136-004: is_non_enip suppresses all lifecycle findings
#   AC-136-005: open_connection_count / close_connection_count tracked

Output AC-ALL-connection-lifecycle-10-green.gif
Output AC-ALL-connection-lifecycle-10-green.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-136-enip-lifecycle"
Enter
Wait
Show

Type "# STORY-136: mod connection_lifecycle full suite (10 tests) — BC-2.17.015"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests connection_lifecycle 2>&1 | grep -E 'test connection_lifecycle|ok|FAILED|result'"
Enter
Wait
Sleep 2s
