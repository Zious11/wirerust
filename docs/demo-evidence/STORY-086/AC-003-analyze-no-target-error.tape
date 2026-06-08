Output AC-003-analyze-no-target-error.gif
Output AC-003-analyze-no-target-error.webm

Set Shell "bash"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 20
Set Theme "Dracula"
Set PlaybackSpeed 1.0

# AC-003 (BC-2.12.001 invariant 1): analyze with no targets → clap required-arg error
# clap prints "the following required arguments were not provided: <TARGETS>..."
Hide
Type 'export WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-086/target/debug/wirerust'
Enter
Sleep 300ms
Show

Type "$WIRERUST analyze"
Enter
Sleep 2s
