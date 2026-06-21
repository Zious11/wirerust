Output AC-008-multi-idb-conflict.gif
Output AC-008-multi-idb-conflict.webm

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

Type "echo '=== AC-008: Multi-IDB link-type conflict → E-INP-011 ==='"
Enter
Sleep 500ms

Type "echo '# 2 IDBs: interface 0 = ETHERNET, interface 1 = LINUX_SLL'"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-124/target/release/wirerust analyze /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-124/fixtures/multi_idb_conflict.pcapng"
Enter
Sleep 3s
