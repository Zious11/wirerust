Output AC-008-overlap-threshold-out-of-range.gif
Output AC-008-overlap-threshold-out-of-range.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-008 (BC-2.12.005 postcondition 6): --overlap-threshold 256 → clap range error
# Expected: "error: invalid value '256' for '--overlap-threshold': 256 is not in 0..=255"
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --overlap-threshold 256 analyze x.pcap"
Enter
Sleep 2s
