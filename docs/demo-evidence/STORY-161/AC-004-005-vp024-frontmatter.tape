Output .worktrees/STORY-161/docs/demo-evidence/STORY-161/AC-004-005-vp024-frontmatter.gif
Output .worktrees/STORY-161/docs/demo-evidence/STORY-161/AC-004-005-vp024-frontmatter.webm

Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Padding 20

Hide
Type "VP024='.factory/specs/verification-properties/vp-024-arp-parse-safety.md'"
Enter
Show

Sleep 0.5s
Type "# AC-161-004: VP-024 gains kani_version sibling field (historical, LMR-002)"
Enter
Sleep 0.3s
Type "grep -n 'kani_version\\|proof_file_hash\\|verified_at_commit' $VP024 | head -5"
Enter
Sleep 3s

Type "# AC-161-005: verification_lock: true unchanged; version 2.5; FU-F6-KANI-CLEANUP absent"
Enter
Sleep 0.3s
Type "grep -n 'verification_lock\\|^version:\\|FU-F6-KANI-CLEANUP' $VP024 | head -5"
Enter
Sleep 3s

Type "# Confirm proof_file_hash is the full 64-char hex (not null)"
Enter
Sleep 0.3s
Type "grep 'proof_file_hash' $VP024 | head -1"
Enter
Sleep 2s

Sleep 2s
