Output AC-009-small-segment-ignore-ports.gif
Output AC-009-small-segment-ignore-ports.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-009 (BC-2.12.005 invariant 3): --small-segment-ignore-ports 23,513 → accepted (comma-delimited Vec<u16>)
# "Error: Target not found" proves clap accepted the comma-delimited value; a clap error would show "error: ..."
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --small-segment-ignore-ports 23,513 analyze x.pcap"
Enter
Sleep 2s
