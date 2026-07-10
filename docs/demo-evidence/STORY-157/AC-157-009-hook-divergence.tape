Output docs/demo-evidence/STORY-157/AC-157-009-hook-divergence.gif
Output docs/demo-evidence/STORY-157/AC-157-009-hook-divergence.webm

Require grep

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 650
Set Padding 20

Sleep 1s
Type "# AC-157-009: CLAUDE.md documents PG-HASH-HOOK-DIVERGENCE"
Enter
Sleep 0.5s
Type "grep -n 'Known Tool Divergences' CLAUDE.md"
Enter
Sleep 1s
Type "grep -A 28 'Known Tool Divergences' CLAUDE.md"
Enter
Sleep 3s
