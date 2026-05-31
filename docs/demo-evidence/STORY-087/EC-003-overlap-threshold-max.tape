Output EC-003-overlap-threshold-max.gif
Output EC-003-overlap-threshold-max.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# EC-003: --overlap-threshold 255 (max boundary) → accepted (overlap_threshold = Some(255))
# "Error: Target not found" proves clap accepted the max boundary value
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --overlap-threshold 255 analyze x.pcap"
Enter
Sleep 2s
