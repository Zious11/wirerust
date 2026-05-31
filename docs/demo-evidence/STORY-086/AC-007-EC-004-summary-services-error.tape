Output AC-007-EC-004-summary-services-error.gif
Output AC-007-EC-004-summary-services-error.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-007/EC-004 (BC-2.12.002 invariant 4): --services removed flag → clap UnknownArgument
# clap prints "unexpected argument '--services' found"
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST summary cap.pcap --services"
Enter
Sleep 2s
