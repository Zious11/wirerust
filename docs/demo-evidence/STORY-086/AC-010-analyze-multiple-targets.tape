Output AC-010-analyze-multiple-targets.gif
Output AC-010-analyze-multiple-targets.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-010 (BC-2.12.006): multiple positional targets accepted in order
# IO error for each file proves clap parsed all three; not a usage/parse error
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze a.pcap b.pcap c.pcap"
Enter
Sleep 2s
