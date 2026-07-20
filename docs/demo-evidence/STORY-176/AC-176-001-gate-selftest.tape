Output docs/demo-evidence/STORY-176/AC-176-001-gate-selftest.gif
Output docs/demo-evidence/STORY-176/AC-176-001-gate-selftest.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 320
Set Theme "Dracula"
Set Padding 20
Set WaitTimeout 60s

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

# ── AC-176-001: self-test suite (91 passed, 0 failed) ──────────────────────
Type "echo '=== AC-176-001: test_check_green_doc_tense.py (91 passed, 0 failed) ==='"
Enter
Sleep 500ms

Type "python3 bin/test_check_green_doc_tense.py 2>&1 | tail -3"
Enter
Wait+Screen /Results:/
Sleep 2s
