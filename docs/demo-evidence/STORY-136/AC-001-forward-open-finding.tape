# STORY-136 AC-136-001: ForwardOpen (0x54) and LargeForwardOpen (0x5B) emit Anomaly/Possible/Low
#
# AC-001: CIP request with classify_cip_service returning ForwardOpen or LargeForwardOpen
#          + type_id==0x00B2 + !is_non_enip
#          → ONE Finding per occurrence:
#            category=Anomaly, verdict=Possible, confidence=Low, mitre_techniques=[]
#            summary="CIP ForwardOpen connection establishment observed from src=...: connection lifecycle anomaly"
#   - ForwardOpen response (0xD4) does NOT fire (response-bit handled inside classify_cip_service)
#   - type_id==0x00B1 does NOT fire (F-P9-001 gate)
#   - LargeForwardOpen (0x5B) treated identically to ForwardOpen (0x54)
#
# Tests:
#   connection_lifecycle::test_forward_open_emits_finding
#   connection_lifecycle::test_forward_open_no_mitre_technique
#   connection_lifecycle::test_forward_open_connected_item_no_finding
#   connection_lifecycle::test_forward_open_response_no_finding
#   connection_lifecycle::test_large_forward_open_emits_finding
# Traces: BC-2.17.015 Postconditions 1-3, Invariant 5; ADR-010 Decision 7

Output AC-001-forward-open-finding.gif
Output AC-001-forward-open-finding.webm

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

Type "# AC-136-001: ForwardOpen/LargeForwardOpen → Anomaly/Possible/Low, mitre_techniques=[] (BC-2.17.015)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'connection_lifecycle::test_forward_open' 2>&1 | grep -E 'test_forward_open|test_large_forward|ok|FAILED|result'"
Enter
Wait
Sleep 2s
