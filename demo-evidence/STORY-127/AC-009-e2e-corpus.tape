Output AC-009-e2e-corpus.gif
Output AC-009-e2e-corpus.webm

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-009 / BC-2.12.011 EC-001..002: E2E corpus wiring.
# Sub-case 1: smb3.pcapng -> full reader stack -> 54 SMB packets.
# Sub-case 2: synthetic 16-EPB pcapng with .cap extension -> 16 decode events.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
Enter
Sleep 1s
Show

Type "bash /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-127/demo-ac009-e2e.sh"
Enter
Sleep 8s
