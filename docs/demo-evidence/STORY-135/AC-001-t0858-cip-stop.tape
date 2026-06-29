# STORY-135 AC-135-001: CIP Stop service (0x07) emits T0858 (Change Operating Mode)
#
# AC-001: CIP request with service=0x07 (Stop) + type_id==0x00B2 + !is_non_enip
#          → ONE Finding per occurrence (category=Execution, verdict=Likely, confidence=High,
#            mitre_techniques=["T0858"], summary="CIP Stop service observed: controller
#            run→stop transition command (T0858)")
#   - Does NOT fire for CIP Stop responses (0x87)
#   - Does NOT fire for type_id==0x00B1 (F-P9-001 gate)
#   - SetAttribute services (0x02/0x04/0x10) do NOT trigger T0858 — feed T0836 path
#
# Tests:
#   command_detections::test_t0858_stop_service_0x07
#   command_detections::test_t0858_stop_response_no_finding
#   command_detections::test_t0858_connected_item_no_finding
#   command_detections::test_t0858_set_attribute_no_t0858
# Traces: BC-2.17.011; F-P9-001

Output AC-001-t0858-cip-stop.gif
Output AC-001-t0858-cip-stop.webm

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

Type "# AC-135-001: T0858 CIP Stop (0x07) — emits Execution finding (BC-2.17.011)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'command_detections::test_t0858' 2>&1 | grep -E 'test_t0858|ok|FAILED|result'"
Enter
Wait
Sleep 2s
