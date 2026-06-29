# STORY-134 AC-134-001: ListIdentity ENIP command emits T0846 finding (per-flow one-shot)
#
# AC-001: EnipAnalyzer::process_pdu with command=0x0063 (ListIdentity) →
#          - Pushes exactly 1 Finding (category=Reconnaissance, verdict=Likely,
#            confidence=High, mitre_techniques=["T0846"])
#          - Sets flow.list_identity_emitted = true (per-flow one-shot guard)
#          - Subsequent ListIdentity frames on same flow: NO additional finding
#
# Tests:
#   recon::test_list_identity_emits_t0846
#   recon::test_list_identity_one_shot_guard_multi_frame
#   recon::test_list_identity_respects_max_findings
# Traces: BC-2.17.010 postconditions 1-3; BC-2.17.010 invariant 1

Output AC-001-t0846-list-identity.gif
Output AC-001-t0846-list-identity.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-134-enip-recon"
Enter
Wait
Show

Type "# AC-134-001: ListIdentity → T0846 one-shot detection (BC-2.17.010)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'recon::test_list_identity' 2>&1 | grep -E 'test_list_identity|ok|FAILED|result'"
Enter
Wait
Sleep 2s
