# STORY-147 AC-147-002: the REAL cargo-mutants 27.0.0 binary also rejects an
# invalid `jobs` key with a fatal parse error (not just the guard test), and
# succeeds again once the file is reverted to the shipped, valid config.
# The absolute-path fragment cargo-mutants emits in its error message is
# scrubbed live via `sed` (PG-W70-DEMO-SCRUB) — the underlying error and its
# cause chain are real, unedited tool output.
# Traces: STORY-147 AC-147-002.

Output AC-147-002-cargo-mutants-tool-enforcement.gif
Output AC-147-002-cargo-mutants-tool-enforcement.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 800
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

Type "# Negative path: inject the invalid `jobs` key"
Enter
Sleep 300ms
Type "echo 'jobs = 1' >> .cargo/mutants.toml"
Enter
Wait
Sleep 300ms
Type "cargo mutants --list 2>&1 | sed -E 's#/[A-Za-z0-9_./-]*/\.cargo/mutants\.toml#<repo>/.cargo/mutants.toml#' | head -6"
Enter
Wait
Sleep 1500ms

Type "# Revert and confirm the real tool succeeds again"
Enter
Sleep 300ms
Type "git checkout -- .cargo/mutants.toml"
Enter
Wait
Sleep 300ms
Type "cargo mutants --list 2>/dev/null | head -3"
Enter
Wait
Sleep 2s
