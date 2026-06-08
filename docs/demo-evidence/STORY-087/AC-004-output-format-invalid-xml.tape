Output AC-004-output-format-invalid-xml.gif
Output AC-004-output-format-invalid-xml.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-004 (BC-2.12.004 postcondition 4): --output-format xml → clap error (unrecognized variant)
# Expected: "error: invalid value 'xml' for '--output-format <OUTPUT_FORMAT>'"
#           "[possible values: json, csv]"
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --output-format xml summary x.pcap"
Enter
Sleep 2s
