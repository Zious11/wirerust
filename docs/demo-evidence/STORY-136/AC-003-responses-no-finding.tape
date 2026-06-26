# STORY-136 AC-136-003: ForwardOpen/ForwardClose responses (0xD4, 0xCE) do NOT emit findings
#
# AC-003: Response service bytes 0xD4 (ForwardOpen response) and 0xCE (ForwardClose response)
#          have high bit set → classify_cip_service returns CipServiceClass::Response, never
#          ForwardOpen/ForwardClose → no finding emitted (BC-2.17.007 Invariant 1)
#
# Tests:
#   connection_lifecycle::test_forward_open_response_no_finding
#   connection_lifecycle::test_forward_close_response_no_finding
# Traces: BC-2.17.015 Invariant 2; BC-2.17.007 Invariant 1

Output AC-003-responses-no-finding.gif
Output AC-003-responses-no-finding.webm

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

Type "# AC-136-003: CIP responses 0xD4/0xCE → no finding (response-bit in classify_cip_service)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'connection_lifecycle::test_forward_open_response_no_finding' 2>&1 | grep -E 'test_forward_open_response|ok|FAILED|result'"
Enter
Wait
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'connection_lifecycle::test_forward_close_response_no_finding' 2>&1 | grep -E 'test_forward_close_response|ok|FAILED|result'"
Enter
Wait
Sleep 2s
