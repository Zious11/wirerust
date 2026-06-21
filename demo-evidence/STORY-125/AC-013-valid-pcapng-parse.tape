Output AC-013-valid-pcapng-parse.gif
Output AC-013-valid-pcapng-parse.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 920
Set Height 340
Set Theme "Dracula"
Set Padding 20

Hide
Type "WIRERUST=/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-125/target/release/wirerust"
Enter
Type "cd /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-125"
Enter
Type "clear"
Enter
Show

Type "# AC-013: Valid SHB+IDB+3xEPB pcapng parses and reports packets"
Enter
Sleep 400ms
Type "$WIRERUST summary --no-color multi_packet.pcapng"
Enter
Sleep 3s
