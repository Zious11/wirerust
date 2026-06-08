Output AC-008-verbose-short-flag-rejected.gif
Output AC-008-verbose-short-flag-rejected.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-008 (BC-2.13.004 postcondition 1): short form -v also not declared
# Demonstrates: wirerust analyze -v test.pcap → clap unexpected-argument error
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-096/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze -v test.pcap"
Enter
Sleep 2s
