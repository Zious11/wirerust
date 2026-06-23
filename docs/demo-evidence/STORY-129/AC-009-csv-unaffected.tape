# STORY-129 AC-9: mitre_attack absent from CSV output (non-breaking / additive)
# Demonstrates: CSV reporter emits only mitre_techniques (raw IDs), no mitre_attack key.
# Input: tests/fixtures/modbus-write.pcap
# Tool: wirerust CLI (release build)

Output AC-009-csv-unaffected.gif
Output AC-009-csv-unaffected.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 400
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-129"
Enter
Sleep 1s
Show

Type "./target/release/wirerust analyze --csv --modbus tests/fixtures/modbus-write.pcap"
Sleep 500ms
Enter

Sleep 4s
