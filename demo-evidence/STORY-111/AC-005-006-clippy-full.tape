Output AC-005-006-clippy-full.gif
Output AC-005-006-clippy-full.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-111"
Enter
Sleep 1s
Show

Type "# AC-005/AC-006: three-way dispatch + unreachable! arms compile clean"
Enter
Sleep 0.3s
Type "# AC-010: cargo clippy -D warnings (no warnings = clean)"
Enter
Sleep 0.3s
Type "cargo clippy --all-targets -- -D warnings 2>&1 && echo 'CLIPPY CLEAN'"
Enter
Sleep 8s

Type "cargo fmt --check 2>&1 && echo 'FMT CLEAN'"
Enter
Sleep 5s
