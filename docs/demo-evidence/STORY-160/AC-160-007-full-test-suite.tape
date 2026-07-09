Output docs/demo-evidence/STORY-160/AC-160-007-full-test-suite.gif
Output docs/demo-evidence/STORY-160/AC-160-007-full-test-suite.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-007: Full cargo test --all-targets passes after all enum casing changes and
# existing test updates (test_BC_2_11_001_top_level_keys updated to 6-key vec,
# src/analyzer/arp.rs stale comment updated, all 14 new BC-driven tests in suite).

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "cargo test --all-targets 2>&1 | grep -E '^test result:|^running [0-9]'"
Enter
Sleep 20s
