Output AC-001-analyze-single-target.gif
Output AC-001-analyze-single-target.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-001 (BC-2.12.001): wirerust analyze cap.pcap — clap accepts single target
# "Error: Target not found" proves parsing succeeded; a clap error would print usage
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze cap.pcap"
Enter
Sleep 2s
