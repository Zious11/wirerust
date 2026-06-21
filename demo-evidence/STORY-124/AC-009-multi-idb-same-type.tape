Output AC-009-multi-idb-same-type.gif
Output AC-009-multi-idb-same-type.webm

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

Type "echo '=== AC-009: Multi-IDB same link type → accepted ==='"
Enter
Sleep 500ms

Type "echo '# 2 IDBs: interface 0 = ETHERNET, interface 1 = ETHERNET (agreement)'"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-124/target/release/wirerust analyze /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-124/fixtures/multi_idb_same_type.pcapng"
Enter
Sleep 3s
