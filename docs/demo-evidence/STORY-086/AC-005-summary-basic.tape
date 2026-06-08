Output AC-005-summary-basic.gif
Output AC-005-summary-basic.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-005 (BC-2.12.002 postcondition 1): summary subcommand accepts single target
# IO error proves clap parsed successfully
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST summary cap.pcap"
Enter
Sleep 2s
