Output AC-007-non-whitelisted-idb.gif
Output AC-007-non-whitelisted-idb.webm

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

Type "echo '=== AC-007: Non-whitelisted IDB (IEEE802_11=105) → E-INP-001 ==='"
Enter
Sleep 500ms

Type "echo '# Graceful rejection, no panic, lists all 5 supported link types'"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-124/target/release/wirerust analyze /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-124/fixtures/non_whitelisted_idb.pcapng"
Enter
Sleep 3s
