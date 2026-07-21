Output docs/demo-evidence/STORY-176/AC-176-003-gitignore-glob.gif
Output docs/demo-evidence/STORY-176/AC-176-003-gitignore-glob.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 900
Set Height 500
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

# ── AC-176-003: .gitignore mutants.out* glob ──────────────────────────────
Type "echo '=== AC-176-003: .gitignore mutants.out* glob + regression guard ==='"
Enter
Sleep 500ms

# (a) Pattern present in .gitignore
Type "grep -n 'mutants.out' .gitignore"
Enter
Wait+Screen /mutants.out/
Sleep 800ms

# (b) git check-ignore confirms both directories are ignored
Type "mkdir -p mutants.out mutants.out.j4-invalid"
Enter
Sleep 300ms
Type "git status --porcelain | grep mutants || echo 'NO-UNTRACKED-MUTANTS-DIRS'"
Enter
Sleep 600ms
Type "rmdir mutants.out mutants.out.j4-invalid"
Enter
Sleep 300ms

# (c) Regression guard self-test: 2/0
Type "python3 bin/test_gitignore_mutants_glob.py"
Enter
Wait+Screen /Results:/
Sleep 800ms

# (d) CI wiring
Type "grep -n 'test_gitignore_mutants_glob' .github/workflows/ci.yml"
Enter
Wait+Screen /test_gitignore_mutants_glob/
Sleep 2s
