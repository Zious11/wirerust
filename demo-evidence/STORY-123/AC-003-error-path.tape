Output AC-003-error-path.gif
Output AC-003-error-path.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "clear"
Enter
Sleep 800ms
Show

Type "echo '=== Error path: unrecognized magic bytes ==='"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-123/target/release/wirerust analyze /tmp/wirerust-demo/bad_magic.bin"
Enter
Sleep 3s

Type "echo '=== Success path: valid LE pcapng ==='"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-123/target/release/wirerust analyze /tmp/wirerust-demo/minimal_le.pcapng"
Enter
Sleep 3s
