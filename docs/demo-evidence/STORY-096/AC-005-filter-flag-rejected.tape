Output AC-005-filter-flag-rejected.gif
Output AC-005-filter-flag-rejected.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-005 / EC-003 (BC-2.13.003 postcondition 1): --filter <BPF> flag not declared
# Demonstrates: wirerust analyze --filter "tcp port 80" test.pcap → clap error
# EC-003: space-separated BPF expression; clap errors on --filter before parsing rest
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-096/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type '$WIRERUST analyze --filter "tcp port 80" test.pcap'
Enter
Sleep 2s
