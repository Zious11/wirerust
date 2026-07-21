# STORY-147 AC-147-002: the guard test (`repo_mutation_config_tests`)
# enforces `.cargo/mutants.toml` content validity — 9/9 green on the shipped
# file. Negative path: temporarily appending an invalid `jobs` key causes
# the allowlist/no-jobs assertions to FAIL with the fatal-key message; the
# file is then reverted (`git checkout -- .cargo/mutants.toml`) and the
# suite is shown clean again.
# Traces: STORY-147 AC-147-002.

Output AC-147-002-guard-test-success-negative-revert.gif
Output AC-147-002-guard-test-success-negative-revert.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 900
Set Padding 24
Set Theme "Dracula"
Set TypingSpeed 60ms

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-147"
Enter
Wait
Type "clear"
Enter
Wait
Show

Type "# Success path: guard test suite is green on the shipped config"
Enter
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests"
Enter
Wait
Sleep 1s

Type "# Negative path: inject an invalid `jobs` key, then re-run"
Enter
Sleep 300ms
Type "echo 'jobs = 1' >> .cargo/mutants.toml"
Enter
Wait
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests"
Enter
Wait
Sleep 1500ms

Type "# Revert and confirm clean again"
Enter
Sleep 300ms
Type "git checkout -- .cargo/mutants.toml"
Enter
Wait
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests"
Enter
Wait
Sleep 2s
