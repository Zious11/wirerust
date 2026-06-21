Output AC-001-002-003-error-paths.gif
Output AC-001-002-003-error-paths.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 920
Set Height 380
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

Type "# AC-002: EPB before any IDB -- graceful E-INP-009, no panic"
Enter
Sleep 400ms
Type "$WIRERUST analyze epb_before_idb.pcapng"
Enter
Sleep 3s

Type "# AC-003: EPB with OOB interface_id -- graceful E-INP-010, no panic"
Enter
Sleep 400ms
Type "$WIRERUST analyze epb_oob_interface_id.pcapng"
Enter
Sleep 3s
