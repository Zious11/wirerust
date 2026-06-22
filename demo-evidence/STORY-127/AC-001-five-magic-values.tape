Output AC-001-five-magic-values.gif
Output AC-001-five-magic-values.webm

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 560
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-001 / BC-2.12.011 PC1 + Inv1 + Inv2:
# All 5 magic values detected by content regardless of file extension.
# Files: .PCAP (uppercase), .CAP, .data, .txt, .bin all detected.
# Wrong-magic .pcap silently excluded.
# Observable: "Skipped: 5 packets (decode errors)" -- one per magic variant.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
Enter
Sleep 1s
Show

Type "bash /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-127/demo-ac001-five-magics.sh"
Enter
Sleep 7s
