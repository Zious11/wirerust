Output AC-010-valid-invocation-unaffected.gif
Output AC-010-valid-invocation-unaffected.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-010 / EC-004 (BC-2.13.001 postcondition 3 / BC-2.13.002 postcondition 3):
# Valid invocations parse OK — removed flags do not affect valid parses.
# IO/file error ("Target not found") proves clap parse succeeded; NOT a usage error.
# Also shows: --http flag accepted (EC-004)
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-096/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze test.pcap"
Enter
Sleep 1500ms

Type "$WIRERUST analyze --http test.pcap"
Enter
Sleep 2s
