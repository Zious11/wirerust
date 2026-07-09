Output .worktrees/STORY-161/docs/demo-evidence/STORY-161/AC-001-002-vp-index-algorithm.gif
Output .worktrees/STORY-161/docs/demo-evidence/STORY-161/AC-001-002-vp-index-algorithm.webm

Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Padding 20

Hide
Type "VP_INDEX='.factory/specs/verification-properties/VP-INDEX.md'"
Enter
Show

Sleep 0.5s
Type "# AC-161-001: Multi-File Proof Anchor Algorithm section in VP-INDEX v2.39"
Enter
Sleep 0.3s
Type "grep -n 'Multi-File Proof Anchor\\|proof_file_hash' $VP_INDEX | head -5"
Enter
Sleep 3s

Type "# Show the algorithm section heading and version bump"
Enter
Sleep 0.3s
Type "grep -n '^## Multi-File Proof Anchor\\|version:' $VP_INDEX | head -5"
Enter
Sleep 2s

Type "# AC-161-002: Section-Scoping Rule (non-empty output required)"
Enter
Sleep 0.3s
Type "grep -A3 -E 'Section-Scoping Rule|section.scoping|closing brace' $VP_INDEX | head -12"
Enter
Sleep 3s

Sleep 2s
