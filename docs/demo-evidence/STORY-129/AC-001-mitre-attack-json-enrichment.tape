# STORY-129 AC-1/AC-4/AC-7/AC-8: per-finding mitre_attack array in JSON output
# Demonstrates: known techniques produce fully-resolved 5-field objects;
#   multi-tag preserves order; ICS tactic_id correct (T0888->TA0102, T0836->TA0106);
#   mitre_techniques unchanged.
# Input: tests/fixtures/modbus-write.pcap (3 findings with mitre_attack populated)
# Tool: wirerust CLI (release build)
# Re-recorded 2026-06-23 on fix/ics-tactic-ids: T0888 now resolves to TA0102 "Discovery (ICS)"

Output docs/demo-evidence/STORY-129/AC-001-mitre-attack-json-enrichment.gif
Output docs/demo-evidence/STORY-129/AC-001-mitre-attack-json-enrichment.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 650
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Require wirerust

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/ics-tactic-ids"
Enter
Sleep 1s
Show

Type "./target/release/wirerust analyze --json --modbus tests/fixtures/modbus-write.pcap | python3 -m json.tool | grep -A 25 mitre_attack | head -60"
Sleep 500ms
Enter

Sleep 6s
