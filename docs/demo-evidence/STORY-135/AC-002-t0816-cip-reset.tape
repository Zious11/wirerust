# STORY-135 AC-135-002: CIP Reset service (0x05) emits T0816 (Device Restart/Shutdown)
#
# AC-002: classify_cip_service(service)==CipServiceClass::Reset + type_id==0x00B2 + !is_non_enip
#          → ONE Finding per occurrence (category=Execution, verdict=Likely, confidence=High,
#            mitre_techniques=["T0816"],
#            summary="CIP Reset service observed: adversary-triggered device restart (T0816)")
#   - Does NOT fire for CIP Reset responses (0x85 — high bit set → Response class)
#   - Does NOT fire for type_id==0x00B1 (F-P9-001 gate)
#   - Per-occurrence (no one-shot guard — BC-2.17.013 postcondition 2)
#
# Tests:
#   command_detections::test_t0816_reset_service
#   command_detections::test_t0816_response_no_finding
#   command_detections::test_t0816_connected_item_no_finding
# Traces: BC-2.17.013; F-P9-001

Output AC-002-t0816-cip-reset.gif
Output AC-002-t0816-cip-reset.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-135-enip-command-detections"
Enter
Wait
Show

Type "# AC-135-002: T0816 CIP Reset (0x05) — emits Execution finding (BC-2.17.013)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'command_detections::test_t0816' 2>&1 | grep -E 'test_t0816|ok|FAILED|result'"
Enter
Wait
Sleep 2s
