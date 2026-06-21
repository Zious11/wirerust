Output AC-002-le-be-endianness.gif
Output AC-002-le-be-endianness.webm

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

Type "echo '=== Little-endian pcapng (BOM: 4D 3C 2B 1A) ==='"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-123/target/release/wirerust analyze /tmp/wirerust-demo/minimal_le.pcapng"
Enter
Sleep 3s

Type "echo '=== Big-endian pcapng (BOM: 1A 2B 3C 4D) ==='"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-123/target/release/wirerust analyze /tmp/wirerust-demo/minimal_be.pcapng"
Enter
Sleep 3s
