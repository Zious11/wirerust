Output docs/demo-evidence/STORY-176/AC-176-002-rebaseline-note.gif
Output docs/demo-evidence/STORY-176/AC-176-002-rebaseline-note.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 260
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 30s

# Setup: navigate to worktree and set clean prompt
Hide
Type "cd <repo>/.worktrees/STORY-176"
Enter
Sleep 300ms
Type "export PS1='$ '"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

# ── AC-176-002: delivery-doc re-baseline note present ────────────────────
Type "echo '=== AC-176-002: re-baseline note in delivery-doc-currency-protocol.md ==='"
Enter
Sleep 500ms

Type "grep -n 're-baseline' ../../.factory/maintenance/delivery-doc-currency-protocol.md"
Enter
Wait+Screen /re-baseline/
Sleep 2s
