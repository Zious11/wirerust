# STORY-147 AC-147-004: self-audit conjunction — BOTH defenses (the
# `.cargo/mutants.toml` timeout floor AND the CLAUDE.md Mutation testing
# note) are present simultaneously. Shows the conjunction test passing plus
# a combined config+doc view. Negative path: breaking ONE defense (the
# config, via an invalid `jobs` key) makes the conjunction test FAIL; the
# file is then reverted and the test is shown green again.
# Traces: STORY-147 AC-147-004.

Output AC-147-004-conjunction-both-defenses.gif
Output AC-147-004-conjunction-both-defenses.webm

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

Type "# Success path: conjunction self-audit test is green"
Enter
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests test_AC_147_004"
Enter
Wait
Sleep 1s

Type "# Combined view: config timeout floor + doc guidance, side by side"
Enter
Sleep 300ms
Type "cat .cargo/mutants.toml; echo '---'; grep -A 3 '### Mutation testing' CLAUDE.md"
Enter
Wait
Sleep 1500ms

Type "# Negative path: break ONE defense (config) and re-run the conjunction test"
Enter
Sleep 300ms
Type "echo 'jobs = 1' >> .cargo/mutants.toml"
Enter
Wait
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests test_AC_147_004"
Enter
Wait
Sleep 1500ms

Type "# Revert and confirm both defenses green again"
Enter
Sleep 300ms
Type "git checkout -- .cargo/mutants.toml"
Enter
Wait
Sleep 300ms
Type "cargo test --test repo_mutation_config_tests test_AC_147_004"
Enter
Wait
Sleep 2s
