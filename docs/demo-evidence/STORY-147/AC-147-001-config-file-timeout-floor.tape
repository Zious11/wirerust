# STORY-147 AC-147-001: `.cargo/mutants.toml` exists at the exact path
# cargo-mutants reads by default and sets `minimum_test_timeout = 300`.
# Negative/decoy path: a repo-root `mutants.toml` (silently ignored by
# cargo-mutants) does NOT exist.
# Traces: STORY-147 AC-147-001.

Output AC-147-001-config-file-timeout-floor.gif
Output AC-147-001-config-file-timeout-floor.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 760
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

Type "# AC-147-001: .cargo/mutants.toml sets a >=300s timeout floor"
Enter
Sleep 300ms
Type "cat .cargo/mutants.toml"
Enter
Wait
Sleep 1s

Type "# Negative/decoy path: cargo-mutants does NOT read a repo-root mutants.toml"
Enter
Sleep 300ms
Type "ls mutants.toml"
Enter
Wait
Sleep 2s
