Output EC-003-hosts-on-analyze-error.gif
Output EC-003-hosts-on-analyze-error.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# EC-003: --hosts passed to analyze subcommand → clap unexpected-argument error
# --hosts is summary-only; clap rejects it on analyze
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze cap.pcap --hosts"
Enter
Sleep 2s
