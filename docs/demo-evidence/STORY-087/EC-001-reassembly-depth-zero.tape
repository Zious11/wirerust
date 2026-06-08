Output EC-001-reassembly-depth-zero.gif
Output EC-001-reassembly-depth-zero.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# EC-001: --reassembly-depth 0 → accepted (0 is a valid u64; reassembly_depth = 0)
# "Error: Target not found" proves clap accepted the flag value
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --reassembly-depth 0 analyze x.pcap"
Enter
Sleep 2s
