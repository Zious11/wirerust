Output AC-002-EC-001-protocol-flags.gif
Output AC-002-EC-001-protocol-flags.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-002/EC-001 (BC-2.12.001 postcondition 3): --dns --http --all flags accepted by clap
# IO error proves parsing succeeded; clap accepted all three flags
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze cap.pcap --dns --http --all"
Enter
Sleep 2s
