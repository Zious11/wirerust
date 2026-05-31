Output AC-012-reassemble-alone.gif
Output AC-012-reassemble-alone.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set FontFamily "Menlo"
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-012 (BC-2.12.007 edge case EC-003): --reassemble alone → accepted (cli.reassemble = true)
# "Error: Target not found" proves clap accepted the flag; a clap error would show "error: ..."
Hide
Type 'export BIN=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-087/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$BIN --reassemble analyze x.pcap"
Enter
Sleep 2s
