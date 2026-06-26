# STORY-138 AC-001: RegisterSession/UnRegisterSession — classified, PDU-counted, no finding
#
# Demonstrates: RegisterSession (0x0065) and UnRegisterSession (0x0066) increment pdu_count
#   and are reflected in command_counts, but emit NO finding (BC-2.17.025 / BC-2.17.024).
#
# Tests:
#   test_register_session_pdu_counted_no_finding  — 0x0065 → pdu_count=1, no finding
#   test_unregister_session_pdu_counted_no_finding — 0x0066 → pdu_count=1, no finding
#   test_session_command_counts_accumulated        — multiple session frames → command_counts correct
#
# Traces to: BC-2.17.025 Postconditions 1–5, BC-2.17.024 Postconditions 1–2

Output AC-001-session-handshake.gif
Output AC-001-session-handshake.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-138-session-summary"
Enter
Wait
Show

Type "# AC-001: RegisterSession/UnRegisterSession — PDU-counted, no finding (BC-2.17.025)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'session_lifecycle::test_register_session|session_lifecycle::test_unregister_session|session_lifecycle::test_session_command' 2>&1 | grep -E 'session_lifecycle::test_register|session_lifecycle::test_unregister|session_lifecycle::test_session_command|result'"
Enter
Wait
Sleep 2s
