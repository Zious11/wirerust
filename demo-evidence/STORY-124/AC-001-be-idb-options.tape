Output AC-001-be-idb-options.gif
Output AC-001-be-idb-options.webm

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

Type "echo '=== AC-001: Big-endian pcapng with IDB options → accepted ==='"
Enter
Sleep 500ms

Type "echo '# BE SHB + BE IDB: ETHERNET (00 01 BE), if_tsresol=9 nanosecond option'"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-124/target/release/wirerust analyze /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-124/fixtures/big_endian_with_idb_options.pcapng"
Enter
Sleep 3s
