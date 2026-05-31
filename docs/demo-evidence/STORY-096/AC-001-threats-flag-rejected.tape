Output AC-001-threats-flag-rejected.gif
Output AC-001-threats-flag-rejected.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-001 (BC-2.13.001 postcondition 1): --threats flag not declared; clap rejects it
# Demonstrates: wirerust analyze --threats test.pcap → clap unexpected-argument error
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-096/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze --threats test.pcap"
Enter
Sleep 2s
