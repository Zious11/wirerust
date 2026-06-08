Output AC-006-summary-hosts-flag.gif
Output AC-006-summary-hosts-flag.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-006 (BC-2.12.002 postcondition 3): --hosts flag accepted by summary subcommand
# IO error proves clap parsed successfully (not a usage error)
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST summary cap.pcap --hosts"
Enter
Sleep 2s
