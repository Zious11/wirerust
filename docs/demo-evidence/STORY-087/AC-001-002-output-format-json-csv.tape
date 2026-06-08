Output AC-001-002-output-format-json-csv.gif
Output AC-001-002-output-format-json-csv.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-001 (BC-2.12.004 postcondition 1): --output-format json accepted
# AC-002 (BC-2.12.004 postcondition 2): --output-format csv accepted
# "Error: Target not found" proves clap accepted the flag; a clap error would show "error: ..."
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --output-format json summary x.pcap"
Enter
Sleep 1s

Type "$BIN --output-format csv summary x.pcap"
Enter
Sleep 2s
