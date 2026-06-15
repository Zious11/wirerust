Output AC-010-etherparse-version.gif
Output AC-010-etherparse-version.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-111"
Enter
Sleep 1s
Show

Type "# STORY-111 AC-010: etherparse version in Cargo.toml"
Enter
Sleep 0.3s
Type "grep etherparse Cargo.toml"
Enter
Sleep 1s

Type "# AC-010: release build succeeds on etherparse 0.20"
Enter
Sleep 0.3s
Type "cargo build --release 2>&1 | tail -3"
Enter
Sleep 12s
