# STORY-134 AC-134-004 (formerly AC-134-003): T0888 Pattern A — CIP GetAttribute to Identity Object
#
# AC-004: CIP request with GetAttributeSingle/All/List + CipPathSegment::Class(0x01) +
#          type_id==0x00B2 → ONE Finding (category=Reconnaissance, verdict=Likely,
#          confidence=High, mitre_techniques=["T0888"])
#   - Fires per-occurrence (NOT one-shot)
#   - Does NOT fire for type_id==0x00B1 (F-P9-001 gate)
#   - Does NOT fire for GetAttribute to non-Identity classes (e.g., Class 0x04 Assembly)
#
# Tests:
#   recon::test_t0888_pattern_a_identity_read
#   recon::test_t0888_pattern_a_non_identity_no_finding
#   recon::test_t0888_pattern_a_connected_item_no_finding
#   recon::test_t0888_pattern_a_fires_per_occurrence
# Traces: BC-2.17.014 postconditions Pattern A; F-P9-001

Output AC-004-t0888-pattern-a.gif
Output AC-004-t0888-pattern-a.webm

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

Type "# AC-134-004: T0888 Pattern A — CIP Identity Object reads (BC-2.17.014)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'recon::test_t0888_pattern_a' 2>&1 | grep -E 'test_t0888_pattern_a|ok|FAILED|result'"
Enter
Wait
Sleep 2s
