Output EC-001-threats-before-subcommand.gif
Output EC-001-threats-before-subcommand.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# EC-001 (BC-2.13.001 postcondition 1, edge case): --threats placed before subcommand
# Also rejected — clap treats unknown global args as errors
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-096/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST --threats analyze test.pcap"
Enter
Sleep 2s
