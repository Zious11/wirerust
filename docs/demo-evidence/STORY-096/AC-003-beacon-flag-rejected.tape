Output AC-003-beacon-flag-rejected.gif
Output AC-003-beacon-flag-rejected.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-003 / EC-002 (BC-2.13.002 postcondition 1): --beacon flag not declared
# Demonstrates: wirerust analyze --beacon test.pcap → clap unexpected-argument error
# EC-002: error fires before any analysis; combined with valid flag to show ordering
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-096/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze --beacon test.pcap"
Enter
Sleep 2s
