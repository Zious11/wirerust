Output docs/demo-evidence/STORY-176/AC-176-001-gate-success.gif
Output docs/demo-evidence/STORY-176/AC-176-001-gate-success.webm

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

# ── AC-176-001: green-doc-tense gate success path ─────────────────────────
Type "echo '=== AC-176-001: check-green-doc-tense gate (SUCCESS path) ==='"
Enter
Sleep 500ms

Type "python3 bin/check-green-doc-tense"
Enter
Wait+Screen /PASS/
Sleep 2s
