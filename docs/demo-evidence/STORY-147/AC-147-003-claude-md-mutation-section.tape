# STORY-147 AC-147-003: CLAUDE.md carries a "### Mutation testing" section
# with all required markers (low-parallelism guidance, --jobs warning,
# rationale, PG-MUTANTS-JOBS-001 + fix-tls-clienthello-frag reference,
# #654 upstream pointer, minimum_test_timeout note). Negative path:
# temporarily dropping the PG-MUTANTS-JOBS-001 line makes the guard test
# FAIL; the file is then reverted and the test is shown green again.
# Traces: STORY-147 AC-147-003.

Output AC-147-003-claude-md-mutation-section.gif
Output AC-147-003-claude-md-mutation-section.webm

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

Type "# Success path: the guard test for CLAUDE.md's Mutation testing note is green"
Enter
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests test_AC_147_003"
Enter
Wait
Sleep 1s

Type 'grep -A 12 "### Mutation testing" CLAUDE.md'
Enter
Wait
Sleep 1500ms

Type "# Negative path: drop the PG-MUTANTS-JOBS-001 reference, then re-run"
Enter
Sleep 300ms
Type "sed -i '' '/PG-MUTANTS-JOBS-001/d' CLAUDE.md"
Enter
Wait
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests test_AC_147_003"
Enter
Wait
Sleep 1500ms

Type "# Revert and confirm clean again"
Enter
Sleep 300ms
Type "git checkout -- CLAUDE.md"
Enter
Wait
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests test_AC_147_003"
Enter
Wait
Sleep 2s
