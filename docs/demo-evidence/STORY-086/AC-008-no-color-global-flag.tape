Output AC-008-no-color-global-flag.gif
Output AC-008-no-color-global-flag.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-008 (BC-2.12.003 postcondition 1): --no-color accepted before AND after subcommand
# Both placements succeed (IO error, not clap parse error) — demonstrates global flag semantics
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST --no-color analyze cap.pcap"
Enter
Sleep 1500ms

Type "$WIRERUST analyze cap.pcap --no-color"
Enter
Sleep 2s
