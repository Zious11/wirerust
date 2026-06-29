# STORY-136 AC-136-002: ForwardClose (0x4E) emits Anomaly/Possible/Low (no MITRE technique)
#
# AC-002: CIP request with classify_cip_service returning ForwardClose
#          + type_id==0x00B2 + !is_non_enip
#          → ONE Finding per occurrence:
#            category=Anomaly, verdict=Possible, confidence=Low, mitre_techniques=[]
#            summary="CIP ForwardClose connection teardown observed from src=...: connection lifecycle closed"
#   - ForwardClose response (0xCE) does NOT fire (response-bit handled inside classify_cip_service)
#
# Tests:
#   connection_lifecycle::test_forward_close_emits_finding
#   connection_lifecycle::test_forward_close_no_mitre_technique
# Traces: BC-2.17.015 Postconditions 4-5; ADR-010 Decision 7

Output AC-002-forward-close-finding.gif
Output AC-002-forward-close-finding.webm

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

Type "# AC-136-002: ForwardClose (0x4E) → Anomaly/Possible/Low, mitre_techniques=[] (BC-2.17.015 PC-4/PC-5)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'connection_lifecycle::test_forward_close' 2>&1 | grep -E 'test_forward_close|ok|FAILED|result'"
Enter
Wait
Sleep 2s
