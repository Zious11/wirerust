Output AC-010-011-reassemble-conflict.gif
Output AC-010-011-reassemble-conflict.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-010 (BC-2.12.007 postcondition 1): --reassemble --no-reassemble → clap conflict error
# AC-011 (BC-2.12.007 invariant 1): reversed order --no-reassemble --reassemble → also conflict error
# Expected: "error: the argument '--reassemble' cannot be used with '--no-reassemble'"
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --reassemble --no-reassemble analyze x.pcap"
Enter
Sleep 1s

Type "$BIN --no-reassemble --reassemble analyze x.pcap"
Enter
Sleep 2s
