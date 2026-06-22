Output AC-000-test-suite.gif
Output AC-000-test-suite.webm

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# STORY-127 complete test suite: all 9 ACs pass.
# BC-2.12.011: 5 magic values, silent skip, short file, .cap extension,
# sorted output, empty dir, I/O error skip, subdir skip, E2E corpus.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
Enter
Sleep 1s
Show

Type "cargo test --test bc_2_12_011_story127_tests"
Enter
Sleep 20s
