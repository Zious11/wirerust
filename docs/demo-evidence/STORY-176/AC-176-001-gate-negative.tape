Output docs/demo-evidence/STORY-176/AC-176-001-gate-negative.gif
Output docs/demo-evidence/STORY-176/AC-176-001-gate-negative.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 460
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

# ── AC-176-001: negative path — gate detects stub-era pattern ─────────────
Type "echo '=== AC-176-001: NEGATIVE PATH — gate catches stub-era comment ==='"
Enter
Sleep 500ms

# Create temp file with violating comment
Type "printf '// harness skeleton compiles only -- wiring deferred\\n' > tests/demo_stub_prose_tests.rs"
Enter
Sleep 300ms

# Stage it so git ls-files picks it up
Type "git add tests/demo_stub_prose_tests.rs"
Enter
Sleep 300ms

# Run gate — should FLAG the line
Type "python3 bin/check-green-doc-tense"
Enter
Wait+Screen /FAIL/
Sleep 1500ms

# Delete the temp file and unstage
Type "git restore --staged tests/demo_stub_prose_tests.rs && rm tests/demo_stub_prose_tests.rs"
Enter
Sleep 400ms

# Verify git status clean of temp file
Type "git status --porcelain | grep demo_stub || echo 'CLEAN: demo_stub_prose_tests.rs removed'"
Enter
Sleep 500ms

# Gate returns to PASS
Type "python3 bin/check-green-doc-tense"
Enter
Wait+Screen /PASS/
Sleep 2s
